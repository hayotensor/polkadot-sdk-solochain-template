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
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
  pub fn set_vote_model_in(path: Vec<u8>, memory_mb: u128) -> DispatchResult {
    // ensure subnet doesn't exists by path
    ensure!(
      !SubnetPaths::<T>::contains_key(path.clone()),
      Error::<T>::SubnetNotExist
    );
    
    let pre_subnet_data = PreSubnetData {
      path: path.clone(),
      memory_mb: memory_mb,
    };
  
    let vote_subnet_data = VoteSubnetData {
      data: pre_subnet_data,
      active: true,
    };

		SubnetActivated::<T>::insert(path.clone(), vote_subnet_data);

    Self::deposit_event(Event::SetVoteSubnetIn(path.clone()));

    Ok(())
  }

  pub fn set_vote_model_out(path: Vec<u8>) -> DispatchResult {
    // ensure subnet exists by path
    ensure!(
      SubnetPaths::<T>::contains_key(path.clone()),
      Error::<T>::SubnetNotExist
    );

    let pre_subnet_data = PreSubnetData {
      path: path.clone(),
      memory_mb: 0,
    };
  
    let vote_subnet_data = VoteSubnetData {
      data: pre_subnet_data,
      active: false,
    };

		SubnetActivated::<T>::insert(path.clone(), vote_subnet_data);

    Self::deposit_event(Event::SetVoteSubnetOut(path.clone()));

    Ok(())
  }

  pub fn set_max_models(value: u32) -> DispatchResult {
    ensure!(
      value <= 100,
      Error::<T>::InvalidMaxSubnets
    );

    MaxSubnets::<T>::set(value);

    Self::deposit_event(Event::SetMaxSubnets(value));

    Ok(())
  }

  pub fn set_min_subnet_nodes(value: u32) -> DispatchResult {
    // let max_subnet_nodes = MaxSubnetNodes::<T>::get();

    // let peer_removal_threshold = NodeRemovalThreshold::<T>::get();
    // let min_value = Self::percent_div_round_up(1 as u128, Self::PERCENTAGE_FACTOR - peer_removal_threshold);

    // // Ensure over 10
    // // Ensure less than MaxSubnetNodes
    // // Ensure divisible by NodeRemovalThreshold
    // //  • e.g. if the threshold is .8, we need a minimum of
    // ensure!(
    //   value >= 9 && value <= max_subnet_nodes && value >= min_value as u32,
    //   Error::<T>::InvalidMinSubnetNodes
    // );

    // MinSubnetNodes::<T>::set(value);

    // Self::deposit_event(Event::SetMinSubnetNodes(value));

    Ok(())
  }

  pub fn set_max_subnet_nodes(value: u32) -> DispatchResult {
    // Ensure divisible by .01%
    // Ensuring less than or equal to PERCENTAGE_FACTOR is redundant but keep
    // for possible updates in future versions
    // * Remove `value <= Self::PERCENTAGE_FACTOR` if never used in mainnet
    ensure!(
      value <= 1000 && value as u128 <= Self::PERCENTAGE_FACTOR,
      Error::<T>::InvalidMaxSubnetNodes
    );

    MaxSubnetNodes::<T>::set(value);

    Self::deposit_event(Event::SetMaxSubnetNodes(value));

    Ok(())
  }

  pub fn set_min_stake_balance(value: u128) -> DispatchResult {
    ensure!(
      value > 0,
      Error::<T>::InvalidMinStakeBalance
    );

    MinStakeBalance::<T>::set(value);

    Self::deposit_event(Event::SetMinStakeBalance(value));

    Ok(())
  }

  pub fn set_tx_rate_limit(value: u64) -> DispatchResult {
    TxRateLimit::<T>::set(value);

    Self::deposit_event(Event::SetTxRateLimit(value));

    Ok(())
  }

  pub fn set_max_consensus_epochs_errors(value: u32) -> DispatchResult {
    Ok(())
  }

  // Set the time required for a subnet to be in storage before consensus can be formed
  // This allows time for peers to become subnet peers to the subnet doesn't increment `no-consensus'`
  pub fn set_min_required_model_consensus_submit_epochs(value: u64) -> DispatchResult {
    Ok(())
  }

  pub fn set_min_required_peer_consensus_submit_epochs(value: u64) -> DispatchResult {
    Ok(())
  }
  
  pub fn set_min_required_peer_consensus_inclusion_epochs(value: u64) -> DispatchResult {
    Ok(())
  }

  pub fn set_min_required_peer_consensus_dishonesty_epochs(value: u64) -> DispatchResult {
    Ok(())
  }

  pub fn set_max_outlier_delta_percent(value: u8) -> DispatchResult {
    Ok(())
  }

  pub fn set_subnet_node_consensus_submit_percent_requirement(value: u128) -> DispatchResult {
    Ok(())
  }

  pub fn set_consensus_blocks_interval(value: u64) -> DispatchResult {
    Ok(())
  }

  pub fn set_peer_removal_threshold(value: u128) -> DispatchResult {
    Ok(())
  }

  pub fn set_max_model_rewards_weight(value: u128) -> DispatchResult {
    Ok(())
  }

  pub fn set_stake_reward_weight(value: u128) -> DispatchResult {
    Ok(())
  }

  pub fn set_model_per_peer_init_cost(value: u128) -> DispatchResult {
    Ok(())
  }

  pub fn set_model_consensus_unconfirmed_threshold(value: u128) -> DispatchResult {
    Ok(())
  }

  pub fn set_remove_subnet_node_epoch_percentage(value: u128) -> DispatchResult {
    Ok(())
  }
}