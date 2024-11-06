// Copyright (C) 2021 Subspace Labs, Inc.
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

//! Pallet for issuing rewards to block producers.

#![cfg_attr(not(feature = "std"), no_std)]

// pub mod weights;

pub use pallet::*;
use frame_system::{
  pallet_prelude::OriginFor,
  ensure_signed, ensure_root
};
use frame_support::{
  weights::Weight,
  pallet_prelude::DispatchResult,
  ensure,
  traits::EnsureOrigin,
};
use sp_std::vec::Vec;
use pallet_network::{MinNodesCurveParametersSet};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use frame_support::pallet_prelude::*;
  use pallet_network::AdminInterface as NetworkAdminInterface;
  use pallet_subnet_democracy::AdminInterface as SubnetDemocracyAdminInterface;
  
  #[pallet::config]
  pub trait Config: frame_system::Config + Sized {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    type CollectiveOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    
    type NetworkAdminInterface: NetworkAdminInterface<Self::AccountId>; 

    type SubnetDemocracyAdminInterface: SubnetDemocracyAdminInterface;
  }

  /// Pallet rewards for issuing rewards to block producers.
  #[pallet::pallet]
  pub struct Pallet<T>(_);

  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
  }

  /// A storage item for this pallet.
	///
	/// In this template, we are declaring a storage item called `Something` that stores a single
	/// `u32` value. Learn more about runtime storage here: <https://docs.substrate.io/build/runtime-storage/>
	#[pallet::storage]
	pub type Something<T> = StorageValue<_, u32>;

  //
  // All conditional logic takes place in the callee pallets themselves
  //
  #[pallet::call]
  impl<T: Config> Pallet<T> {
    #[pallet::call_index(0)]
    #[pallet::weight(0)]
    pub fn set_vote_subnet_in(origin: OriginFor<T>, value: Vec<u8>, memory_mb: u128) -> DispatchResult {
      ensure_root(origin)?;
      T::NetworkAdminInterface::set_vote_subnet_in(value, memory_mb)
    }

    #[pallet::call_index(1)]
    #[pallet::weight(0)]
    pub fn set_vote_subnet_out(origin: OriginFor<T>, value: Vec<u8>) -> DispatchResult {
      ensure_root(origin)?;
      T::NetworkAdminInterface::set_vote_subnet_out(value)
    }

    #[pallet::call_index(2)]
    #[pallet::weight(0)]
    pub fn set_max_subnets(origin: OriginFor<T>, value: u32) -> DispatchResult {
      ensure_root(origin)?;
      T::NetworkAdminInterface::set_max_subnets(value)
    }

    #[pallet::call_index(3)]
    #[pallet::weight(0)]
    pub fn set_min_subnet_nodes(origin: OriginFor<T>, value: u32) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_min_subnet_nodes(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(4)]
    #[pallet::weight(0)]
    pub fn set_max_subnet_nodes(origin: OriginFor<T>, value: u32) -> DispatchResult {
      ensure_root(origin)?;
      T::NetworkAdminInterface::set_max_subnet_nodes(value)
    }

    #[pallet::call_index(5)]
    #[pallet::weight(0)]
    pub fn set_min_stake_balance(origin: OriginFor<T>, value: u128) -> DispatchResult {
      ensure_root(origin)?;
      T::NetworkAdminInterface::set_min_stake_balance(value)
    }

    #[pallet::call_index(6)]
    #[pallet::weight(0)]
    pub fn set_tx_rate_limit(origin: OriginFor<T>, value: u64) -> DispatchResult {
      ensure_root(origin)?;
      T::NetworkAdminInterface::set_tx_rate_limit(value)
    }

    #[pallet::call_index(7)]
    #[pallet::weight(0)]
    pub fn set_max_consensus_epochs_errors(origin: OriginFor<T>, value: u32) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_max_consensus_epochs_errors(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(8)]
    #[pallet::weight(0)]
    pub fn set_min_required_subnet_consensus_submit_epochs(origin: OriginFor<T>, value: u64) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_min_required_subnet_consensus_submit_epochs(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(9)]
    #[pallet::weight(0)]
    pub fn set_min_required_peer_consensus_submit_epochs(origin: OriginFor<T>, value: u64) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_min_required_peer_consensus_submit_epochs(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(10)]
    #[pallet::weight(0)]
    pub fn set_min_required_peer_consensus_inclusion_epochs(origin: OriginFor<T>, value: u64) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_min_required_peer_consensus_inclusion_epochs(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(11)]
    #[pallet::weight(0)]
    pub fn set_min_required_peer_consensus_dishonesty_epochs(origin: OriginFor<T>, value: u64) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_min_required_peer_consensus_dishonesty_epochs(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(12)]
    #[pallet::weight(0)]
    pub fn set_max_outlier_delta_percent(origin: OriginFor<T>, value: u8) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_max_outlier_delta_percent(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(13)]
    #[pallet::weight(0)]
    pub fn set_subnet_node_consensus_submit_percent_requirement(origin: OriginFor<T>, value: u128) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_subnet_node_consensus_submit_percent_requirement(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(14)]
    #[pallet::weight(0)]
    pub fn set_consensus_blocks_interval(origin: OriginFor<T>, value: u64) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_consensus_blocks_interval(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(15)]
    #[pallet::weight(0)]
    pub fn set_peer_removal_threshold(origin: OriginFor<T>, value: u128) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_peer_removal_threshold(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(16)]
    #[pallet::weight(0)]
    pub fn set_max_subnet_rewards_weight(origin: OriginFor<T>, value: u128) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_max_subnet_rewards_weight(value)
      // Update
      Ok(())
    }
    
    #[pallet::call_index(17)]
    #[pallet::weight(0)]
    pub fn set_stake_reward_weight(origin: OriginFor<T>, value: u128) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_stake_reward_weight(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(18)]
    #[pallet::weight(0)]
    pub fn set_subnet_per_peer_init_cost(origin: OriginFor<T>, value: u128) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_subnet_per_peer_init_cost(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(19)]
    #[pallet::weight(0)]
    pub fn set_subnet_consensus_unconfirmed_threshold(origin: OriginFor<T>, value: u128) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_subnet_consensus_unconfirmed_threshold(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(20)]
    #[pallet::weight(0)]
    pub fn set_remove_subnet_node_epoch_percentage(origin: OriginFor<T>, value: u128) -> DispatchResult {
      ensure_root(origin)?;
      // T::NetworkAdminInterface::set_remove_subnet_node_epoch_percentage(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(21)]
    #[pallet::weight(0)]
    pub fn set_peer_vote_premium(origin: OriginFor<T>, value: u32) -> DispatchResult {
      // let account_id: T::AccountId = ensure_signed(origin)?;
      T::CollectiveOrigin::ensure_origin(origin)?;

      // log::error!("account_id set_peer_vote_premium {:?}", account_id);
      Something::<T>::put(value);
      // ensure_root(origin)?;
      // T::SubnetDemocracyAdminInterface::set_peer_vote_premium(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(22)]
    #[pallet::weight(0)]
    pub fn set_quorum(origin: OriginFor<T>, value: u128) -> DispatchResult {
      ensure_root(origin)?;
      // T::SubnetDemocracyAdminInterface::set_quorum(value)
      // Update
      Ok(())
    }

    #[pallet::call_index(23)]
    #[pallet::weight(0)]
    pub fn remove_subnet(origin: OriginFor<T>, path: Vec<u8>) -> DispatchResult {
      T::CollectiveOrigin::ensure_origin(origin)?;
      T::NetworkAdminInterface::council_remove_subnet(path);
      Ok(())
    }

    #[pallet::call_index(24)]
    #[pallet::weight(0)]
    pub fn set_min_nodes_slope_parameters(origin: OriginFor<T>, params: MinNodesCurveParametersSet) -> DispatchResult {
      T::CollectiveOrigin::ensure_origin(origin)?;
      T::NetworkAdminInterface::set_min_nodes_slope_parameters(params);
      Ok(())
    }
  }
}