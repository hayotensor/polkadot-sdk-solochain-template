
//! Autogenerated weights for `pallet_network`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 42.0.0
//! DATE: 2024-12-24, STEPS: `5`, REPEAT: `2`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `Bob`, CPU: `11th Gen Intel(R) Core(TM) i7-11800H @ 2.30GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/release/solochain-template-node
// benchmark
// pallet
// --chain=dev
// --wasm-execution=compiled
// --pallet=pallet_network
// --extrinsic=*
// --steps=5
// --repeat=2
// --output=pallets/network/src/weights.rs
// --template
// ./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_network`.
pub trait WeightInfo {
	fn register_subnet() -> Weight;
	fn activate_subnet() -> Weight;
	fn add_subnet_node() -> Weight;
	fn register_subnet_node() -> Weight;
	fn activate_subnet_node() -> Weight;
	fn deactivate_subnet_node() -> Weight;
	fn remove_subnet_node() -> Weight;
	fn add_to_stake() -> Weight;
	fn remove_stake() -> Weight;
	fn add_to_delegate_stake() -> Weight;
}

/// Weights for `pallet_network` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Network::SubnetPaths` (r:1 w:1)
	/// Proof: `Network::SubnetPaths` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetsData` (r:1 w:1)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxSubnets` (r:1 w:0)
	/// Proof: `Network::MaxSubnets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinSubnetRegistrationBlocks` (r:1 w:0)
	/// Proof: `Network::MinSubnetRegistrationBlocks` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxSubnetRegistrationBlocks` (r:1 w:0)
	/// Proof: `Network::MaxSubnetRegistrationBlocks` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxSubnetMemoryMB` (r:1 w:0)
	/// Proof: `Network::MaxSubnetMemoryMB` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Network::StakeVaultBalance` (r:1 w:1)
	/// Proof: `Network::StakeVaultBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnets` (r:1 w:1)
	/// Proof: `Network::TotalSubnets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::BaseSubnetNodeMemoryMB` (r:1 w:0)
	/// Proof: `Network::BaseSubnetNodeMemoryMB` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinNodesCurveParameters` (r:1 w:0)
	/// Proof: `Network::MinNodesCurveParameters` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinSubnetNodes` (r:1 w:0)
	/// Proof: `Network::MinSubnetNodes` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TargetSubnetNodesMultiplier` (r:1 w:0)
	/// Proof: `Network::TargetSubnetNodesMultiplier` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn register_subnet() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `616`
		//  Estimated: `4081`
		// Minimum execution time: 48_385_000 picoseconds.
		Weight::from_parts(101_686_000, 4081)
			.saturating_add(T::DbWeight::get().reads(13_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: `Network::SubnetsData` (r:1 w:1)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetActivationEnactmentPeriod` (r:1 w:0)
	/// Proof: `Network::SubnetActivationEnactmentPeriod` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetNodes` (r:1 w:0)
	/// Proof: `Network::TotalSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetDelegateStakeBalance` (r:1 w:0)
	/// Proof: `Network::TotalSubnetDelegateStakeBalance` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinStakeBalance` (r:1 w:0)
	/// Proof: `Network::MinStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinSubnetDelegateStakePercentage` (r:1 w:0)
	/// Proof: `Network::MinSubnetDelegateStakePercentage` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinSubnetDelegateStake` (r:1 w:0)
	/// Proof: `Network::MinSubnetDelegateStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetRewardsValidator` (r:1 w:1)
	/// Proof: `Network::SubnetRewardsValidator` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodesData` (r:13 w:0)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `InsecureRandomnessCollectiveFlip::RandomMaterial` (r:1 w:0)
	/// Proof: `InsecureRandomnessCollectiveFlip::RandomMaterial` (`max_values`: Some(1), `max_size`: Some(2594), added: 3089, mode: `MaxEncodedLen`)
	fn activate_subnet() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3099`
		//  Estimated: `36264`
		// Minimum execution time: 91_529_000 picoseconds.
		Weight::from_parts(145_018_000, 36264)
			.saturating_add(T::DbWeight::get().reads(22_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Network::SubnetsData` (r:1 w:0)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxSubnetNodes` (r:1 w:0)
	/// Proof: `Network::MaxSubnetNodes` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodesData` (r:1 w:1)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodeAccount` (r:1 w:1)
	/// Proof: `Network::SubnetNodeAccount` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::AccountSubnetStake` (r:1 w:1)
	/// Proof: `Network::AccountSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinStakeBalance` (r:1 w:0)
	/// Proof: `Network::MinStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxStakeBalance` (r:1 w:0)
	/// Proof: `Network::MaxStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Network::LastTxBlock` (r:1 w:1)
	/// Proof: `Network::LastTxBlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TxRateLimit` (r:1 w:0)
	/// Proof: `Network::TxRateLimit` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalAccountStake` (r:1 w:1)
	/// Proof: `Network::TotalAccountStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetStake` (r:1 w:1)
	/// Proof: `Network::TotalSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalStake` (r:1 w:1)
	/// Proof: `Network::TotalStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalActiveSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalActiveSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn add_subnet_node() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2656`
		//  Estimated: `6121`
		// Minimum execution time: 99_686_000 picoseconds.
		Weight::from_parts(100_438_000, 6121)
			.saturating_add(T::DbWeight::get().reads(15_u64))
			.saturating_add(T::DbWeight::get().writes(10_u64))
	}
	/// Storage: `Network::SubnetsData` (r:1 w:0)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxSubnetNodes` (r:1 w:0)
	/// Proof: `Network::MaxSubnetNodes` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodesData` (r:1 w:1)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodeAccount` (r:1 w:1)
	/// Proof: `Network::SubnetNodeAccount` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::AccountSubnetStake` (r:1 w:1)
	/// Proof: `Network::AccountSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinStakeBalance` (r:1 w:0)
	/// Proof: `Network::MinStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxStakeBalance` (r:1 w:0)
	/// Proof: `Network::MaxStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Network::LastTxBlock` (r:1 w:1)
	/// Proof: `Network::LastTxBlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TxRateLimit` (r:1 w:0)
	/// Proof: `Network::TxRateLimit` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalAccountStake` (r:1 w:1)
	/// Proof: `Network::TotalAccountStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetStake` (r:1 w:1)
	/// Proof: `Network::TotalSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalStake` (r:1 w:1)
	/// Proof: `Network::TotalStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn register_subnet_node() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2645`
		//  Estimated: `6110`
		// Minimum execution time: 87_285_000 picoseconds.
		Weight::from_parts(114_730_000, 6110)
			.saturating_add(T::DbWeight::get().reads(14_u64))
			.saturating_add(T::DbWeight::get().writes(9_u64))
	}
	/// Storage: `Network::SubnetsData` (r:1 w:0)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodesData` (r:1 w:1)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalActiveSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalActiveSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn activate_subnet_node() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1145`
		//  Estimated: `4610`
		// Minimum execution time: 22_836_000 picoseconds.
		Weight::from_parts(24_129_000, 4610)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Network::SubnetNodesData` (r:1 w:1)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalActiveSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalActiveSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn deactivate_subnet_node() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `952`
		//  Estimated: `4417`
		// Minimum execution time: 22_541_000 picoseconds.
		Weight::from_parts(22_793_000, 4417)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Network::SubnetNodesData` (r:14 w:1)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetRewardsSubmission` (r:1 w:1)
	/// Proof: `Network::SubnetRewardsSubmission` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::AccountantData` (r:1 w:1)
	/// Proof: `Network::AccountantData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalActiveSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalActiveSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodeAccount` (r:0 w:1)
	/// Proof: `Network::SubnetNodeAccount` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodePenalties` (r:0 w:1)
	/// Proof: `Network::SubnetNodePenalties` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn remove_subnet_node() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3044`
		//  Estimated: `38684`
		// Minimum execution time: 99_627_000 picoseconds.
		Weight::from_parts(102_352_000, 38684)
			.saturating_add(T::DbWeight::get().reads(18_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
	/// Storage: `Network::SubnetsData` (r:1 w:0)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodesData` (r:1 w:0)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::AccountSubnetStake` (r:1 w:1)
	/// Proof: `Network::AccountSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinStakeBalance` (r:1 w:0)
	/// Proof: `Network::MinStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxStakeBalance` (r:1 w:0)
	/// Proof: `Network::MaxStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Network::LastTxBlock` (r:1 w:1)
	/// Proof: `Network::LastTxBlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TxRateLimit` (r:1 w:0)
	/// Proof: `Network::TxRateLimit` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalAccountStake` (r:1 w:1)
	/// Proof: `Network::TotalAccountStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetStake` (r:1 w:1)
	/// Proof: `Network::TotalSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalStake` (r:1 w:1)
	/// Proof: `Network::TotalStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn add_to_stake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2627`
		//  Estimated: `6092`
		// Minimum execution time: 74_047_000 picoseconds.
		Weight::from_parts(74_430_000, 6092)
			.saturating_add(T::DbWeight::get().reads(11_u64))
			.saturating_add(T::DbWeight::get().writes(6_u64))
	}
	/// Storage: `Network::SubnetNodesData` (r:1 w:0)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::AccountSubnetStake` (r:1 w:1)
	/// Proof: `Network::AccountSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinStakeBalance` (r:1 w:0)
	/// Proof: `Network::MinStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::LastTxBlock` (r:1 w:1)
	/// Proof: `Network::LastTxBlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TxRateLimit` (r:1 w:0)
	/// Proof: `Network::TxRateLimit` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalAccountStake` (r:1 w:1)
	/// Proof: `Network::TotalAccountStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetStake` (r:1 w:1)
	/// Proof: `Network::TotalSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalStake` (r:1 w:1)
	/// Proof: `Network::TotalStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetStakeUnbondingLedger` (r:1 w:1)
	/// Proof: `Network::SubnetStakeUnbondingLedger` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn remove_stake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2083`
		//  Estimated: `5548`
		// Minimum execution time: 56_516_000 picoseconds.
		Weight::from_parts(161_547_000, 5548)
			.saturating_add(T::DbWeight::get().reads(9_u64))
			.saturating_add(T::DbWeight::get().writes(6_u64))
	}
	/// Storage: `Network::SubnetsData` (r:1 w:0)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::AccountSubnetDelegateStakeShares` (r:1 w:1)
	/// Proof: `Network::AccountSubnetDelegateStakeShares` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetDelegateStakeShares` (r:1 w:1)
	/// Proof: `Network::TotalSubnetDelegateStakeShares` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetDelegateStakeBalance` (r:1 w:1)
	/// Proof: `Network::TotalSubnetDelegateStakeBalance` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxDelegateStakeBalance` (r:1 w:0)
	/// Proof: `Network::MaxDelegateStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Network::LastTxBlock` (r:1 w:1)
	/// Proof: `Network::LastTxBlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TxRateLimit` (r:1 w:0)
	/// Proof: `Network::TxRateLimit` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn add_to_delegate_stake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1227`
		//  Estimated: `4692`
		// Minimum execution time: 56_592_000 picoseconds.
		Weight::from_parts(62_145_000, 4692)
			.saturating_add(T::DbWeight::get().reads(8_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	/// Storage: `Network::SubnetPaths` (r:1 w:1)
	/// Proof: `Network::SubnetPaths` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetsData` (r:1 w:1)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxSubnets` (r:1 w:0)
	/// Proof: `Network::MaxSubnets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinSubnetRegistrationBlocks` (r:1 w:0)
	/// Proof: `Network::MinSubnetRegistrationBlocks` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxSubnetRegistrationBlocks` (r:1 w:0)
	/// Proof: `Network::MaxSubnetRegistrationBlocks` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxSubnetMemoryMB` (r:1 w:0)
	/// Proof: `Network::MaxSubnetMemoryMB` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Network::StakeVaultBalance` (r:1 w:1)
	/// Proof: `Network::StakeVaultBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnets` (r:1 w:1)
	/// Proof: `Network::TotalSubnets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::BaseSubnetNodeMemoryMB` (r:1 w:0)
	/// Proof: `Network::BaseSubnetNodeMemoryMB` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinNodesCurveParameters` (r:1 w:0)
	/// Proof: `Network::MinNodesCurveParameters` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinSubnetNodes` (r:1 w:0)
	/// Proof: `Network::MinSubnetNodes` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TargetSubnetNodesMultiplier` (r:1 w:0)
	/// Proof: `Network::TargetSubnetNodesMultiplier` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn register_subnet() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `616`
		//  Estimated: `4081`
		// Minimum execution time: 48_385_000 picoseconds.
		Weight::from_parts(101_686_000, 4081)
			.saturating_add(RocksDbWeight::get().reads(13_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
	/// Storage: `Network::SubnetsData` (r:1 w:1)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetActivationEnactmentPeriod` (r:1 w:0)
	/// Proof: `Network::SubnetActivationEnactmentPeriod` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetNodes` (r:1 w:0)
	/// Proof: `Network::TotalSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetDelegateStakeBalance` (r:1 w:0)
	/// Proof: `Network::TotalSubnetDelegateStakeBalance` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinStakeBalance` (r:1 w:0)
	/// Proof: `Network::MinStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinSubnetDelegateStakePercentage` (r:1 w:0)
	/// Proof: `Network::MinSubnetDelegateStakePercentage` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinSubnetDelegateStake` (r:1 w:0)
	/// Proof: `Network::MinSubnetDelegateStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetRewardsValidator` (r:1 w:1)
	/// Proof: `Network::SubnetRewardsValidator` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodesData` (r:13 w:0)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `InsecureRandomnessCollectiveFlip::RandomMaterial` (r:1 w:0)
	/// Proof: `InsecureRandomnessCollectiveFlip::RandomMaterial` (`max_values`: Some(1), `max_size`: Some(2594), added: 3089, mode: `MaxEncodedLen`)
	fn activate_subnet() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3099`
		//  Estimated: `36264`
		// Minimum execution time: 91_529_000 picoseconds.
		Weight::from_parts(145_018_000, 36264)
			.saturating_add(RocksDbWeight::get().reads(22_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `Network::SubnetsData` (r:1 w:0)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxSubnetNodes` (r:1 w:0)
	/// Proof: `Network::MaxSubnetNodes` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodesData` (r:1 w:1)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodeAccount` (r:1 w:1)
	/// Proof: `Network::SubnetNodeAccount` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::AccountSubnetStake` (r:1 w:1)
	/// Proof: `Network::AccountSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinStakeBalance` (r:1 w:0)
	/// Proof: `Network::MinStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxStakeBalance` (r:1 w:0)
	/// Proof: `Network::MaxStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Network::LastTxBlock` (r:1 w:1)
	/// Proof: `Network::LastTxBlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TxRateLimit` (r:1 w:0)
	/// Proof: `Network::TxRateLimit` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalAccountStake` (r:1 w:1)
	/// Proof: `Network::TotalAccountStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetStake` (r:1 w:1)
	/// Proof: `Network::TotalSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalStake` (r:1 w:1)
	/// Proof: `Network::TotalStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalActiveSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalActiveSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn add_subnet_node() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2656`
		//  Estimated: `6121`
		// Minimum execution time: 99_686_000 picoseconds.
		Weight::from_parts(100_438_000, 6121)
			.saturating_add(RocksDbWeight::get().reads(15_u64))
			.saturating_add(RocksDbWeight::get().writes(10_u64))
	}
	/// Storage: `Network::SubnetsData` (r:1 w:0)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxSubnetNodes` (r:1 w:0)
	/// Proof: `Network::MaxSubnetNodes` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodesData` (r:1 w:1)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodeAccount` (r:1 w:1)
	/// Proof: `Network::SubnetNodeAccount` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::AccountSubnetStake` (r:1 w:1)
	/// Proof: `Network::AccountSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinStakeBalance` (r:1 w:0)
	/// Proof: `Network::MinStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxStakeBalance` (r:1 w:0)
	/// Proof: `Network::MaxStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Network::LastTxBlock` (r:1 w:1)
	/// Proof: `Network::LastTxBlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TxRateLimit` (r:1 w:0)
	/// Proof: `Network::TxRateLimit` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalAccountStake` (r:1 w:1)
	/// Proof: `Network::TotalAccountStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetStake` (r:1 w:1)
	/// Proof: `Network::TotalSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalStake` (r:1 w:1)
	/// Proof: `Network::TotalStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn register_subnet_node() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2645`
		//  Estimated: `6110`
		// Minimum execution time: 87_285_000 picoseconds.
		Weight::from_parts(114_730_000, 6110)
			.saturating_add(RocksDbWeight::get().reads(14_u64))
			.saturating_add(RocksDbWeight::get().writes(9_u64))
	}
	/// Storage: `Network::SubnetsData` (r:1 w:0)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodesData` (r:1 w:1)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalActiveSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalActiveSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn activate_subnet_node() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1145`
		//  Estimated: `4610`
		// Minimum execution time: 22_836_000 picoseconds.
		Weight::from_parts(24_129_000, 4610)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `Network::SubnetNodesData` (r:1 w:1)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalActiveSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalActiveSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn deactivate_subnet_node() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `952`
		//  Estimated: `4417`
		// Minimum execution time: 22_541_000 picoseconds.
		Weight::from_parts(22_793_000, 4417)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `Network::SubnetNodesData` (r:14 w:1)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetRewardsSubmission` (r:1 w:1)
	/// Proof: `Network::SubnetRewardsSubmission` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::AccountantData` (r:1 w:1)
	/// Proof: `Network::AccountantData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalActiveSubnetNodes` (r:1 w:1)
	/// Proof: `Network::TotalActiveSubnetNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodeAccount` (r:0 w:1)
	/// Proof: `Network::SubnetNodeAccount` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodePenalties` (r:0 w:1)
	/// Proof: `Network::SubnetNodePenalties` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn remove_subnet_node() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3044`
		//  Estimated: `38684`
		// Minimum execution time: 99_627_000 picoseconds.
		Weight::from_parts(102_352_000, 38684)
			.saturating_add(RocksDbWeight::get().reads(18_u64))
			.saturating_add(RocksDbWeight::get().writes(7_u64))
	}
	/// Storage: `Network::SubnetsData` (r:1 w:0)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodesData` (r:1 w:0)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::AccountSubnetStake` (r:1 w:1)
	/// Proof: `Network::AccountSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinStakeBalance` (r:1 w:0)
	/// Proof: `Network::MinStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxStakeBalance` (r:1 w:0)
	/// Proof: `Network::MaxStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Network::LastTxBlock` (r:1 w:1)
	/// Proof: `Network::LastTxBlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TxRateLimit` (r:1 w:0)
	/// Proof: `Network::TxRateLimit` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalAccountStake` (r:1 w:1)
	/// Proof: `Network::TotalAccountStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetStake` (r:1 w:1)
	/// Proof: `Network::TotalSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalStake` (r:1 w:1)
	/// Proof: `Network::TotalStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn add_to_stake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2627`
		//  Estimated: `6092`
		// Minimum execution time: 74_047_000 picoseconds.
		Weight::from_parts(74_430_000, 6092)
			.saturating_add(RocksDbWeight::get().reads(11_u64))
			.saturating_add(RocksDbWeight::get().writes(6_u64))
	}
	/// Storage: `Network::SubnetNodesData` (r:1 w:0)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::AccountSubnetStake` (r:1 w:1)
	/// Proof: `Network::AccountSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MinStakeBalance` (r:1 w:0)
	/// Proof: `Network::MinStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::LastTxBlock` (r:1 w:1)
	/// Proof: `Network::LastTxBlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TxRateLimit` (r:1 w:0)
	/// Proof: `Network::TxRateLimit` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalAccountStake` (r:1 w:1)
	/// Proof: `Network::TotalAccountStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetStake` (r:1 w:1)
	/// Proof: `Network::TotalSubnetStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalStake` (r:1 w:1)
	/// Proof: `Network::TotalStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetStakeUnbondingLedger` (r:1 w:1)
	/// Proof: `Network::SubnetStakeUnbondingLedger` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn remove_stake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2083`
		//  Estimated: `5548`
		// Minimum execution time: 56_516_000 picoseconds.
		Weight::from_parts(161_547_000, 5548)
			.saturating_add(RocksDbWeight::get().reads(9_u64))
			.saturating_add(RocksDbWeight::get().writes(6_u64))
	}
	/// Storage: `Network::SubnetsData` (r:1 w:0)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::AccountSubnetDelegateStakeShares` (r:1 w:1)
	/// Proof: `Network::AccountSubnetDelegateStakeShares` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetDelegateStakeShares` (r:1 w:1)
	/// Proof: `Network::TotalSubnetDelegateStakeShares` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TotalSubnetDelegateStakeBalance` (r:1 w:1)
	/// Proof: `Network::TotalSubnetDelegateStakeBalance` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxDelegateStakeBalance` (r:1 w:0)
	/// Proof: `Network::MaxDelegateStakeBalance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Network::LastTxBlock` (r:1 w:1)
	/// Proof: `Network::LastTxBlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::TxRateLimit` (r:1 w:0)
	/// Proof: `Network::TxRateLimit` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn add_to_delegate_stake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1227`
		//  Estimated: `4692`
		// Minimum execution time: 56_592_000 picoseconds.
		Weight::from_parts(62_145_000, 4692)
			.saturating_add(RocksDbWeight::get().reads(8_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
}