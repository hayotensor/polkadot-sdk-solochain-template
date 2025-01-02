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

#![cfg(test)]
use crate::mock::*;
use sp_core::crypto::AccountId32;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use frame_system::pallet_prelude::OriginFor;
use sp_runtime::Percent;
use log::info;
use sp_core::{H256, U256};
use frame_support::traits::Currency;
use sp_core::OpaquePeerId as PeerId;
use crate::{
  Error, SubnetNode, PropsType, SubnetVote, VotesBalance, ReservableCurrency, PropCount, VoteType,
  Votes, ActiveProposalsCount, Proposals, PropsStatus, PropsPathStatus, BalanceOf, RegistrationSubnetData,
  ActivateProposalsCount, ActiveActivateProposals, DeactivateProposalsCount
};
use strum::IntoEnumIterator;

type AccountIdOf<Test> = <Test as frame_system::Config>::AccountId;

const DEFAULT_IP: &str = "172.2.54.234";
const DEFAULT_PORT: u16 = 5000;
const DEFAULT_DEPOSIT_AMOUNT: u128 = 10000000000000000000000; // 10,000
const DEFAULT_MODEL_PATH: &str = "hf/llama2";
const DEFAULT_EXISTING_MODEL_PATH: &str = "hf/baluga";
// Defaut is minimum stake per subnet node
const DEFAUT_VOTE_AMOUNT: u128 = 1000e+18 as u128;

fn account(id: u32) -> AccountIdOf<Test> {
	[id as u8; 32].into()
}

fn peer(id: u32) -> PeerId {
  let peer_id = format!("QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N{id}"); 
	PeerId(peer_id.into())
}

fn default_subnet_path() -> Vec<u8> {
  DEFAULT_MODEL_PATH.into()
}

fn default_add_subnet_data() -> RegistrationSubnetData {
  let subnet_data = RegistrationSubnetData {
    path: DEFAULT_MODEL_PATH.into(),
		memory_mb: 50000,
  };
  subnet_data
}

fn default_existing_add_subnet_data() -> RegistrationSubnetData {
  let subnet_data = RegistrationSubnetData {
    path: DEFAULT_EXISTING_MODEL_PATH.into(),
		memory_mb: 50000,
  };
  subnet_data
}

fn default_ip() -> Vec<u8> {
  DEFAULT_IP.into()
}

fn get_default_min_subnet_nodes() -> u32 {
  let min_subnet_nodes = <pallet_network::Pallet<Test> as SubnetVote<OriginFor<Test>, <Test as frame_system::Config>::AccountId>>::get_min_subnet_nodes(
    default_add_subnet_data().memory_mb
  );

  // let min_subnet_nodes = <pallet_network::Pallet<Test> as SubnetVote<<Test as frame_system::Config>::AccountId>>::get_min_subnet_nodes(
  //   default_add_subnet_data().memory_mb
  // );
  min_subnet_nodes
}

fn get_default_existing_min_subnet_nodes() -> u32 {
  let min_subnet_nodes = <pallet_network::Pallet<Test> as SubnetVote<OriginFor<Test>, <Test as frame_system::Config>::AccountId>>::get_min_subnet_nodes(
    default_add_subnet_data().memory_mb
  );

  // let min_subnet_nodes = <pallet_network::Pallet<Test> as SubnetVote<<Test as frame_system::Config>::AccountId>>::get_min_subnet_nodes(
  //   default_add_subnet_data().memory_mb
  // );
  min_subnet_nodes
}

fn build_existing_subnet(start: u32, end: u32) {
  let subnet_path: Vec<u8> = DEFAULT_EXISTING_MODEL_PATH.into();
  let min_subnet_nodes = pallet_network::MinSubnetNodes::<Test>::get();

  let subnet_initialization_cost = get_subnet_initialization_cost();
  let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost+1000);

  let add_subnet_data = RegistrationSubnetData {
    path: subnet_path.clone(),
    memory_mb: 50000,
  };
  assert_ok!(
    Network::activate_subnet(
      account(0),
      account(0),
      add_subnet_data,
    )
  );

  // assert_ok!(
  //   Network::vote_subnet(
  //     RuntimeOrigin::signed(account(0)), 
  //     subnet_path.clone(),
  //   )
  // );

  // let add_subnet_data = RegistrationSubnetData {
  //   path: subnet_path.clone(),
  //   memory_mb: 50000,
  // };

  // assert_ok!(
  //   Network::add_subnet(
  //     RuntimeOrigin::signed(account(0)),
  //     add_subnet_data.clone(),
  //   ) 
  // );
  // let add_subnet_data = RegistrationSubnetData {
  //   path: subnet_path.clone(),
  //   memory_mb: 50000,
  // };

  // assert_ok!(
  //   Network::activate_subnet(
  //     account(0),
  //     account(0),
  //     add_subnet_data,
  //   )
  // );  

  let subnet_id = pallet_network::SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
  let min_stake = pallet_network::MinStakeBalance::<Test>::get();

  for n in start..end {
    let _ = Balances::deposit_creating(&account(n), min_stake + 100000);
    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(n)),
        subnet_id,
        peer(n),
        // "172.20.54.234".into(),
        // 8888,
        min_stake,
      ) 
    );
  }
  let submit_epochs = pallet_network::MinRequiredSubnetConsensusSubmitEpochs::<Test>::get();
  let starting_block = System::block_number();
  let epoch_length = EpochLength::get();

  System::set_block_number(starting_block + submit_epochs * epoch_length + 1);

  let epoch = System::block_number() / epoch_length;
  let node_set = pallet_network::get_classified_accounts(subnet_id, &SubnetNodeClass::Submittable, epoch);

  assert_eq!(node_set.len(), end as usize - start as usize);


  let last_class_id = pallet_network::SubnetNodeClass::iter().last().unwrap();
  let starting_block = System::block_number();

}

fn build_subnet_nodes(start: u32, end: u32, deposit_amount: u128) -> Vec<SubnetNode<AccountId>> {
  let mut subnet_nodes: Vec<SubnetNode<<Test as frame_system::Config>::AccountId>> = Vec::new();
  
  for n in start..end {
    let _ = Balances::deposit_creating(&account(n), deposit_amount);
    let subnet_node = SubnetNode {
      account_id: account(n),
      peer_id: peer(n),
    };
    subnet_nodes.push(subnet_node);
  }
  subnet_nodes
}

fn post_success_proposal_activate_ensures(
  path: Vec<u8>, 
  proposal_index: u32, 
  proposer: u32, 
  proposal_start_block: u64, 
  subnet_nodes_length: u32
) {
  let proposal = Proposals::<Test>::get(proposal_index);
  assert_eq!(proposal.path, path.clone());
  assert_eq!(proposal.proposal_status, PropsStatus::Active);
  assert_eq!(proposal.proposal_type, PropsType::Activate);
  assert_eq!(proposal.start_vote_block, proposal_start_block + VerifyPeriod::get());
  assert_eq!(proposal.end_vote_block, proposal_start_block + VerifyPeriod::get() + VotingPeriod::get());
  assert_eq!(proposal.subnet_nodes.len() as u32, subnet_nodes_length);

  let active_activate_proposals = ActiveProposalsCount::<Test>::get();
  assert_eq!(active_activate_proposals, proposal_index + 1);

  let proposal_path_status = PropsPathStatus::<Test>::get(path.clone());
  assert_eq!(proposal_path_status, PropsStatus::Active);
}

fn post_cast_vote_ensures(proposal_index: u32, voter: u32) {
  assert_err!(
    SubnetVoting::unreserve(
      RuntimeOrigin::signed(account(voter)),
      proposal_index, 
    ),
    Error::<Test>::ProposalInvalid
  );
}

fn get_subnet_initialization_cost() -> u128 {
  let subnet_initialization_cost = <pallet_network::Pallet<Test> as SubnetVote<OriginFor<Test>, <Test as frame_system::Config>::AccountId>>::get_subnet_initialization_cost();
  subnet_initialization_cost
}

fn post_yay_ensures(proposal_index: u32, prev_votes: u128, voter: u32, vote_amount: u128) {
  let subnet_initialization_cost = get_subnet_initialization_cost();

  let reserved_balance = Balances::reserved_balance(&account(voter));
  let voting_power = SubnetVoting::get_voting_power(account(voter), vote_amount);

  if voter == 0 {
    assert_eq!(reserved_balance, subnet_initialization_cost);
  } else {
    // no reserved balance on votes since it comes from staking now
    // calculations are done in pallet
    // assert_eq!(reserved_balance, vote_amount);
  }

  assert_eq!(VotesBalance::<Test>::get(proposal_index, account(voter)), voting_power);  

  let votes = Votes::<Test>::get(proposal_index);

  assert_eq!(votes.yay, prev_votes + voting_power);
}

fn post_nay_ensures(proposal_index: u32, prev_votes: u128, voter: u32, vote_amount: u128) {
  let subnet_initialization_cost = get_subnet_initialization_cost();
  let reserved_balance = Balances::reserved_balance(&account(voter));
  let voting_power = SubnetVoting::get_voting_power(account(voter), vote_amount);

  if voter == 0 {
    assert_eq!(reserved_balance, subnet_initialization_cost);
  } else {
    // no reserved balance on votes since it comes from staking now
    // calculations are done in pallet
    // assert_eq!(reserved_balance, vote_amount);
  }

  assert_eq!(VotesBalance::<Test>::get(proposal_index, account(voter)), voting_power);

  let votes = Votes::<Test>::get(proposal_index);

  assert_eq!(votes.nay, prev_votes + voting_power);
}

fn post_abstain_ensures(proposal_index: u32, prev_votes: u128, voter: u32, vote_amount: u128) {
  let reserved_balance = Balances::reserved_balance(&account(voter));
  if voter == 0 {
    let subnet_initialization_cost = get_subnet_initialization_cost();
    assert_eq!(reserved_balance, subnet_initialization_cost);  
  } else {
    // no reserved balance on votes since it comes from staking now
    // calculations are done in pallet
    // assert_eq!(reserved_balance, vote_amount);  
  }

  let voting_power = SubnetVoting::get_voting_power(account(voter), vote_amount);
  assert_eq!(VotesBalance::<Test>::get(proposal_index, account(voter)), voting_power);

  let votes = Votes::<Test>::get(proposal_index);

  assert_eq!(votes.abstain, prev_votes + voting_power);
}


fn post_activate_execute_succeeded_ensures(proposal_index: u32, path: Vec<u8>) {
  // let vote_subnet_data = pallet_network::SubnetActivated::<Test>::get(path.clone());
  // assert_eq!(vote_subnet_data.active, true);

  let proposal = Proposals::<Test>::get(proposal_index);
  assert_eq!(proposal.proposal_status, PropsStatus::Succeeded);
  assert_eq!(proposal.proposer_stake, 0);

  let active_activate_proposals = ActiveProposalsCount::<Test>::get();
  assert_eq!(active_activate_proposals, proposal_index);

  post_proposal_concluded(proposal_index, path.clone());

  let proposal_path_status = PropsPathStatus::<Test>::get(path.clone());
  assert_eq!(proposal_path_status, PropsStatus::Succeeded);

  // Check that the subnet has been added to the network pallet
  let subnet_id = pallet_network::SubnetPaths::<Test>::get(path.clone()).unwrap();
  assert!(subnet_id != 0, "Subnet path has no subnet ID");
  
  let subnet_data = pallet_network::SubnetsData::<Test>::get(subnet_id);
  let subnet_path: Vec<u8> = subnet_data.unwrap().path;
  assert_eq!(subnet_path, path);
}

fn post_deactivate_succeeded_execute_ensures(proposal_index: u32, path: Vec<u8>) {
  // let vote_subnet_data = pallet_network::SubnetActivated::<Test>::get(path.clone());
  // assert_eq!(vote_subnet_data.active, false);

  let proposal = Proposals::<Test>::get(proposal_index);
  assert_eq!(proposal.proposal_status, PropsStatus::Succeeded);

  let active_activate_proposals = ActiveProposalsCount::<Test>::get();
  assert_eq!(active_activate_proposals, proposal_index);

  post_proposal_concluded(proposal_index, path.clone());

  let proposal_path_status = PropsPathStatus::<Test>::get(path.clone());
  assert_eq!(proposal_path_status, PropsStatus::Succeeded);

  // Ensure path is removed
  let subnet_id = pallet_network::SubnetPaths::<Test>::get(path.clone());
  assert_eq!(subnet_id, None);
}


fn post_activate_cancel_ensures(proposal_index: u32, path: Vec<u8>) {
  // let vote_subnet_data = pallet_network::SubnetActivated::<Test>::get(path.clone());
  // assert_eq!(vote_subnet_data, None);
  // assert_eq!(vote_subnet_data.active, false);

  let proposal = Proposals::<Test>::get(proposal_index);
  assert_eq!(proposal.proposal_status, PropsStatus::Cancelled);

  let proposal_path_status = PropsPathStatus::<Test>::get(path.clone());
  assert_eq!(proposal.proposal_status, PropsStatus::Cancelled);

  post_proposal_concluded(proposal_index, path.clone());
}

fn post_success_proposal_deactivate_ensures(path: Vec<u8>, proposal_index: u32, proposer: u32, proposal_start_block: u64) {
  let proposal = Proposals::<Test>::get(proposal_index);
  assert_eq!(proposal.path, path.clone());
  assert_eq!(proposal.proposal_status, PropsStatus::Active);
  assert_eq!(proposal.proposal_type, PropsType::Deactivate);
  assert_eq!(proposal.start_vote_block, proposal_start_block + VerifyPeriod::get());
  assert_eq!(proposal.end_vote_block, proposal_start_block + VerifyPeriod::get() + VotingPeriod::get());

  let active_activate_proposals = ActiveProposalsCount::<Test>::get();
  assert_eq!(active_activate_proposals, proposal_index + 1);

  let proposal_path_status = PropsPathStatus::<Test>::get(path.clone());
  assert_eq!(proposal_path_status, PropsStatus::Active);
}

fn post_proposal_concluded(proposal_index: u32, path: Vec<u8>) {
  let active_activate_proposals = ActiveProposalsCount::<Test>::get();
  assert_eq!(active_activate_proposals, proposal_index);

  // --- Ensure cannot call twice
  assert_err!(
    SubnetVoting::execute(
      RuntimeOrigin::signed(account(0)),
      proposal_index,
    ),
    Error::<Test>::Concluded
  );

  // --- Ensure cannot cast vote
  // assert_err!(
  //   SubnetVoting::cast_vote(
  //     RuntimeOrigin::signed(account(255)),
  //     proposal_index,
  //     1000,
  //     VoteType::Yay,
  //   ),
  //   Error::<Test>::VotingNotOpen
  // );

  // --- Ensure proposal not active
  assert_err!(
    SubnetVoting::cast_vote(
      RuntimeOrigin::signed(account(255)),
      proposal_index,
      1000,
      VoteType::Yay,
    ),
    Error::<Test>::ProposalNotActive
  );

  // --- Ensure cannot cancel proposal
  assert_err!(
    SubnetVoting::cancel_proposal(
      RuntimeOrigin::signed(account(0)),
      proposal_index,
    ),
    Error::<Test>::Concluded
  );

  let proposal_path_status = PropsPathStatus::<Test>::get(path.clone());
  assert_ne!(proposal_path_status, PropsStatus::Active);
  assert_ne!(proposal_path_status, PropsStatus::None);
}

fn post_proposal_conclusion_unreserves(proposal_index: u32, start: u32, end: u32, vote_amount: u128) {
  let proposal = Proposals::<Test>::get(proposal_index);
  for n in start..end {
    let beginning_balance = Balances::free_balance(&account(n));
    let votes_balance = VotesBalance::<Test>::get(proposal_index, account(n));

    assert_ok!(
      SubnetVoting::unreserve(
        RuntimeOrigin::signed(account(n)),
        proposal_index, 
      )
    );

    // let balance = Balances::free_balance(&account(n));
    // assert_eq!(balance, beginning_balance + votes_balance);

    let votes_balance = VotesBalance::<Test>::get(proposal_index, account(n));
    assert_eq!(votes_balance, 0);
  }
  // check proposers reserve
  // let proposers_reserve = Balances::reserved_balance(&account(0));
  // assert_eq!(proposers_reserve, 0);
}

/// Activate a proposal and verify subnet nodes
fn build_propose_activate(path: Vec<u8>, start: u32, end: u32, deposit_amount: u128) -> u32 {
  let subnet_nodes = build_subnet_nodes(start, end, deposit_amount);

  let subnet_initialization_cost = get_subnet_initialization_cost();
  let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost+1000);

  assert_ok!(
    SubnetVoting::propose(
      RuntimeOrigin::signed(account(0)),
      default_add_subnet_data(), 
      subnet_nodes,
      PropsType::Activate,
    )
  );
  let proposal_index = PropCount::<Test>::get();

  for n in start..end {
    assert_ok!(
      SubnetVoting::verify_proposal(
        RuntimeOrigin::signed(account(n)),
        proposal_index - 1, 
      )
    );  
  }

  assert_ok!(
    SubnetVoting::activate_proposal(
      RuntimeOrigin::signed(account(0)),
      proposal_index - 1, 
    )
  );

  let active_activate_proposals = ActiveActivateProposals::<Test>::get();
  let this_active_proposal = active_activate_proposals.get(&(proposal_index - 1));
  assert_ne!(this_active_proposal, None);

  return proposal_index - 1
}


/// Uses existing subnet paths
fn build_propose_deactivate(path: Vec<u8>, start: u32, end: u32, deposit_amount: u128) -> u32 {
  let min_subnet_nodes = get_default_min_subnet_nodes();
  build_existing_subnet(0, min_subnet_nodes);

  let subnet_nodes = build_subnet_nodes(start, end, deposit_amount);

  let submit_epochs = pallet_network::MinRequiredSubnetConsensusSubmitEpochs::<Test>::get();
  let epoch_length = EpochLength::get();

  let subnet_path: Vec<u8> = DEFAULT_EXISTING_MODEL_PATH.into();
  let subnet_id = pallet_network::SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
  // pallet_network::SubnetPenaltyCount::<Test>::insert(subnet_id, 1);

  System::set_block_number(System::block_number() + submit_epochs * epoch_length + 1000);

  let proposer_stake = MinProposerStake::get();
  let _ = Balances::deposit_creating(&account(0), proposer_stake);

  assert_ok!(
    SubnetVoting::propose(
      RuntimeOrigin::signed(account(0)),
      default_existing_add_subnet_data(), 
      Vec::new(),
      PropsType::Deactivate,
    )
  );
  0
}

#[test]
fn test_propose_activate() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    let min_stake = pallet_network::MinStakeBalance::<Test>::get();
    let subnet_initialization_cost = get_subnet_initialization_cost();
    
    let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost);

    let subnet_nodes = build_subnet_nodes(0, min_subnet_nodes, min_stake);

    assert_ok!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_add_subnet_data(), 
        subnet_nodes.clone(),
        PropsType::Activate,
      )
    );

    let activate_proposals = ActivateProposalsCount::<Test>::get();

    assert_eq!(activate_proposals, 1);
    post_success_proposal_activate_ensures(default_add_subnet_data().path, prop_count, 0, System::block_number(), subnet_nodes.clone().len() as u32);
  })
}

#[test]
fn test_propose_activate_and_verify() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    let min_stake = pallet_network::MinStakeBalance::<Test>::get();
    let subnet_initialization_cost = get_subnet_initialization_cost();
    let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost);

    let subnet_nodes = build_subnet_nodes(0, min_subnet_nodes, min_stake);

    assert_ok!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_add_subnet_data(), 
        subnet_nodes.clone(),
        PropsType::Activate,
      )
    );

    let activate_proposals = ActivateProposalsCount::<Test>::get();
    assert_eq!(activate_proposals, 1);
    post_success_proposal_activate_ensures(default_add_subnet_data().path, prop_count, 0, System::block_number(), subnet_nodes.clone().len() as u32);
 
    for n in 0..min_subnet_nodes {
      let proposal = Proposals::<Test>::get(0);
      assert_eq!(proposal.subnet_nodes_verified.len() as u32, n);

      assert_ok!(
        SubnetVoting::verify_proposal(
          RuntimeOrigin::signed(account(n)),
          0, 
        )
      );  
    }
  })
}

#[test]
fn test_propose_activate_duplicate_nodes() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();

    let min_subnet_nodes = get_default_min_subnet_nodes();
    let min_stake = pallet_network::MinStakeBalance::<Test>::get();
    let subnet_initialization_cost = get_subnet_initialization_cost();

    let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost);

    let mut subnet_nodes: Vec<SubnetNode<<Test as frame_system::Config>::AccountId>> = Vec::new();
  
    for n in 0..min_subnet_nodes {
      let _ = Balances::deposit_creating(&account(0), min_stake);
      let subnet_node = SubnetNode {
        account_id: account(0),
        peer_id: peer(0),
      };
      subnet_nodes.push(subnet_node);
    }
  
    assert_err!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_add_subnet_data(), 
        subnet_nodes,
        PropsType::Activate,
      ),
      Error::<Test>::SubnetNodesLengthInvalid,
    );
  })
}


#[test]
fn test_propose_activate_subnet_path_exists_err() {
  new_test_ext().execute_with(|| {

    // let min_subnet_nodes = pallet_network::MinSubnetNodes::<Test>::get();

    // let min_subnet_nodes = <pallet_network::Pallet<Test> as SubnetVote<Test>>::get_min_subnet_nodes(default_add_subnet_data().memory_mb);
    // let min_subnet_nodes = <pallet_network::Pallet<Test> as SubnetVote<<Test as frame_system::Config>::AccountId>>::get_min_subnet_nodes(default_add_subnet_data().memory_mb);
    let min_subnet_nodes = get_default_min_subnet_nodes();
    // Create existing subnet
    build_existing_subnet(0, min_subnet_nodes);
    let subnet_data = pallet_network::SubnetsData::<Test>::get(1);
    let subnet_path: Vec<u8> = subnet_data.unwrap().path;

    let min_stake = pallet_network::MinStakeBalance::<Test>::get();
    let subnet_initialization_cost = get_subnet_initialization_cost();

    let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost);

    assert_err!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_existing_add_subnet_data(),
        Vec::new(),
        PropsType::Activate,
      ),
      Error::<Test>::SubnetPathExists
    );
  })
}

#[test]
fn test_propose_activate_already_active_err() {
  new_test_ext().execute_with(|| {
    // let min_subnet_nodes = <pallet_network::Pallet<Test> as SubnetVote<<Test as frame_system::Config>::AccountId>>::get_min_subnet_nodes(default_add_subnet_data().memory_mb);
    let min_subnet_nodes = get_default_min_subnet_nodes();
    let min_stake = pallet_network::MinStakeBalance::<Test>::get();
    let subnet_initialization_cost = get_subnet_initialization_cost();

    let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost);

    let subnet_nodes = build_subnet_nodes(0, min_subnet_nodes, min_stake);

    assert_ok!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_add_subnet_data(), 
        subnet_nodes.clone(),
        PropsType::Activate,
      )
    );

    let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost);
    let subnet_nodes = build_subnet_nodes(0, min_subnet_nodes, min_stake);

    assert_err!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_add_subnet_data(),
        subnet_nodes.clone(),
        PropsType::Activate,
      ),
      Error::<Test>::ProposalInvalid
    );
  })
}

#[test]
fn test_propose_activate_peers_min_length_err() {
  new_test_ext().execute_with(|| {
    let min_subnet_nodes = get_default_min_subnet_nodes();
    let min_stake = pallet_network::MinStakeBalance::<Test>::get();
    let subnet_initialization_cost = get_subnet_initialization_cost();

    let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost);

    assert_err!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_add_subnet_data(),
        Vec::new(),
        PropsType::Activate,
      ),
      Error::<Test>::SubnetNodesLengthInvalid
    );
  })
}

#[test]
fn test_propose_activate_peers_balance_err() {
  new_test_ext().execute_with(|| {
    let min_subnet_nodes = get_default_min_subnet_nodes();
    let min_stake = pallet_network::MinStakeBalance::<Test>::get();
    let subnet_initialization_cost = get_subnet_initialization_cost();

    let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost);

    let subnet_nodes = build_subnet_nodes(0, min_subnet_nodes, min_stake-10000);

    assert_err!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_add_subnet_data(),
        subnet_nodes,
        PropsType::Activate,
      ),
      Error::<Test>::NotEnoughMinStakeBalance
    );
  })
}

#[test]
fn test_propose_activate_subnet_init_balance_err() {
  new_test_ext().execute_with(|| {
    let min_subnet_nodes = get_default_min_subnet_nodes();
    let offset = 1;
    let min_stake = pallet_network::MinStakeBalance::<Test>::get();

    // let min_subnet_nodes = <pallet_network::Pallet<Test> as SubnetVote<Test>>::get_min_subnet_nodes(default_add_subnet_data().memory_mb);
    // add subnet to ensure initialization cost is over zero
    build_existing_subnet(offset, min_subnet_nodes + offset);



    // let min_subnet_nodes = <pallet_network::Pallet<Test> as SubnetVote<Test>>::get_min_subnet_nodes(default_add_subnet_data().memory_mb);
    // let offset = 1;
    // let min_stake = pallet_network::MinStakeBalance::<Test>::get();

    // // add subnet to ensure initialization cost is over zero
    // build_existing_subnet(offset, min_subnet_nodes + offset);

    // make_subnet_node_included();

    let _ = Balances::deposit_creating(&account(0), 0);

    let subnet_nodes = build_subnet_nodes(offset, min_subnet_nodes + offset, min_stake);

    assert_err!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_add_subnet_data(), 
        subnet_nodes,
        PropsType::Activate,
      ),
      Error::<Test>::NotEnoughSubnetInitializationBalance
    );
  })
}

#[test]
fn test_cast_vote_activate_yay() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);

    let active_activate_proposals = ActiveActivateProposals::<Test>::get();
    assert_eq!(active_activate_proposals.len(), 1);
  
    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);

    let _ = Balances::deposit_creating(&account(0), DEFAUT_VOTE_AMOUNT);

    let votes = Votes::<Test>::get(prop_count);

    assert_ok!(
      SubnetVoting::cast_vote(
        RuntimeOrigin::signed(account(0)),
        prop_count,
        DEFAUT_VOTE_AMOUNT,
        VoteType::Yay,
      )
    );

    post_yay_ensures(prop_count, votes.yay, 0, DEFAUT_VOTE_AMOUNT);
    post_cast_vote_ensures(prop_count, 0);
  })
}

#[test]
fn test_cast_vote_activate_yay_props_exists_err() {
  new_test_ext().execute_with(|| {
    assert_err!(
      SubnetVoting::cast_vote(
        RuntimeOrigin::signed(account(0)),
        0,
        DEFAUT_VOTE_AMOUNT,
        VoteType::Yay,
      ),
      Error::<Test>::ProposalInvalid
    );
  })
}

#[test]
fn test_cast_vote_activate_yay_voting_not_open_err() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    let active_activate_proposals = ActiveActivateProposals::<Test>::get();
    assert_eq!(active_activate_proposals.len(), 1);

    let verifying_period = VerifyPeriod::get();
    let voting_period = VotingPeriod::get();
    
    let _ = Balances::deposit_creating(&account(0), DEFAUT_VOTE_AMOUNT);

    System::set_block_number(System::block_number() + voting_period + verifying_period + 1);

    assert_err!(
      SubnetVoting::cast_vote(
        RuntimeOrigin::signed(account(0)),
        proposal_index,
        DEFAUT_VOTE_AMOUNT,
        VoteType::Yay,
      ),
      Error::<Test>::VotingNotOpen
    );

  })
}

#[test]
fn test_cast_vote_activate_yay_not_enough_balance_err() {
  new_test_ext().execute_with(|| {
    let offset = 1;
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0+offset, min_subnet_nodes+offset, DEFAULT_DEPOSIT_AMOUNT);
    let active_activate_proposals = ActiveActivateProposals::<Test>::get();
    assert_eq!(active_activate_proposals.len(), 1);

    let _ = Balances::deposit_creating(&account(255), 100);

    assert_err!(
      SubnetVoting::cast_vote(
        RuntimeOrigin::signed(account(255)),
        proposal_index,
        DEFAUT_VOTE_AMOUNT,
        VoteType::Yay,
      ),
      Error::<Test>::NotEnoughBalanceToVote
    );
  })
}

#[test]
fn test_cast_vote_activate_nay() {
  new_test_ext().execute_with(|| {
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();

    build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    let active_activate_proposals = ActiveActivateProposals::<Test>::get();
    assert_eq!(active_activate_proposals.len(), 1);

    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);

    let _ = Balances::deposit_creating(&account(0), DEFAUT_VOTE_AMOUNT);

    let votes = Votes::<Test>::get(prop_count);

    assert_ok!(
      SubnetVoting::cast_vote(
        RuntimeOrigin::signed(account(0)),
        prop_count,
        DEFAUT_VOTE_AMOUNT,
        VoteType::Nay,
      )
    );

    post_nay_ensures(prop_count, votes.nay, 0, DEFAUT_VOTE_AMOUNT);
    post_cast_vote_ensures(prop_count, 0);
  })
}

#[test]
fn test_cast_vote_activate_nay_props_exists_err() {
  new_test_ext().execute_with(|| {
    assert_err!(
      SubnetVoting::cast_vote(
        RuntimeOrigin::signed(account(0)),
        0,
        DEFAUT_VOTE_AMOUNT,
        VoteType::Nay,
      ),
      Error::<Test>::ProposalInvalid
    );
  })
}

#[test]
fn test_cast_vote_activate_nay_voting_not_open_err() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    let active_activate_proposals = ActiveActivateProposals::<Test>::get();
    assert_eq!(active_activate_proposals.len(), 1);

    let _ = Balances::deposit_creating(&account(0), DEFAUT_VOTE_AMOUNT);

    let verifying_period = VerifyPeriod::get();
    let voting_period = VotingPeriod::get();
    System::set_block_number(System::block_number() + verifying_period + voting_period + 1);

    assert_err!(
      SubnetVoting::cast_vote(
        RuntimeOrigin::signed(account(0)),
        proposal_index,
        DEFAUT_VOTE_AMOUNT,
        VoteType::Nay,
      ),
      Error::<Test>::VotingNotOpen
    );

  })
}

#[test]
fn test_cast_vote_activate_nay_not_enough_balance_err() {
  new_test_ext().execute_with(|| {
    let offset = 1;
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0+offset, min_subnet_nodes+offset, DEFAULT_DEPOSIT_AMOUNT);
    let active_activate_proposals = ActiveActivateProposals::<Test>::get();
    assert_eq!(active_activate_proposals.len(), 1);

    let _ = Balances::deposit_creating(&account(255), 100);

    assert_err!(
      SubnetVoting::cast_vote(
        RuntimeOrigin::signed(account(255)),
        proposal_index,
        DEFAUT_VOTE_AMOUNT,
        VoteType::Nay,
      ),
      Error::<Test>::NotEnoughBalanceToVote
    );
  })
}

#[test]
fn test_cast_vote_activate_abstain() {
  new_test_ext().execute_with(|| {
    // build subnet so accounst have voting balance
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();

    build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    let active_activate_proposals = ActiveActivateProposals::<Test>::get();
    assert_eq!(active_activate_proposals.len(), 1);

    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);
    let _ = Balances::deposit_creating(&account(0), DEFAUT_VOTE_AMOUNT);

    let votes = Votes::<Test>::get(prop_count);

    assert_ok!(
      SubnetVoting::cast_vote(
        RuntimeOrigin::signed(account(0)),
        prop_count,
        DEFAUT_VOTE_AMOUNT,
        VoteType::Abstain,
      )
    );

    post_abstain_ensures(prop_count, votes.abstain, 0, DEFAUT_VOTE_AMOUNT);
    post_cast_vote_ensures(prop_count, 0);
  })
}

#[test]
fn test_cast_vote_activate_abstain_props_exists_err() {
  new_test_ext().execute_with(|| {
    assert_err!(
      SubnetVoting::cast_vote(
        RuntimeOrigin::signed(account(0)),
        0,
        DEFAUT_VOTE_AMOUNT,
        VoteType::Abstain,
      ),
      Error::<Test>::ProposalInvalid
    );
  })
}

#[test]
fn test_cast_vote_activate_abstain_voting_not_open_err() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    let active_activate_proposals = ActiveActivateProposals::<Test>::get();
    assert_eq!(active_activate_proposals.len(), 1);

    let _ = Balances::deposit_creating(&account(0), DEFAUT_VOTE_AMOUNT);

    let verifying_period = VerifyPeriod::get();
    let voting_period = VotingPeriod::get();
    System::set_block_number(System::block_number() + verifying_period + voting_period + 1);

    assert_err!(
      SubnetVoting::cast_vote(
        RuntimeOrigin::signed(account(0)),
        proposal_index,
        DEFAUT_VOTE_AMOUNT,
        VoteType::Abstain,
      ),
      Error::<Test>::VotingNotOpen
    );
  })
}

#[test]
fn test_cast_vote_activate_abstain_not_enough_balance_err() {
  new_test_ext().execute_with(|| {
    let offset = 1;
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0+offset, min_subnet_nodes+offset, DEFAULT_DEPOSIT_AMOUNT);
    let active_activate_proposals = ActiveActivateProposals::<Test>::get();
    assert_eq!(active_activate_proposals.len(), 1);

    let _ = Balances::deposit_creating(&account(255), 100);

    assert_err!(
      SubnetVoting::cast_vote(
        RuntimeOrigin::signed(account(256)),
        proposal_index,
        DEFAUT_VOTE_AMOUNT,
        VoteType::Abstain,
      ),
      Error::<Test>::NotEnoughBalanceToVote
    );
  })
}

#[test]
fn test_execute_activate_succeeded() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    let active_activate_proposals = ActiveActivateProposals::<Test>::get();
    assert_eq!(active_activate_proposals.len(), 1);

    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);

    for n in 0..min_subnet_nodes {
      let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
  
      assert_ok!(
        SubnetVoting::cast_vote(
          RuntimeOrigin::signed(account(n)),
          proposal_index,
          DEFAUT_VOTE_AMOUNT,
          VoteType::Yay,
        )
      );
      post_cast_vote_ensures(proposal_index, n);
    }

    let verifying_period = VerifyPeriod::get();
    let voting_period = VotingPeriod::get();
    System::set_block_number(System::block_number() + verifying_period + voting_period + 1);

    assert_ok!(
      SubnetVoting::execute(
        RuntimeOrigin::signed(account(0)),
        proposal_index,
      )
    );

    post_activate_execute_succeeded_ensures(proposal_index, DEFAULT_MODEL_PATH.into());

    post_proposal_conclusion_unreserves(proposal_index, 0, min_subnet_nodes, DEFAUT_VOTE_AMOUNT);
  })
}

#[test]
fn test_execute_activate_succeeded_reexecute() {
  new_test_ext().execute_with(|| {
    // Should allow max activate proposals after execute()
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);
    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);

    for n in 1..2 {
      let prop_count = PropCount::<Test>::get();

      let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
      let active_activate_proposals = ActiveActivateProposals::<Test>::get();
      assert_eq!(active_activate_proposals.len(), n);
  
      let activate_proposals = ActivateProposalsCount::<Test>::get();
      assert_eq!(activate_proposals, 1);
  
      for n in 0..min_subnet_nodes {
        let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
    
        assert_ok!(
          SubnetVoting::cast_vote(
            RuntimeOrigin::signed(account(n)),
            proposal_index,
            DEFAUT_VOTE_AMOUNT,
            VoteType::Yay,
          )
        );
        post_cast_vote_ensures(proposal_index, n);
      }
  
      let verifying_period = VerifyPeriod::get();
      let voting_period = VotingPeriod::get();
      System::set_block_number(System::block_number() + verifying_period + voting_period + 1);
    
      assert_ok!(
        SubnetVoting::execute(
          RuntimeOrigin::signed(account(0)),
          proposal_index,
        )
      );
  
      post_activate_execute_succeeded_ensures(proposal_index, DEFAULT_MODEL_PATH.into());
  
      post_proposal_conclusion_unreserves(proposal_index, 0, min_subnet_nodes, DEFAUT_VOTE_AMOUNT);  
    }
  })
}

#[test]
fn test_execute_activate_succeeded_reexecute_expired_enactment() {
  new_test_ext().execute_with(|| {
    // Should allow max activate proposals after execute()
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    for p in 0..2 {
      let prop_count = PropCount::<Test>::get();

      let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
      let active_activate_proposals = ActiveActivateProposals::<Test>::get();
      assert_eq!(active_activate_proposals.len(), p+1);
      
      System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);
  
      for n in 0..min_subnet_nodes {
        let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
    
        assert_ok!(
          SubnetVoting::cast_vote(
            RuntimeOrigin::signed(account(n)),
            proposal_index,
            DEFAUT_VOTE_AMOUNT,
            VoteType::Yay,
          )
        );
        post_cast_vote_ensures(proposal_index, n);
      }
  
      let voting_period = VotingPeriod::get();
  
      let enactment_period = EnactmentPeriod::get();

      System::set_block_number(System::block_number() + voting_period + enactment_period + 1);

      assert_ok!(
        SubnetVoting::execute(
          RuntimeOrigin::signed(account(0)),
          proposal_index,
        )
      );

      let activate_proposals = ActivateProposalsCount::<Test>::get();
      assert_eq!(activate_proposals, 0);
  
      let proposal = Proposals::<Test>::get(proposal_index);
      assert_eq!(proposal.proposal_type, PropsType::Activate);

      let path: Vec<u8> = DEFAULT_MODEL_PATH.into();

      let proposal_path_status = PropsPathStatus::<Test>::get(path.clone());
      assert_eq!(proposal_path_status, PropsStatus::Expired);
    
      assert_eq!(proposal.proposal_status, PropsStatus::Expired);
  
      // let is_active = pallet_network::SubnetActivated::<Test>::get(path);
      // assert_eq!(is_active.active, false);
    }
  })
}

#[test]
fn test_execute_voting_period_err() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);

    for n in 0..min_subnet_nodes {
      let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
  
      assert_ok!(
        SubnetVoting::cast_vote(
          RuntimeOrigin::signed(account(n)),
          proposal_index,
          DEFAUT_VOTE_AMOUNT,
          VoteType::Yay,
        )
      );
      post_cast_vote_ensures(proposal_index, n);
    }

    assert_err!(
      SubnetVoting::execute(
        RuntimeOrigin::signed(account(0)),
        proposal_index,
      ),
      Error::<Test>::VotingOpen,
    );
  })
}

#[test]
fn test_execute_enactment_period_err() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);

    for n in 0..min_subnet_nodes {
      let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
  
      assert_ok!(
        SubnetVoting::cast_vote(
          RuntimeOrigin::signed(account(n)),
          proposal_index,
          DEFAUT_VOTE_AMOUNT,
          VoteType::Yay,
        )
      );
      post_cast_vote_ensures(proposal_index, n);
    }

    let voting_period = VotingPeriod::get();
    let enactment_period = EnactmentPeriod::get();

    System::set_block_number(System::block_number() + voting_period + enactment_period + 1);

    assert_ok!(
      SubnetVoting::execute(
        RuntimeOrigin::signed(account(0)),
        proposal_index,
      )
      // Error::<Test>::EnactmentPeriodPassed,
    );
  })
}

#[test]
fn test_execute_quorum_not_reached_err() {
  new_test_ext().execute_with(|| {
    // let quorum = Quorum::<Test>::get();
    let quorum = Quorum::get();
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);

    
    for n in 0..min_subnet_nodes {
      let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
  
      assert_ok!(
        SubnetVoting::cast_vote(
          RuntimeOrigin::signed(account(n)),
          proposal_index,
          1, // too low to reach minimum quorum
          VoteType::Yay,
        )
      );
      post_cast_vote_ensures(proposal_index, n);
    }

    let proposal = Proposals::<Test>::get(proposal_index);

    let voting_period = VotingPeriod::get();
    let enactment_period = EnactmentPeriod::get();

    System::set_block_number(proposal.end_vote_block + 1);
    // System::set_block_number(proposal.end_vote_block + enactment_period + 1);

    assert_ok!(
      SubnetVoting::execute(
        RuntimeOrigin::signed(account(0)),
        proposal_index,
      )
    );

    let proposal = Proposals::<Test>::get(proposal_index);

    let path: Vec<u8> = DEFAULT_MODEL_PATH.into();

    let proposal_path_status = PropsPathStatus::<Test>::get(path.clone());
    assert_eq!(proposal_path_status, PropsStatus::Expired);
  
    assert_eq!(proposal.proposal_status, PropsStatus::Expired);

    // let is_active = pallet_network::SubnetActivated::<Test>::get(path);
    // assert_eq!(is_active.active, false);


    post_proposal_conclusion_unreserves(proposal_index, 0, min_subnet_nodes, DEFAUT_VOTE_AMOUNT);
  })
}

#[test]
fn test_execute_defeated() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    // let min_subnet_nodes = get_default_min_subnet_nodes();
    let min_subnet_nodes = 12;
    build_existing_subnet(0, min_subnet_nodes);

    // if min_subnet_nodes < 12 {
    //   min_subnet_nodes = 12
    // }

    // Get more nay voters than yay voters
    let yay_voters = min_subnet_nodes / 4;

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);

    let vote_amount: u128 = 500e+18 as u128;

    // let quorum = Quorum::<Test>::get();
    let quorum = Quorum::get();

    let mut total_vote_amount = 0;

    for n in 0..yay_voters {
      let _ = Balances::deposit_creating(&account(n), vote_amount);
      total_vote_amount += vote_amount;
      assert_ok!(
        SubnetVoting::cast_vote(
          RuntimeOrigin::signed(account(n)),
          proposal_index,
          vote_amount,
          VoteType::Yay,
        )
      );
      post_cast_vote_ensures(proposal_index, n);
    }

    for n in yay_voters..min_subnet_nodes {
      let _ = Balances::deposit_creating(&account(n), vote_amount);
      assert_ok!(
        SubnetVoting::cast_vote(
          RuntimeOrigin::signed(account(n)),
          proposal_index,
          vote_amount,
          VoteType::Nay,
        )
      );
      post_cast_vote_ensures(proposal_index, n);
    }

    log::error!("total_vote_amount {:?}", total_vote_amount);
    log::error!("quorum            {:?}", quorum);
    assert!(total_vote_amount >= quorum, "Total votes must be greater than quorum");

    let verifying_period = VerifyPeriod::get();
    let voting_period = VotingPeriod::get();
    System::set_block_number(System::block_number() + verifying_period + voting_period + 1);

    assert_ok!(
      SubnetVoting::execute(
        RuntimeOrigin::signed(account(0)),
        proposal_index,
      )
    );

    let proposal = Proposals::<Test>::get(proposal_index);
    assert_eq!(proposal.proposal_status, PropsStatus::Defeated);

    post_proposal_conclusion_unreserves(proposal_index, 0, min_subnet_nodes, vote_amount);
  })
}

#[test]
fn test_execute_cancelled() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    System::set_block_number(System::block_number() + 1);
    let proposal = Proposals::<Test>::get(proposal_index);
    let proposer_stake = proposal.proposer_stake;
    // let proposer_beginning_balance = Balances::free_balance(&account(0));
    // let proposer_assumed_balance = proposer_beginning_balance - DEFAUT_VOTE_AMOUNT;

    let subnet_initialization_cost = get_subnet_initialization_cost();

    assert_ne!(subnet_initialization_cost, 0);

    for n in 0..min_subnet_nodes {
      let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
  
      assert_ok!(
        SubnetVoting::cast_vote(
          RuntimeOrigin::signed(account(n)),
          proposal_index,
          DEFAUT_VOTE_AMOUNT,
          VoteType::Yay,
        )
      );
      post_cast_vote_ensures(proposal_index, n);
    }

    let proposer_pre_cancel_balance = Balances::free_balance(&account(0));

    assert_ok!(
      SubnetVoting::cancel_proposal(
        RuntimeOrigin::signed(account(0)),
        proposal_index,
      )
    );

    let proposer_stake_unreserve = proposer_stake - Percent::from_percent(CancelSlashPercent::get()) * proposer_stake;

    let proposer_after_balance = Balances::free_balance(&account(0));
    // assert_eq!(proposer_after_balance, proposer_beginning_balance + subnet_initialization_cost + DEFAUT_VOTE_AMOUNT);
    assert_eq!(proposer_after_balance, proposer_pre_cancel_balance + proposer_stake_unreserve);

    post_activate_cancel_ensures(prop_count, DEFAULT_MODEL_PATH.into());
    // post_proposal_conclusion_unreserves(proposal_index, 0, min_subnet_nodes, DEFAUT_VOTE_AMOUNT);
  })
}

#[test]
fn test_execute_cancelled_not_proposer_err() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    System::set_block_number(System::block_number() + 1);

    for n in 0..min_subnet_nodes {
      let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
  
      assert_ok!(
        SubnetVoting::cast_vote(
          RuntimeOrigin::signed(account(n)),
          proposal_index,
          DEFAUT_VOTE_AMOUNT,
          VoteType::Yay,
        )
      );
      post_cast_vote_ensures(proposal_index, n);
    }

    assert_err!(
      SubnetVoting::cancel_proposal(
        RuntimeOrigin::signed(account(1)),
        proposal_index,
      ),
      Error::<Test>::NotProposer
    );
  })
}

#[test]
fn test_execute_cancelled_verified_err() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);

    assert_err!(
      SubnetVoting::cancel_proposal(
        RuntimeOrigin::signed(account(1)),
        proposal_index,
      ),
      Error::<Test>::ProposalVerified
    );
  })
}

#[test]
fn test_execute_cancelled_proposal_index_err() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);

    for n in 0..min_subnet_nodes {
      let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
  
      assert_ok!(
        SubnetVoting::cast_vote(
          RuntimeOrigin::signed(account(n)),
          proposal_index,
          DEFAUT_VOTE_AMOUNT,
          VoteType::Yay,
        )
      );
      post_cast_vote_ensures(proposal_index, n);
    }

    assert_err!(
      SubnetVoting::cancel_proposal(
        RuntimeOrigin::signed(account(1)),
        proposal_index + 1,
      ),
      Error::<Test>::ProposalInvalid
    );
  })
}

#[test]
fn test_execute_cancelled_vote_completed_err() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);

    for n in 0..min_subnet_nodes {
      let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
  
      assert_ok!(
        SubnetVoting::cast_vote(
          RuntimeOrigin::signed(account(n)),
          proposal_index,
          DEFAUT_VOTE_AMOUNT,
          VoteType::Yay,
        )
      );
      post_cast_vote_ensures(proposal_index, n);
    }

    let verifying_period = VerifyPeriod::get();
    let voting_period = VotingPeriod::get();
    System::set_block_number(System::block_number() + verifying_period + voting_period + 1);

    assert_err!(
      SubnetVoting::cancel_proposal(
        RuntimeOrigin::signed(account(0)),
        proposal_index,
      ),
      Error::<Test>::VoteComplete
    );
  })
}

#[test]
fn test_propose_activate_not_verified_err() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    let min_stake = pallet_network::MinStakeBalance::<Test>::get();
    let subnet_initialization_cost = get_subnet_initialization_cost();

    let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost);

    let subnet_nodes = build_subnet_nodes(0, min_subnet_nodes, min_stake);

    assert_ok!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_add_subnet_data(), 
        subnet_nodes.clone(),
        PropsType::Activate,
      )
    );

    let activate_proposals = ActivateProposalsCount::<Test>::get();
    assert_eq!(activate_proposals, 1);
    post_success_proposal_activate_ensures(default_add_subnet_data().path, prop_count, 0, System::block_number(), subnet_nodes.clone().len() as u32);
    
    let proposal_index = PropCount::<Test>::get();

    // assert_err!(
    //   SubnetVoting::cast_vote(
    //     RuntimeOrigin::signed(account(0)),
    //     proposal_index - 1,
    //     DEFAUT_VOTE_AMOUNT,
    //     VoteType::Yay,
    //   ),
    //   Error::<Test>::ProposalNotVerified
    // );
    assert_err!(
      SubnetVoting::cast_vote(
        RuntimeOrigin::signed(account(0)),
        proposal_index - 1,
        DEFAUT_VOTE_AMOUNT,
        VoteType::Yay,
      ),
      Error::<Test>::ProposalNotActive
    );

  })
}

#[test]
fn test_propose_deactivate() {
  new_test_ext().execute_with(|| {
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);
    let prop_count = PropCount::<Test>::get();

    let submit_epochs = pallet_network::MinRequiredSubnetConsensusSubmitEpochs::<Test>::get();
    let epoch_length = EpochLength::get();

    let subnet_path: Vec<u8> = DEFAULT_EXISTING_MODEL_PATH.into();
    let subnet_id = pallet_network::SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    pallet_network::SubnetPenaltyCount::<Test>::insert(subnet_id, 1);
  
    System::set_block_number(System::block_number() + submit_epochs * epoch_length + 1000);

    let proposer_stake = MinProposerStake::get();
    let _ = Balances::deposit_creating(&account(0), proposer_stake);
  
    assert_ok!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_existing_add_subnet_data(), 
        Vec::new(),
        PropsType::Deactivate,
      )
    );

    post_success_proposal_deactivate_ensures(DEFAULT_EXISTING_MODEL_PATH.into(), prop_count, 0, System::block_number());
  })
}

#[test]
fn test_propose_deactivate_peers_min_length_err() {
  new_test_ext().execute_with(|| {
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);
    let min_stake = pallet_network::MinStakeBalance::<Test>::get();
    let subnet_initialization_cost = get_subnet_initialization_cost();

    let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost);

    let subnet_nodes = build_subnet_nodes(0, min_subnet_nodes, min_stake);

    let submit_epochs = pallet_network::MinRequiredSubnetConsensusSubmitEpochs::<Test>::get();
    let epoch_length = EpochLength::get();

    System::set_block_number(System::block_number() + submit_epochs * epoch_length + 1000);

    let proposer_stake = MinProposerStake::get();
    let _ = Balances::deposit_creating(&account(0), proposer_stake);
  
    assert_err!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_existing_add_subnet_data(), 
        subnet_nodes,
        PropsType::Deactivate,
      ),
      Error::<Test>::SubnetNodesLengthInvalid
    );
  })
}

#[test]
fn test_propose_deactivate_subnet_id_exist_err() {
  new_test_ext().execute_with(|| {
    let min_subnet_nodes = get_default_min_subnet_nodes();
    let proposer_stake = MinProposerStake::get();
    let _ = Balances::deposit_creating(&account(0), proposer_stake);

    assert_err!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_existing_add_subnet_data(), 
        Vec::new(),
        PropsType::Deactivate,
      ),
      Error::<Test>::SubnetIdNotExists
    );
  })
}

#[test]
fn test_propose_deactivate_already_active_err() {
  new_test_ext().execute_with(|| {
    let min_subnet_nodes = get_default_existing_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);

    let submit_epochs = pallet_network::MinRequiredSubnetConsensusSubmitEpochs::<Test>::get();
    let epoch_length = EpochLength::get();

    let subnet_path: Vec<u8> = DEFAULT_EXISTING_MODEL_PATH.into();
    let subnet_id = pallet_network::SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    pallet_network::SubnetPenaltyCount::<Test>::insert(subnet_id, 1);
  
    System::set_block_number(System::block_number() + submit_epochs * epoch_length + 1000);

    let proposer_stake = MinProposerStake::get();
    let _ = Balances::deposit_creating(&account(0), proposer_stake + 1000);
  
    assert_ok!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_existing_add_subnet_data(), 
        Vec::new(),
        PropsType::Deactivate,
      )
    );

    let proposer_stake = MinProposerStake::get();
    let _ = Balances::deposit_creating(&account(0), proposer_stake + 1000);

    assert_err!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_existing_add_subnet_data(), 
        Vec::new(),
        PropsType::Deactivate,
      ),
      Error::<Test>::ProposalInvalid
    );
  })
}

#[test]
fn test_execute_deactivate_succeeded() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_existing_min_subnet_nodes();

    let proposal_index = build_propose_deactivate(DEFAULT_EXISTING_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);

    assert_eq!(DeactivateProposalsCount::<Test>::get(), 1);

    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);

    for n in 0..min_subnet_nodes {
      let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
  
      assert_ok!(
        SubnetVoting::cast_vote(
          RuntimeOrigin::signed(account(n)),
          proposal_index,
          DEFAUT_VOTE_AMOUNT,
          VoteType::Yay,
        )
      );
      post_cast_vote_ensures(proposal_index, n);
    }

    let verifying_period = VerifyPeriod::get();
    let voting_period = VotingPeriod::get();
    System::set_block_number(System::block_number() + verifying_period + voting_period + 1);

    assert_ok!(
      SubnetVoting::execute(
        RuntimeOrigin::signed(account(0)),
        proposal_index,
      )
    );

    post_deactivate_succeeded_execute_ensures(proposal_index, DEFAULT_EXISTING_MODEL_PATH.into());

    post_proposal_conclusion_unreserves(proposal_index, 0, min_subnet_nodes, DEFAUT_VOTE_AMOUNT);
  })
}

#[test]
fn test_propose_activate_expired() {
  new_test_ext().execute_with(|| {
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);
    let prop_count = PropCount::<Test>::get();

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);

    let verifying_period = VerifyPeriod::get();
    let voting_period = VotingPeriod::get();
    System::set_block_number(System::block_number() + verifying_period + voting_period + 1);

    assert_ok!(
      SubnetVoting::execute(
        RuntimeOrigin::signed(account(0)),
        proposal_index,
      )
    );

    let proposal = Proposals::<Test>::get(proposal_index);
    assert_eq!(proposal.proposal_status, PropsStatus::Expired);


    let proposal = Proposals::<Test>::get(proposal_index);
    let path: Vec<u8> = DEFAULT_MODEL_PATH.into();

    let proposal_path_status = PropsPathStatus::<Test>::get(path.clone());
    assert_eq!(proposal_path_status, PropsStatus::Expired);
  
    assert_eq!(proposal.proposal_status, PropsStatus::Expired);

    // let is_active = pallet_network::SubnetActivated::<Test>::get(path);
    // assert_eq!(is_active, None);
    // assert_eq!(is_active.active, false);

  })
}

#[test]
fn test_balance_on_multiple_votes() {
  new_test_ext().execute_with(|| {
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);
    let prop_count = PropCount::<Test>::get();

    let proposal_index = build_propose_activate(DEFAULT_MODEL_PATH.into(), 0, min_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT);
    System::set_block_number(System::block_number() + VerifyPeriod::get() + 1);

    for n in 0..min_subnet_nodes {
      let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
  
      assert_ok!(
        SubnetVoting::cast_vote(
          RuntimeOrigin::signed(account(n)),
          proposal_index,
          DEFAUT_VOTE_AMOUNT,
          VoteType::Yay,
        )
      );
    }

    let subnet_initialization_cost = get_subnet_initialization_cost();


    for n in 0..min_subnet_nodes {
      let votes_balance = VotesBalance::<Test>::get(proposal_index, account(n));
      let reserve_balance: BalanceOf<Test> = <pallet_balances::Pallet<Test> as ReservableCurrency<AccountId>>::reserved_balance(&account(n));
      if n == 0 {
        assert_eq!(votes_balance, DEFAUT_VOTE_AMOUNT);
        assert_eq!(reserve_balance, subnet_initialization_cost);
      } else {
        assert_eq!(votes_balance, DEFAUT_VOTE_AMOUNT);
        // assert_eq!(reserve_balance, DEFAUT_VOTE_AMOUNT);
      }
    }

    for n in 0..min_subnet_nodes {
      let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
  
      // assert_ok!(
      //   SubnetVoting::cast_vote(
      //     RuntimeOrigin::signed(account(n)),
      //     proposal_index,
      //     DEFAUT_VOTE_AMOUNT,
      //     VoteType::Nay,
      //   )
      // );
      assert_err!(
        SubnetVoting::cast_vote(
          RuntimeOrigin::signed(account(n)),
          proposal_index,
          DEFAUT_VOTE_AMOUNT,
          VoteType::Nay,
        ),
        Error::<Test>::NotEnoughBalanceToVote
      );
    }

    // for n in 0..min_subnet_nodes {
    //   let votes_balance = VotesBalance::<Test>::get(proposal_index, account(n));
    //   let reserve_balance: BalanceOf<Test> = <pallet_balances::Pallet<Test> as ReservableCurrency<AccountId>>::reserved_balance(&account(n));
    //   if n == 0 {
    //     assert_eq!(votes_balance, DEFAUT_VOTE_AMOUNT*2);
    //     assert_eq!(reserve_balance, subnet_initialization_cost);
    //   } else {
    //     assert_eq!(votes_balance, DEFAUT_VOTE_AMOUNT*2);
    //     // assert_eq!(reserve_balance, DEFAUT_VOTE_AMOUNT*2);
    //   }
    // }

    // for n in 0..min_subnet_nodes {
    //   let _ = Balances::deposit_creating(&account(n), DEFAUT_VOTE_AMOUNT);
  
    //   assert_ok!(
    //     SubnetVoting::cast_vote(
    //       RuntimeOrigin::signed(account(n)),
    //       proposal_index,
    //       DEFAUT_VOTE_AMOUNT,
    //       VoteType::Abstain
    //     )
    //   );
    // }

    // for n in 0..min_subnet_nodes {
    //   let votes_balance = VotesBalance::<Test>::get(proposal_index, account(n));
    //   let reserve_balance: BalanceOf<Test> = <pallet_balances::Pallet<Test> as ReservableCurrency<AccountId>>::reserved_balance(&account(n));
    //   if n == 0 {
    //     assert_eq!(votes_balance, DEFAUT_VOTE_AMOUNT*3);
    //     assert_eq!(reserve_balance, subnet_initialization_cost);
    //   } else {
    //     assert_eq!(votes_balance, DEFAUT_VOTE_AMOUNT*3);
    //     // assert_eq!(reserve_balance, DEFAUT_VOTE_AMOUNT*3);
    //   }
    // }
  })
}

#[test]
fn test_propose_activate_verify_period_passed_err() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    let min_stake = pallet_network::MinStakeBalance::<Test>::get();
    let subnet_initialization_cost = get_subnet_initialization_cost();

    let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost);

    let subnet_nodes = build_subnet_nodes(0, min_subnet_nodes, min_stake);

    assert_ok!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_add_subnet_data(), 
        subnet_nodes.clone(),
        PropsType::Activate,
      )
    );

    let activate_proposals = ActivateProposalsCount::<Test>::get();
    assert_eq!(activate_proposals, 1);
    post_success_proposal_activate_ensures(default_add_subnet_data().path, prop_count, 0, System::block_number(), subnet_nodes.clone().len() as u32);
  
    let proposal_index = PropCount::<Test>::get();

    let verifying_period = VerifyPeriod::get();
    let voting_period = VotingPeriod::get();
    System::set_block_number(System::block_number() + verifying_period + voting_period + 1);

    assert_err!(
      SubnetVoting::verify_proposal(
        RuntimeOrigin::signed(account(0)),
        proposal_index - 1, 
      ),
      Error::<Test>::VerifyPeriodPassed
    );  

  })
}

#[test]
fn test_propose_deactivate_verify_proposal_is_deactivate_err() {
  new_test_ext().execute_with(|| {
    let min_subnet_nodes = get_default_min_subnet_nodes();
    build_existing_subnet(0, min_subnet_nodes);
    let prop_count = PropCount::<Test>::get();

    let submit_epochs = pallet_network::MinRequiredSubnetConsensusSubmitEpochs::<Test>::get();
    let epoch_length = EpochLength::get();

    let subnet_path: Vec<u8> = DEFAULT_EXISTING_MODEL_PATH.into();
    let subnet_id = pallet_network::SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    pallet_network::SubnetPenaltyCount::<Test>::insert(subnet_id, 1);
  
    System::set_block_number(System::block_number() + submit_epochs * epoch_length + 1000);

    let proposer_stake = MinProposerStake::get();
    let _ = Balances::deposit_creating(&account(0), proposer_stake);
  
    assert_ok!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_existing_add_subnet_data(), 
        Vec::new(),
        PropsType::Deactivate,
      )
    );
  
    let proposal_index = PropCount::<Test>::get();
    
    assert_err!(
      SubnetVoting::verify_proposal(
        RuntimeOrigin::signed(account(0)),
        proposal_index - 1, 
      ),
      Error::<Test>::ProposalIsDeactvation
    );  

  })
}

#[test]
fn test_propose_deactivate_verify_not_verify_subnet_node_err() {
  new_test_ext().execute_with(|| {
    let prop_count = PropCount::<Test>::get();
    let min_subnet_nodes = get_default_min_subnet_nodes();
    let min_stake = pallet_network::MinStakeBalance::<Test>::get();
    let subnet_initialization_cost = get_subnet_initialization_cost();

    let _ = Balances::deposit_creating(&account(0), subnet_initialization_cost);

    let subnet_nodes = build_subnet_nodes(0, min_subnet_nodes, min_stake);

    assert_ok!(
      SubnetVoting::propose(
        RuntimeOrigin::signed(account(0)),
        default_add_subnet_data(), 
        subnet_nodes,
        PropsType::Activate,
      )
    );

    let proposal_index = PropCount::<Test>::get();

    let proposal = Proposals::<Test>::get(proposal_index - 1);

    assert_err!(
      SubnetVoting::verify_proposal(
        RuntimeOrigin::signed(account(255)),
        proposal_index - 1, 
      ),
      Error::<Test>::NotSubnetNode
    );

    let proposal_post_err = Proposals::<Test>::get(proposal_index - 1);

    assert_eq!(proposal.subnet_nodes.len(), proposal_post_err.subnet_nodes.len());
  })
}

