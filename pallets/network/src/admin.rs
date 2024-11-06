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
  pub fn set_min_nodes_slope_parameters(params: MinNodesCurveParametersSet) -> DispatchResult {
    // let one_hundred = params.one_hundred;
    // let two_hundred = params.two_hundred;
    let one_hundred = Self::PERCENTAGE_FACTOR;
    let two_hundred = Self::TWO_HUNDRED_PERCENT_FACTOR;
    let x_curve_start = params.x_curve_start;
    let y_end = params.y_end;
    let y_start = params.y_start;
    let x_rise = Self::PERCENTAGE_FACTOR / 100;

    ensure!(
      y_start > y_end,
      Error::<T>::SubnetNotExist
    );

    // --- Linear Slope check
    let x_start_plus_1 = x_curve_start + x_rise;
    let x_start_plus_1_adj = (x_start_plus_1 - x_curve_start) * one_hundred / (one_hundred - x_curve_start);
    let y_start_minus_1 = (y_start - y_end) * (one_hundred - x_start_plus_1_adj) / one_hundred + y_end; 
    let y_rise = y_start - y_start_minus_1;
    let slope = y_rise * one_hundred / x_rise;
    let j = slope * two_hundred / one_hundred;
    let q = one_hundred * one_hundred / j * y_start / one_hundred;
    let max_x = one_hundred * one_hundred / j * y_start / one_hundred + (x_curve_start * one_hundred / two_hundred);
    
    ensure!(
      max_x >= one_hundred,
      Error::<T>::SubnetNotExist
    );

    MinNodesCurveParameters::<T>::put(params);

    Ok(())
  }

  pub fn set_base_subnet_node_memory_mb(value: u128) -> DispatchResult {
    BaseSubnetNodeMemoryMB::<T>::put(value);
    Ok(())
  }

  pub fn set_max_subnet_memory_mb(value: u128) -> DispatchResult {
    MaxSubnetMemoryMB::<T>::put(value);
    Ok(())
  }

  pub fn set_overall_max_subnet_memory_mb(value: u128) -> DispatchResult {
    TotalMaxSubnetMemoryMB::<T>::put(value);
    Ok(())
  }

  pub fn set_vote_subnet_in(path: Vec<u8>, memory_mb: u128) -> DispatchResult {
    Ok(())
  }

  pub fn set_vote_subnet_out(path: Vec<u8>) -> DispatchResult {
    Ok(())
  }

  pub fn set_max_subnets(value: u32) -> DispatchResult {
    ensure!(
      value <= 100,
      Error::<T>::InvalidMaxSubnets
    );

    MaxSubnets::<T>::set(value);

    Self::deposit_event(Event::SetMaxSubnets(value));

    Ok(())
  }

  pub fn set_min_subnet_nodes(value: u32) -> DispatchResult {
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
  pub fn set_min_required_subnet_consensus_submit_epochs(value: u64) -> DispatchResult {
    MinRequiredSubnetConsensusSubmitEpochs::<T>::put(value);
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

  pub fn set_max_subnet_rewards_weight(value: u128) -> DispatchResult {
    Ok(())
  }

  pub fn set_stake_reward_weight(value: u128) -> DispatchResult {
    Ok(())
  }

  pub fn set_subnet_per_peer_init_cost(value: u128) -> DispatchResult {
    Ok(())
  }

  pub fn set_subnet_consensus_unconfirmed_threshold(value: u128) -> DispatchResult {
    Ok(())
  }

  pub fn set_remove_subnet_node_epoch_percentage(value: u128) -> DispatchResult {
    Ok(())
  }
}