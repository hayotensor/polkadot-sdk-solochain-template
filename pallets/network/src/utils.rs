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
use frame_system::pallet_prelude::BlockNumberFor;

impl<T: Config> Pallet<T> {
  pub fn get_current_block_as_u64() -> u64 {
    TryInto::try_into(<frame_system::Pallet<T>>::block_number())
      .ok()
      .expect("blockchain will not exceed 2^64 blocks; QED.")
  }

  pub fn convert_block_as_u64(block: BlockNumberFor<T>) -> u64 {
    TryInto::try_into(block)
      .ok()
      .expect("blockchain will not exceed 2^64 blocks; QED.")
  }
  
  pub fn get_eligible_epoch_block(epoch_length: u64, initialized: u64, epochs: u64) -> u64 {
    let eligible_block: u64 = initialized - (initialized % epoch_length) + epoch_length * epochs;
    eligible_block
  }

  // Loosely validates Node ID
  pub fn validate_peer_id(peer_id: PeerId) -> bool {
    let mut valid = false;

    let peer_id_0 = peer_id.0;

    let len = peer_id_0.len();

    // PeerId must be equal to or greater than 32 chars
    // PeerId must be equal to or less than 128 chars
    if len < 32 || len > 128 {
      return false
    };

    let first_char = peer_id_0[0];

    let second_char = peer_id_0[1];

    if first_char == 49 {
      // Node ID (ed25519, using the "identity" multihash) encoded as a raw base58btc multihash
      valid = len <= 128;
    } else if first_char == 81 && second_char == 109 {
      // Node ID (sha256) encoded as a raw base58btc multihash
      valid = len <= 128;
    } else if first_char == 102 || first_char == 98 || first_char == 122 || first_char == 109 {
      // Node ID (sha256) encoded as a CID
      valid = len <= 128;
    }
    
    valid
  }
  
  // Get subnet peer is eligible to be a subnet peer
  // Checks if account penalties do not surpass the max allowed penalties
  pub fn is_account_eligible(account_id: T::AccountId) -> bool {
    let max_account_penalty_count = MaxAccountPenaltyCount::<T>::get();
    let account_penalty_count = AccountPenaltyCount::<T>::get(account_id);
    account_penalty_count <= max_account_penalty_count
  }

  pub fn get_tx_rate_limit() -> u64 {
    TxRateLimit::<T>::get()
  }

  pub fn set_last_tx_block(key: &T::AccountId, block: u64) {
    LastTxBlock::<T>::insert(key, block)
  }

  pub fn get_last_tx_block(key: &T::AccountId) -> u64 {
    LastTxBlock::<T>::get(key)
  }

  pub fn exceeds_tx_rate_limit(prev_tx_block: u64, current_block: u64) -> bool {
    let rate_limit: u64 = Self::get_tx_rate_limit();
    if rate_limit == 0 || prev_tx_block == 0 {
      return false;
    }

    return current_block - prev_tx_block <= rate_limit;
  }

  // If a subnet or subnet peer is able to be included or submit consensus
  //
  // This checks if the block is equal to or greater than therefor shouldn't 
  // be used while checking if a subnet or subnet peer was able to accept or be 
  // included in consensus during the forming of consensus since it checks for
  // the previous epochs eligibility
  pub fn is_epoch_block_eligible(
    block: u64, 
    epoch_length: u64, 
    epochs: u64, 
    initialized: u64
  ) -> bool {
    block >= Self::get_eligible_epoch_block(
      epoch_length, 
      initialized, 
      epochs
    )
  }

  // Remove all account's subnet peers across all of their subnets
  pub fn do_remove_account_subnet_nodes(block: u64, account_id: T::AccountId) {
    let model_ids: Vec<u32> = AccountSubnets::<T>::get(account_id.clone());
    for subnet_id in model_ids.iter() {
      Self::do_remove_subnet_node(block, *subnet_id, account_id.clone());
    }
  }

  /// Remove subnet peer from subnet
  // to-do: Add slashing to subnet peers stake balance
  // note: We don't reset AccountPenaltyCount
  pub fn do_remove_subnet_node(block: u64, subnet_id: u32, account_id: T::AccountId) {
    // Take and remove SubnetNodesData account_id as key
    // `take()` returns and removes data
    if let Ok(subnet_node) = SubnetNodesData::<T>::try_get(subnet_id, account_id.clone()) {
      let peer_id = subnet_node.peer_id;

      SubnetNodesData::<T>::remove(subnet_id, account_id.clone());

      // Remove SubnetNodeAccount peer_id as key
      SubnetNodeAccount::<T>::remove(subnet_id, peer_id.clone());

      // Update SubnetAccount to reflect removal block instead of initialized block
      // Node will be able to unstake after required epochs have passed
      let mut model_accounts: BTreeMap<T::AccountId, u64> = SubnetAccount::<T>::get(subnet_id);
      model_accounts.insert(account_id.clone(), block);
      SubnetAccount::<T>::insert(subnet_id, model_accounts);

      // Update total subnet peers by substracting 1
      TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n -= 1);

      // Remove subnet_id from AccountSubnets
      let mut account_subnet_ids: Vec<u32> = AccountSubnets::<T>::get(account_id.clone());
      account_subnet_ids.retain(|&x| x != subnet_id);
      // Insert retained subnet_id's
      AccountSubnets::<T>::insert(account_id.clone(), account_subnet_ids);

      // Remove from attestations
      let epoch_length: u64 = T::EpochLength::get();
			let epoch: u64 = block / epoch_length;

      SubnetRewardsSubmission::<T>::try_mutate_exists(
        subnet_id,
        epoch as u32,
        |params| -> DispatchResult {
          let params = if let Some(params) = params {
            let mut attests = &mut params.attests;
            if attests.remove(&account_id.clone()) {
              params.attests = attests.clone();
            }
          };
          Ok(())
        }
      );
    
      // Reset sequential absent subnet node count
      SequentialAbsentSubnetNode::<T>::remove(subnet_id, account_id.clone());

      // Remove from classifications
      for class_id in SubnetNodeClass::iter() {
        let mut node_sets: BTreeMap<T::AccountId, u64> = SubnetNodesClasses::<T>::get(subnet_id, class_id);
        node_sets.retain(|k, _| *k != account_id.clone());
        SubnetNodesClasses::<T>::insert(subnet_id, class_id, node_sets);
      }

      Self::deposit_event(
        Event::SubnetNodeRemoved { 
          subnet_id: subnet_id, 
          account_id: account_id.clone(), 
          peer_id: peer_id,
          block: block
        }
      );
    }
  }

  pub fn get_min_subnet_nodes(base_node_memory: u128, memory_mb: u128) -> u32 {
    // --- Get min nodes based on default memory settings
    let real_min_subnet_nodes: u128 = memory_mb / base_node_memory;
    let mut min_subnet_nodes: u32 = MinSubnetNodes::<T>::get();
    if real_min_subnet_nodes as u32 > min_subnet_nodes {
      min_subnet_nodes = real_min_subnet_nodes as u32;
    }
    min_subnet_nodes
  }

  pub fn get_target_subnet_nodes(base_node_memory: u128, min_subnet_nodes: u32) -> u32 {
    Self::percent_mul(
      min_subnet_nodes.into(), 
      TargetSubnetNodesMultiplier::<T>::get()
    ) as u32 + min_subnet_nodes
  }

  pub fn get_model_initialization_cost(block: u64) -> u128 {
    T::SubnetInitializationCost::get()
  }

    /// Shift up subnet nodes to new classifications
  // This is used to know the len() of each class of subnet nodes instead of iterating through each time
  pub fn shift_node_classes(block: u64, epoch_length: u64) {
    for (subnet_id, _) in SubnetsData::<T>::iter() {
      let class_ids = SubnetNodeClass::iter();
      let last_class_id = class_ids.clone().last().unwrap();
      for mut class_id in class_ids {
        // Can't increase user class after last so skip
        if class_id == last_class_id {
          continue;
        }

        let node_sets: BTreeMap<T::AccountId, u64> = SubnetNodesClasses::<T>::get(
          subnet_id, 
          class_id.clone()
        );

        // If initialized but empty, then skip
        if node_sets.is_empty() {
          continue;
        }
        
        // --- Get next class to shift into
        let class_index = class_id.index();

        // --- Safe unwrap from `continue` from last
        let next_class_id: SubnetNodeClass = SubnetNodeClass::from_repr(class_index + 1).unwrap();

        // --- Copy the node sets for mutation
        let mut node_sets_copy: BTreeMap<T::AccountId, u64> = node_sets.clone();
        
        // --- Get next node sets for mutation or initialize new BTreeMap
        let mut next_node_sets: BTreeMap<T::AccountId, u64> = match SubnetNodesClasses::<T>::try_get(subnet_id, next_class_id) {
          Ok(next_node_sets) => next_node_sets,
          Err(_) => BTreeMap::new(),
        };

        // --- Get epochs required to be in class from the initialization block
        let epochs = SubnetNodeClassEpochs::<T>::get(class_id.clone());

        for node_set in node_sets.iter() {
          let account_eligible: bool = Self::is_account_eligible(node_set.0.clone());

          if !account_eligible {
            next_node_sets.remove(&node_set.0.clone());
            node_sets_copy.remove(&node_set.0.clone());
            continue;
          }

          if let Ok(subnet_node_data) = SubnetNodesData::<T>::try_get(subnet_id, node_set.0.clone()) {
            let initialized: u64 = subnet_node_data.initialized;
            if Self::is_epoch_block_eligible(
              block, 
              epoch_length, 
              epochs, 
              initialized
            ) {
              // --- Insert to the next classification, will only insert if doesn't already exist
              next_node_sets.insert(node_set.0.clone(), *node_set.1);
            }  
          } else {
            // Remove the account from classification if they don't exist anymore
            node_sets_copy.remove(&node_set.0.clone());
          }
        }
        // --- Update classifications
        SubnetNodesClasses::<T>::insert(subnet_id, class_id, node_sets_copy);
        SubnetNodesClasses::<T>::insert(subnet_id, next_class_id, next_node_sets);
      }
    }
  }

  pub fn do_choose_validator_and_accountants(block: u64, epoch: u32, epoch_length: u64) {
    let min_required_model_consensus_submit_epochs = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();
    let target_accountants_len: u32 = TargetAccountantsLength::<T>::get();

    for (subnet_id, data) in SubnetsData::<T>::iter() {
      let min_subnet_nodes = data.min_nodes;

      // --- Ensure model is able to submit consensus
      if block < Self::get_eligible_epoch_block(
        epoch_length, 
        data.initialized, 
        min_required_model_consensus_submit_epochs
      ) {
        continue
      }

      Self::choose_validator(
        block,
        subnet_id,
        min_subnet_nodes,
        epoch,
      );

      Self::choose_accountants(
        block,
        epoch,
        subnet_id,
        min_subnet_nodes,
        target_accountants_len,
      );
    }
  }

  // pub fn validate_signature(
  //   data: &Vec<u8>,
  //   signature: &T::OffchainSignature,
  //   signer: &T::AccountId,
  // ) -> DispatchResult {
  //   if signature.verify(&**data, &signer) {
  //     return Ok(())
  //   }

  //   // NOTE: for security reasons modern UIs implicitly wrap the data requested to sign into
  //   // <Bytes></Bytes>, that's why we support both wrapped and raw versions.
  //   let prefix = b"<Bytes>";
  //   let suffix = b"</Bytes>";
  //   let mut wrapped: Vec<u8> = Vec::with_capacity(data.len() + prefix.len() + suffix.len());
  //   wrapped.extend(prefix);
  //   wrapped.extend(data);
  //   wrapped.extend(suffix);

  //   ensure!(signature.verify(&*wrapped, &signer), Error::<T>::WrongSignature);

  //   Ok(())
  // }

  pub fn get_account_total_stake_balance(account_id: T::AccountId) -> u128 {
		let mut total_stake_balance = 0;
		for subnet_id in AccountSubnets::<T>::get(account_id.clone()).iter() {
      total_stake_balance += AccountSubnetStake::<T>::get(&account_id.clone(), subnet_id);
		}
		total_stake_balance
	}

}
