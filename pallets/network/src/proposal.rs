// Copyright (C) Hypertensor.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use sp_runtime::traits::TrailingZeroInput;

impl<T: Config> Pallet<T> {
  // TODO: Max vector string limit
  pub fn do_propose(
    account_id: T::AccountId, 
    subnet_id: u32,
    peer_id: PeerId,
    data: Vec<u8>,
  ) -> DispatchResult {
    // --- Ensure subnet exists
    let subnet = match SubnetsData::<T>::try_get(subnet_id) {
      Ok(subnet) => subnet,
      Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
    };

    let block: u64 = Self::get_current_block_as_u64();
    let epoch: u64 = block / T::EpochLength::get();

    // --- Ensure proposer account has peer and is accountant
    match SubnetNodesData::<T>::try_get(
      subnet_id, 
      account_id.clone()
    ) {
      Ok(subnet_node) => subnet_node.has_classification(&SubnetNodeClass::Submittable, epoch as u64),
      Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
    };

    // Unique subnet_id -> PeerId
    // Ensure peer ID exists within subnet
    let defendant_account_id = match SubnetNodeAccount::<T>::try_get(subnet_id, peer_id.clone()) {
      Ok(defendant_account_id) => defendant_account_id,
      Err(()) => return Err(Error::<T>::PeerIdNotExist.into()),
    };

    // --- Disputed account_id cannot be the proposer
    ensure!(
      defendant_account_id.clone() != account_id.clone(),
      Error::<T>::PlaintiffIsDefendant
    );

    // --- Ensure the minimum required subnet peers exist
    // --- Only submittable can vote on proposals
    // --- Get all eligible voters from this block
    let subnet_nodes: BTreeSet<T::AccountId> = Self::get_classified_accounts(subnet_id, &SubnetNodeClass::Submittable, epoch);
    let subnet_nodes_count = subnet_nodes.len();

    // There must always be the required minimum subnet peers for each vote
    // This ensure decentralization in order for proposals to be accepted 

    // safe unwrap after `contains_key`
    ensure!(
      subnet_nodes_count as u32 >= subnet.min_nodes,
      Error::<T>::SubnetNodesMin
    );

    // --- Ensure min nodes for proposals
    ensure!(
      subnet_nodes_count as u32 >= ProposalMinSubnetNodes::<T>::get(),
      Error::<T>::SubnetNodesMin
    );

    ensure!(
      !Self::account_has_active_proposal_as_plaintiff(
        subnet_id, 
        account_id.clone(), 
        block,
      ),
      Error::<T>::NodeHasActiveProposal
    );

    ensure!(
      !Self::account_has_active_proposal_as_defendant(
        subnet_id, 
        defendant_account_id.clone(), 
        block,
      ),
      Error::<T>::NodeHasActiveProposal
    );

    let proposal_bid_amount: u128 = ProposalBidAmount::<T>::get();
    let proposal_bid_amount_as_balance = Self::u128_to_balance(proposal_bid_amount);

    let can_withdraw: bool = Self::can_remove_balance_from_coldkey_account(
      &account_id.clone(),
      proposal_bid_amount_as_balance.unwrap(),
    );

    ensure!(
      can_withdraw,
      Error::<T>::NotEnoughBalanceToBid
    );

    // --- Withdraw bid amount from proposer accounts
    let _ = T::Currency::withdraw(
      &account_id.clone(),
      proposal_bid_amount_as_balance.unwrap(),
      WithdrawReasons::except(WithdrawReasons::TRANSFER),
      ExistenceRequirement::KeepAlive,
    );

    let proposal_id = ProposalsCount::<T>::get();

    // TODO: Test adding quorum and consensus into the Proposal storage
    //       by using the amount of nodes in the subnet
    //       It's possible the quorum or consensus for smaller subnets may not be divisible
    Proposals::<T>::insert(
      subnet_id,
      proposal_id,
      ProposalParams {
        subnet_id: subnet_id,
        plaintiff: account_id.clone(),
        defendant: defendant_account_id.clone(),
        plaintiff_bond: proposal_bid_amount,
        defendant_bond: 0,
        eligible_voters: subnet_nodes,
        votes: VoteParams {
          yay: BTreeSet::new(),
          nay: BTreeSet::new(),
        },
        start_block: block,
        challenge_block: 0, // No challenge block initially
        plaintiff_data: data.clone(),
        defendant_data: Vec::new(),
        complete: false,
      }
    );

    ProposalsCount::<T>::put(proposal_id + 1);

    Self::deposit_event(
      Event::Proposal { 
        subnet_id: subnet_id, 
        proposal_id: proposal_id,
        epoch: epoch as u32,
        plaintiff: account_id, 
        defendant: defendant_account_id,
        plaintiff_data: data
      }
    );

    Ok(())
  }

  pub fn do_attest_proposal(
    account_id: T::AccountId, 
    subnet_id: u32,
    proposal_id: u32,
    data: Vec<u8>,
  ) -> DispatchResult {
    let proposal = match Proposals::<T>::try_get(subnet_id, proposal_id) {
      Ok(proposal) => proposal,
      Err(()) =>
        return Err(Error::<T>::ProposalInvalid.into()),
    };

    Self::deposit_event(
      Event::ProposalAttested{ 
        subnet_id: subnet_id, 
        proposal_id: proposal_id, 
        account_id: account_id,
        attestor_data: data
      }
    );

    Ok(())
  }

  pub fn do_challenge_proposal(
    account_id: T::AccountId, 
    subnet_id: u32,
    proposal_id: u32,
    data: Vec<u8>,
  ) -> DispatchResult {
    let proposal = match Proposals::<T>::try_get(subnet_id, proposal_id) {
      Ok(proposal) => proposal,
      Err(()) =>
        return Err(Error::<T>::ProposalInvalid.into()),
    };

    // --- Ensure defendant
    ensure!(
      account_id == proposal.defendant,
      Error::<T>::NotDefendant
    );

    // --- Ensure incomplete
    ensure!(
      !proposal.complete,
      Error::<T>::ProposalComplete
    );
    
    let challenge_period = ChallengePeriod::<T>::get();
    let block: u64 = Self::get_current_block_as_u64();

    // --- Ensure challenge period is active
    ensure!(
      block < proposal.start_block + challenge_period,
      Error::<T>::ProposalChallengePeriodPassed
    );

    // --- Ensure unchallenged
    ensure!(
      proposal.challenge_block == 0,
      Error::<T>::ProposalChallenged
    );

    // --- Get plaintiffs bond to match
    // We get the plaintiff bond in case this amount is updated in between proposals
    let proposal_bid_amount_as_balance = Self::u128_to_balance(proposal.plaintiff_bond);

    let can_withdraw: bool = Self::can_remove_balance_from_coldkey_account(
      &account_id,
      proposal_bid_amount_as_balance.unwrap(),
    );

    // --- Ensure can bond
    ensure!(
      can_withdraw,
      Error::<T>::NotEnoughBalanceToBid
    );

    // --- Withdraw bid amount from proposer accounts
    let _ = T::Currency::withdraw(
      &account_id,
      proposal_bid_amount_as_balance.unwrap(),
      WithdrawReasons::except(WithdrawReasons::TRANSFER),
      ExistenceRequirement::KeepAlive,
    );

    let epoch: u64 = block / T::EpochLength::get();

    Proposals::<T>::mutate(
      subnet_id,
      proposal_id,
      |params: &mut ProposalParams<T::AccountId>| {
        params.defendant_data = data.clone();
        params.defendant_bond = proposal.plaintiff_bond;
        params.challenge_block = block;
      }
    );

    Self::deposit_event(
      Event::ProposalChallenged { 
        subnet_id: subnet_id, 
        proposal_id: proposal_id,
        defendant: account_id, 
        defendant_data: data,
      }
    );

    Ok(())
  }

  pub fn do_vote(
    account_id: T::AccountId, 
    subnet_id: u32,
    proposal_id: u32,
    vote: VoteType
  ) -> DispatchResult {
    let proposal = match Proposals::<T>::try_get(subnet_id, proposal_id) {
      Ok(proposal) => proposal,
      Err(()) =>
        return Err(Error::<T>::ProposalInvalid.into()),
    };

    let plaintiff = proposal.plaintiff;
    let defendant = proposal.defendant;

    // --- Ensure not plaintiff or defendant
    ensure!(
      account_id.clone() != plaintiff && account_id.clone() != defendant,
      Error::<T>::PartiesCannotVote
    );

    // --- Ensure account has peer
    // Proposal voters are calculated within ``do_proposal`` as ``eligible_voters`` so we check if they
    // are still nodes
    ensure!(
      SubnetNodesData::<T>::contains_key(subnet_id, account_id.clone()),
      Error::<T>::SubnetNodeNotExist
    );
    
    // --- Ensure challenged
    ensure!(
      proposal.challenge_block != 0,
      Error::<T>::ProposalUnchallenged
    );

    // --- Ensure incomplete
    ensure!(
      !proposal.complete,
      Error::<T>::ProposalComplete
    );
    
    let voting_period = VotingPeriod::<T>::get();
    let block: u64 = Self::get_current_block_as_u64();

    // --- Ensure voting period is active
    // Voting period starts after the challenge block
    ensure!(
      block < proposal.challenge_block + voting_period,
      Error::<T>::VotingPeriodInvalid
    );

    // --- Ensure is eligible to vote
    ensure!(
      proposal.eligible_voters.get(&account_id).is_some(),
      Error::<T>::NotEligible
    );

    let yays: BTreeSet<T::AccountId> = proposal.votes.yay;
    let nays: BTreeSet<T::AccountId> = proposal.votes.nay;

    // --- Ensure hasn't already voted
    ensure!(
      yays.get(&account_id) == None && nays.get(&account_id) == None,
      Error::<T>::AlreadyVoted
    );

    Proposals::<T>::mutate(
      subnet_id,
      proposal_id,
      |params: &mut ProposalParams<T::AccountId>| {
        if vote == VoteType::Yay {
          params.votes.yay.insert(account_id.clone());
        } else {
          params.votes.nay.insert(account_id.clone());
        };  
      }
    );
    
    Self::deposit_event(
      Event::ProposalVote { 
        subnet_id: subnet_id, 
        proposal_id: proposal_id,
        account_id: account_id,
        vote: vote,
      }
    );

    Ok(())
  }

  pub fn do_cancel_proposal(
    account_id: T::AccountId, 
    subnet_id: u32,
    proposal_id: u32,
  ) -> DispatchResult {
    let proposal = match Proposals::<T>::try_get(subnet_id, proposal_id) {
      Ok(proposal) => proposal,
      Err(()) =>
        return Err(Error::<T>::ProposalInvalid.into()),
    };

    // --- Ensure plaintiff
    ensure!(
      account_id == proposal.plaintiff,
      Error::<T>::NotPlaintiff
    );
    
    // --- Ensure unchallenged
    ensure!(
      proposal.challenge_block == 0,
      Error::<T>::ProposalChallenged
    );

    // --- Ensure incomplete
    ensure!(
      !proposal.complete,
      Error::<T>::ProposalComplete
    );

    // --- Remove proposal
    Proposals::<T>::remove(subnet_id, proposal_id);

    let plaintiff_bond_as_balance = Self::u128_to_balance(proposal.plaintiff_bond);

    // Give plaintiff bond back
    T::Currency::deposit_creating(&proposal.plaintiff, plaintiff_bond_as_balance.unwrap());

    Self::deposit_event(
      Event::ProposalCanceled { 
        subnet_id: subnet_id, 
        proposal_id: proposal_id,
      }
    );

    Ok(())
  }

  /// Finalize the proposal and come to a conclusion
  /// Either plaintiff or defendant win, or neither win if no consensus or quorum is met
  pub fn do_finalize_proposal(
    account_id: T::AccountId, 
    subnet_id: u32,
    proposal_id: u32,
  ) -> DispatchResult {
    let proposal = match Proposals::<T>::try_get(subnet_id, proposal_id) {
      Ok(proposal) => proposal,
      Err(()) =>
        return Err(Error::<T>::ProposalInvalid.into()),
    };

    // --- Ensure challenged
    ensure!(
      proposal.challenge_block != 0,
      Error::<T>::ProposalUnchallenged
    );

    // --- Ensure incomplete
    ensure!(
      !proposal.complete,
      Error::<T>::ProposalComplete
    );
    
    let voting_period = VotingPeriod::<T>::get();
    let block: u64 = Self::get_current_block_as_u64();

    // --- Ensure voting period is completed
    ensure!(
      block > proposal.challenge_block + voting_period,
      Error::<T>::VotingPeriodInvalid
    );

    // TODO: include enactment period for executing proposals

    // --- Ensure quorum reached
    let yays_len: u128 = proposal.votes.yay.len() as u128;
    let nays_len: u128 = proposal.votes.nay.len() as u128;
    let voters_len: u128 = proposal.eligible_voters.len() as u128;
    let voting_percentage: u128 = Self::percent_div(yays_len + nays_len, voters_len);

    let yays_percentage: u128 = Self::percent_div(yays_len, voters_len);
    let nays_percentage: u128 = Self::percent_div(nays_len, voters_len);

    let plaintiff_bond_as_balance = Self::u128_to_balance(proposal.plaintiff_bond);
    let defendant_bond_as_balance = Self::u128_to_balance(proposal.defendant_bond);

    let quorum_reached: bool = voting_percentage >= ProposalQuorum::<T>::get();
    let consensus_threshold: u128 = ProposalConsensusThreshold::<T>::get();

    // --- Mark as complete
    Proposals::<T>::mutate(
      subnet_id,
      proposal_id,
      |params: &mut ProposalParams<T::AccountId>| {
        params.complete = true;
        params.plaintiff_bond = 0;
        params.defendant_bond = 0;
      }
    );

    // --- If quorum not reached and both voting options didn't succeed consensus then complete
    if !quorum_reached || 
      (yays_percentage < consensus_threshold && 
      nays_percentage < consensus_threshold && 
      quorum_reached)
    {
      // Give plaintiff and defendant bonds back
      T::Currency::deposit_creating(&proposal.plaintiff, plaintiff_bond_as_balance.unwrap());
      T::Currency::deposit_creating(&proposal.defendant, defendant_bond_as_balance.unwrap());
      return Ok(())
    }

    // --- At this point we know that one of the voting options are in consensus
    if yays_len > nays_len {
      // --- Plaintiff wins
      // --- Remove defendant
      Self::perform_remove_subnet_node(block, subnet_id, proposal.defendant);
      // --- Return bond
      T::Currency::deposit_creating(&proposal.plaintiff, plaintiff_bond_as_balance.unwrap());
      // --- Distribute bond to voters in consensus
      Self::distribute_bond(
        proposal.defendant_bond, 
        proposal.votes.yay,
        &proposal.plaintiff
      );
    } else {
      // --- Defendant wins
      T::Currency::deposit_creating(&proposal.defendant, defendant_bond_as_balance.unwrap());
      // --- Distribute bond to voters in consensus
      Self::distribute_bond(
        proposal.plaintiff_bond, 
        proposal.votes.nay,
        &proposal.defendant
      );
    }

    Self::deposit_event(
      Event::ProposalFinalized{ 
        subnet_id: subnet_id, 
        proposal_id: proposal_id, 
      }
    );

    Ok(())
  }

  pub fn distribute_bond(
    bond: u128, 
    mut distributees: BTreeSet<T::AccountId>,
    winner: &T::AccountId
  ) {
    // --- Insert winner to distributees
    //     Parties cannot vote but receive distribution
    distributees.insert(winner.clone());
    let voters_len = distributees.len();
    let distribution_amount = bond.saturating_div(voters_len as u128);
    let distribution_amount_as_balance = Self::u128_to_balance(distribution_amount);
    // Redundant
    if !distribution_amount_as_balance.is_some() {
      return
    }

    let mut total_distributed: u128 = 0;
    // --- Distribute losers bond to consensus
    for account in distributees {
      total_distributed += distribution_amount;
      T::Currency::deposit_creating(&account, distribution_amount_as_balance.unwrap());
    }

    // --- Take care of dust and send to winner
    if total_distributed < bond {
      let remaining_bond = bond - total_distributed;
      let remaining_bid_as_balance = Self::u128_to_balance(remaining_bond);
      if remaining_bid_as_balance.is_some() {
        T::Currency::deposit_creating(&winner.clone(), remaining_bid_as_balance.unwrap());
      }
    }
  }

  fn account_has_active_proposal_as_plaintiff(
    subnet_id: u32, 
    account_id: T::AccountId, 
    block: u64,
  ) -> bool {
    let challenge_period = ChallengePeriod::<T>::get();
    let voting_period = VotingPeriod::<T>::get();

    let mut active_proposal: bool = false;

    for proposal in Proposals::<T>::iter_prefix_values(subnet_id) {
      let plaintiff: T::AccountId = proposal.plaintiff;
      if plaintiff != account_id {
        continue;
      }

      // At this point we have a proposal that matches the plaintiff
      let proposal_block: u64 = proposal.start_block;
      let challenge_block: u64 = proposal.challenge_block;
      if challenge_block == 0 {
        // If time remaining for challenge
        if block < proposal.start_block + challenge_period {
          active_proposal = true;
          break;
        }
      } else {
        // If time remaining for vote
        if block < challenge_block + voting_period {
          active_proposal = true;
          break;
        }
      }
    }

    active_proposal
  }

  /// Does a subnet node have a proposal against them under the following conditions
  /// Proposal must not be completed to qualify or awaiting challenge
  fn account_has_active_proposal_as_defendant(
    subnet_id: u32, 
    account_id: T::AccountId, 
    block: u64,
  ) -> bool {
    let challenge_period = ChallengePeriod::<T>::get();
    let voting_period = VotingPeriod::<T>::get();

    let mut active_proposal: bool = false;

    // Proposals::<T>::iter_prefix_values(subnet_id)
    //   .find(|x| {
    //     .defendant == *account_id
    //   })

    for proposal in Proposals::<T>::iter_prefix_values(subnet_id) {
      let defendant: T::AccountId = proposal.defendant;
      if defendant != account_id {
        continue;
      }

      // At this point we have a proposal that matches the defendant
      let proposal_block: u64 = proposal.start_block;
      let challenge_block: u64 = proposal.challenge_block;
      if challenge_block == 0 {
        // If time remaining for challenge
        if block < proposal.start_block + challenge_period {
          active_proposal = true;
          break;
        }
      } else {
        // If time remaining for vote
        if block < challenge_block + voting_period {
          active_proposal = true;
          break;
        }
      }
    }

    active_proposal
  }

  fn remove_proposal(subnet_id: u32, proposal_id: u32) {

  }

  fn delete_completed_proposals() {

  }
}