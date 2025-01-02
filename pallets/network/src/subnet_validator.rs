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
use frame_support::pallet_prelude::DispatchResultWithPostInfo;
use frame_support::pallet_prelude::Pays;

impl<T: Config> Pallet<T> {
  /// Submit subnet scores per subnet node
  /// Validator of the epoch receives rewards when attestation passes consensus
  pub fn do_validate(
    subnet_id: u32, 
    account_id: T::AccountId,
    block: u64, 
    epoch_length: u64,
    epoch: u32,
    mut data: Vec<SubnetNodeData>,
  ) -> DispatchResultWithPostInfo {
    // TODO: Add parameter for params data in case a validator has a reason behind why they left
    //       a specific node(s) out of the consensus data for the other subnet nodes to verify

    // --- Ensure current subnet validator 
    let validator = SubnetRewardsValidator::<T>::get(subnet_id, epoch).ok_or(Error::<T>::InvalidValidator)?;
    
    ensure!(
      account_id == validator,
      Error::<T>::InvalidValidator
    );

    // --- Ensure not submitted already
    ensure!(
      !SubnetRewardsSubmission::<T>::contains_key(subnet_id, epoch),
      Error::<T>::SubnetRewardsAlreadySubmitted
    );

    // Remove duplicates based on peer_id
    data.dedup_by(|a, b| a.peer_id == b.peer_id);

    // Remove idle classified entries
    // Each peer must have an inclusion classification at minimum
    data.retain(|x| {
      match SubnetNodesData::<T>::try_get(
        subnet_id, 
        SubnetNodeAccount::<T>::get(subnet_id, x.peer_id.clone())
      ) {
        Ok(subnet_node) => subnet_node.has_classification(&SubnetNodeClass::Included, epoch as u64),
        Err(()) => false,
      }
    });

    //
    // --- Qualify the data
    //

    // --- Get count of eligible nodes that can be submitted for consensus rewards
    // This is the maximum amount of nodes that can be entered
    let included_nodes = Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch as u64);
    let included_nodes_count = included_nodes.len();

    // --- Ensure data isn't greater than current registered subnet peers
    ensure!(
      data.len() as u32 <= included_nodes_count as u32,
      Error::<T>::InvalidRewardsDataLength
    );
    
    // --- Sum of all entries scores
    // Each score is then used against the sum(scores) for emissions
    // We don't check data accuracy here because that's the job of attesters
    let scores_sum = data.iter().fold(0, |acc, x| acc + x.score);

    let submittable_nodes = Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Submittable, epoch as u64);
    let submittable_nodes_count = submittable_nodes.len();

    // If data.len() is 0 then the validator is deeming the epoch as invalid

    // --- Validator auto-attests the epoch
    let mut attests: BTreeMap<T::AccountId, u64> = BTreeMap::new();
    attests.insert(account_id.clone(), block);

    let rewards_data: RewardsData<T::AccountId> = RewardsData {
      validator: account_id.clone(),
      sum: scores_sum,
      attests: attests,
      data: data,
      complete: false,
    };

    SubnetRewardsSubmission::<T>::insert(subnet_id, epoch, rewards_data);
  
    Self::deposit_event(
      Event::ValidatorSubmission { 
        subnet_id: subnet_id, 
        account_id: account_id, 
        epoch: epoch,
      }
    );

    Ok(Pays::No.into())
  }

    /// Attest validator subnet rewards data
  // Nodes must attest data to receive rewards
  pub fn do_attest(
    subnet_id: u32, 
    account_id: T::AccountId,
    block: u64, 
    epoch_length: u64,
    epoch: u32,
  ) -> DispatchResultWithPostInfo {
    // --- Ensure subnet node exists and is submittable
    match SubnetNodesData::<T>::try_get(
      subnet_id, 
      account_id.clone()
    ) {
      Ok(subnet_node) => subnet_node.has_classification(&SubnetNodeClass::Submittable, epoch as u64),
      Err(()) => return Err(Error::<T>::SubnetNodeNotExist.into()),
    };

    SubnetRewardsSubmission::<T>::try_mutate_exists(
      subnet_id,
      epoch.clone(),
      |maybe_params| -> DispatchResult {
        let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetRewardsSubmission)?;
        let mut attests = &mut params.attests;

        ensure!(attests.insert(account_id.clone(), block) == None, Error::<T>::AlreadyAttested);

        params.attests = attests.clone();
        Ok(())
      }
    )?;

    Self::deposit_event(
      Event::Attestation { 
        subnet_id: subnet_id, 
        account_id: account_id, 
        epoch: epoch,
      }
    );

    Ok(Pays::No.into())
  }

  pub fn choose_validator(
    block: u64,
    subnet_id: u32,
    account_ids: Vec<T::AccountId>,
    min_subnet_nodes: u32,
    epoch: u32,
  ) {
    // TODO: Make sure this is only called if subnet is activated and on the following epoch
    
    // Redundant
    // If validator already chosen, then return
    if let Ok(rewards_validator) = SubnetRewardsValidator::<T>::try_get(subnet_id, epoch) {
      return
    }

    let subnet_nodes_len = account_ids.len();
    
    // --- Ensure min subnet peers that are submittable are at least the minimum required
    // --- Consensus cannot begin until this minimum is reached
    // --- If not min subnet peers count then accountant isn't needed
    if (subnet_nodes_len as u32) < min_subnet_nodes {
      return
    }

    let rand_index = Self::get_random_number((subnet_nodes_len - 1) as u32, block as u32);

    // --- Choose random accountant from eligible accounts
    let validator: &T::AccountId = &account_ids[rand_index as usize];

    // --- Insert validator for next epoch
    SubnetRewardsValidator::<T>::insert(subnet_id, epoch, validator);
  }

  // // Get random account within subnet
  // fn get_random_account(
  //   block: u64,
  //   account_ids: Vec<T::AccountId>,
  // ) -> Option<T::AccountId> {
  //   // --- Get accountant
  //   let accounts_len = account_ids.len();
  //   if accounts_len == 0 {
  //     return None;
  //   }
      
  //   // --- Get random number within the amount of eligible peers
  //   let rand_index = Self::get_random_number((accounts_len - 1) as u32, block as u32);

  //   // --- Choose random accountant from eligible accounts
  //   let new_account: &T::AccountId = &account_ids[rand_index as usize];
        
  //   Some(new_account.clone())
  // }

  /// Return the validators reward that submitted data on the previous epoch
  // The attestation percentage must be greater than the MinAttestationPercentage
  pub fn get_validator_reward(
    attestation_percentage: u128,
  ) -> u128 {
    if MinAttestationPercentage::<T>::get() > attestation_percentage {
      return 0
    }
    Self::percent_mul(BaseValidatorReward::<T>::get(), attestation_percentage)
  }

  pub fn slash_validator(subnet_id: u32, validator: T::AccountId, attestation_percentage: u128, block: u64) {
    // We never ensure balance is above 0 because any validator chosen must have the target stake
    // balance at a minimum

    // --- Get stake balance
    // This could be greater than the target stake balance
    let account_subnet_stake: u128 = AccountSubnetStake::<T>::get(validator.clone(), subnet_id);

    // --- Get slash amount up to max slash
    //
    let mut slash_amount: u128 = Self::percent_mul(account_subnet_stake, SlashPercentage::<T>::get());
    // --- Update slash amount up to attestation percent
    slash_amount = Self::percent_mul(slash_amount, Self::PERCENTAGE_FACTOR - attestation_percentage);
    // --- Update slash amount up to max slash
    let max_slash: u128 = MaxSlashAmount::<T>::get();
    if slash_amount > max_slash {
      slash_amount = max_slash
    }
    
    // --- Decrease account stake
    Self::decrease_account_stake(
      &validator.clone(),
      subnet_id, 
      slash_amount,
    );

    // --- Increase validator penalty count
    // AccountPenaltyCount::<T>::mutate(validator.clone(), |n: &mut u32| *n += 1);
    // SubnetNodePenalties::<T>::mutate(subnet_id, validator.clone(), |n: &mut u32| *n += 1);

    let penalties = SubnetNodePenalties::<T>::get(subnet_id, validator.clone());
    SubnetNodePenalties::<T>::insert(subnet_id, validator.clone(), penalties + 1);

    // --- Ensure maximum sequential removal consensus threshold is reached
    if penalties + 1 > MaxSubnetNodePenalties::<T>::get() {
      // --- Increase account penalty count
      Self::perform_remove_subnet_node(block, subnet_id, validator.clone());
    } else {
      
    }

    Self::deposit_event(
      Event::Slashing { 
        subnet_id: subnet_id, 
        account_id: validator, 
        amount: slash_amount,
      }
    );

  }

  /// Increase a subnet nodes classification
  // Nodes that enter before the activation of a subnet are automatically Submittable, otherwise
  // on entry they are classified as `Idle`
  // After `x` epochs, they can increase their classification to `Inclusion`
  //    - This is used as a way for subnets nodes to do preliminary events before they are ready to be included in
  pub fn increase_classification(subnet_id: u32, account_id: T::AccountId) -> DispatchResult {
    let subnet_node = match SubnetNodesData::<T>::try_get(subnet_id, account_id) {
      Ok(subnet_node) => subnet_node,
      Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
    };

    // --- Get classification

    // --- Get `x` required epochs to increase classification

    // --- Check the most recent `x` count of epochs

    // -- Must be in included `x` epochs
    Ok(())
  }
}