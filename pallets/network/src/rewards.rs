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
use sp_runtime::Saturating;

impl<T: Config> Pallet<T> {
  pub fn reward_subnets(block: u64, epoch: u32, epoch_length: u64) {
    // --- Get base rewards based on subnet memory requirements
    let base_reward_per_mb: u128 = BaseRewardPerMB::<T>::get();
    // --- Get required attestation percentage
    let min_attestation_percentage = MinAttestationPercentage::<T>::get();
    // --- Get max epochs in a row a subnet node can be absent from consensus data
    let max_absent = MaxSequentialAbsentSubnetNode::<T>::get();
    // --- Get the max penalties a subnet node can have before being removed from the network
    let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();
    // --- Get the attestation percentage for a subnet node to be removed from the network
    //     if they are not included in the validators consensus data
    // --- If this attestation threshold is exceeded, the subnet node that is absent will have its
    //     SequentialAbsentSubnetNode incrememented
    let node_attestation_removal_threshold = NodeAttestationRemovalThreshold::<T>::get();
    // --- Get the percentage of the subnet rewards that go to subnet delegate stakers
    let delegate_stake_rewards_percentage: u128 = DelegateStakeRewardsPercentage::<T>::get();

    for (subnet_id, data) in SubnetsData::<T>::iter() {
      // --- We don't check for minimum nodes because nodes cannot validate or attest if they are not met
      //     as they the validator will not be chosen in ``do_choose_validator_and_accountants`` if the 
      //     min nodes are not met on that epoch.
      if let Ok(submission) = SubnetRewardsSubmission::<T>::try_get(subnet_id, epoch) {
        // --- Get memory of the subnet
        let memory_mb = data.memory_mb;

        // --- Get subnet rewards
        let overall_subnet_reward: u128 = Self::percent_mul(base_reward_per_mb, memory_mb);

        // --- Get delegators rewards
        // We get the delegators rewards in case of rounding issues in favor of subnet nodes over delegators
        let delegate_stake_reward: u128 = Self::percent_mul(overall_subnet_reward, delegate_stake_rewards_percentage);

        // --- Get subnet nodes rewards
        let subnet_reward: u128 = overall_subnet_reward.saturating_sub(delegate_stake_reward);

        let min_nodes = data.min_nodes;
        let data_len = submission.data.len();

        // --- Get subnet nodes count to check against attestation count
        // ``reward_subnuts`` is called before ``shift_node_classes`` so we can know how many nodes are submittable
        // while in this function that should have in the epoch the rewards are destined for
        let submission_nodes_count = SubnetNodesClasses::<T>::get(subnet_id, SubnetNodeClass::Submittable).len() as u128;

        let submission_attestations: u128 = submission.attests.len() as u128;
        let mut attestation_percentage: u128 = Self::percent_div(submission_attestations, submission_nodes_count);
        // Redundant
        if attestation_percentage > Self::PERCENTAGE_FACTOR {
          attestation_percentage = Self::PERCENTAGE_FACTOR;
        }
        let validator: T::AccountId = submission.validator;

        // --- If validator submitted no data, or less than the minimum required subnet nodes 
        //     we assume the subnet is broken
        // There is no slashing if subnet is broken, only risk of subnet being removed
        if (data_len as u32) < min_nodes {
          // --- Increase the penalty count for the subnet
          // If the subnet is broken, the validator can avoid slashing by submitting consensus with null data
          SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

          // --- If subnet nodes aren't in consensus this is true
          // Since we can assume the subnet is in a broken state, we don't slash the validator
          // even if others do not attest to this state???

          // --- If the subnet nodes are not in agreement with the validator that the model is broken, we
          //     increase the penalty score for the validator
          if attestation_percentage < min_attestation_percentage {
            AccountPenaltyCount::<T>::mutate(validator, |n: &mut u32| *n += 1);
          }
          continue;
        }

        // --- If the minimum attestation not reached, assume validator is dishonest, slash, and continue
        if min_attestation_percentage > attestation_percentage {
          // --- Slash validator and increase penalty score
          Self::slash_validator(subnet_id, validator, attestation_percentage);
          
          // --- Attestation not successful, move on to next subnet
          continue
        }

        let sum: u128 = submission.sum;
        // let mut rewarded: BTreeSet<T::AccountId> = BTreeSet::new();
        for subnet_node in SubnetNodesData::<T>::iter_prefix_values(subnet_id) {
          let account_id: T::AccountId = subnet_node.account_id;
          let peer_id: PeerId = subnet_node.peer_id;

          let mut validated: bool = false;
          let mut subnet_node_data: SubnetNodeData = SubnetNodeData::default();
          // test copying submission.data and removing found peers to limit future iterations
          for submission_data in submission.data.iter() {
            if submission_data.peer_id == peer_id {
              validated = true;
              subnet_node_data = SubnetNodeData {
                peer_id: peer_id,
                score: submission_data.score,
              };
              break
            }
          }
    
          // --- If not validated, then remove them if consensus is reached and sequential absence threshold is reached
          if !validated {
            // --- To be removed or increase absent count, the consensus threshold must be reached
            if attestation_percentage > node_attestation_removal_threshold {
              // We don't slash nodes for not being in consensus
              // A node can be removed for any reason and may not be due to dishonesty

              // --- Mutate nodes absentee count if in consensus
              let absent_count = SequentialAbsentSubnetNode::<T>::get(subnet_id, account_id.clone());
              SequentialAbsentSubnetNode::<T>::insert(subnet_id, account_id.clone(), absent_count + 1);

              // --- Ensure maximum sequential removal consensus threshold is reached
              if absent_count + 1 > max_absent {
                Self::perform_remove_subnet_node(block, subnet_id, account_id.clone());
              }
            }
            continue;
          }

          // --- If not attested, do not receive rewards
          // We don't penalize accounts for not attesting data in case data is corrupted
          // It is up to subnet nodes to remove them via consensus
          if !submission.attests.contains(&account_id) {
            continue
          }

          // --- Decrease absent count by one if in consensus and attested consensus
          SequentialAbsentSubnetNode::<T>::mutate(subnet_id, account_id.clone(), |n: &mut u32| n.saturating_dec());

          // --- Calculate score percentage of peer versus sum
          let score_percentage: u128 = Self::percent_div(subnet_node_data.score, sum as u128);
          // --- Calculate score percentage of total subnet rewards
          let mut account_reward: u128 = Self::percent_mul(score_percentage, subnet_reward);

          if account_id == validator {
            account_reward += Self::get_validator_reward(attestation_percentage);    
          }

          // --- Skip if no rewards to give
          if account_reward == 0 {
            continue;
          }

          // --- Increase account stake and emit event
          Self::increase_account_stake(
            &account_id,
            subnet_id, 
            account_reward,
          ); 
        }

        // --- Portion of delegate staking
        Self::do_increase_delegate_stake(
          subnet_id,
          delegate_stake_reward,
        );

        // --- Increment down subnet penalty score on successful epochs
        SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| n.saturating_dec());
      } else if let Ok(rewards_validator) = SubnetRewardsValidator::<T>::try_get(subnet_id, epoch) {
        // --- If a validator has been chosen that means they are supposed to be submitting consensus data
        //     since the subnet is passed its MinRequiredSubnetConsensusSubmitEpochs
        // --- If there is no submission but validator chosen, increase penalty on subnet and validator
        // --- Increase the penalty count for the subnet
        // The next validator on the next epoch can increment the penalty score down
        SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

        // NOTE:
        //  Each subnet increases the penalty score if they don't have the minimum subnet nodes required by the time
        //  the subnet is enabled for emissions. This happens by the blockchain validator before choosing the subnet validator

        // If validator didn't submit anything, then slash
        // Even if a subnet is in a broken state, the chosen validator must submit blank data
        Self::slash_validator(subnet_id, rewards_validator, 0);
      }

      // TODO: Automatically remove subnet if greater than max penalties count
      // TODO: Get benchmark for removing max subnets in one epoch to ensure does not surpass max weights
      let subnet_penalty_count = SubnetPenaltyCount::<T>::get(subnet_id);
      if subnet_penalty_count > max_subnet_penalty_count {
        // Self::do_remove_subnet(block, subnet_id);
        Self::deactivate_subnet(
          data.path,
          SubnetRemovalReason::MaxPenalties,
        );
      }

      // TODO: Check delegate stake amount is above minimum required
    }
  }
}