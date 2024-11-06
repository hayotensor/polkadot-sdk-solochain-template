//! Benchmarking setup for pallet-network
// ./target/release/solochain-template-node benchmark pallet --chain=dev --wasm-execution=compiled --pallet=pallet_network --extrinsic=* --steps=5 --repeat=2 --output="pallets/network/src/weights.rs" --template ./.maintain/frame-weight-template.hbs

// ./target/release/solochain-template-node benchmark pallet \
// --pallet pallet_network \
// --extrinsic '*' \
// --steps 50 \
// --repeat 20 \
// --output pallets/network/src/weights.rs


// cargo build --release --features runtime-benchmarks
// cargo test --release --features runtime-benchmarks
// Build only this pallet
// cargo build --package pallet-network --features runtime-benchmarks
// cargo build --package pallet-collective --features runtime-benchmarks
// cargo +nightly build --release --features runtime-benchmarks

#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Network;
use crate::{SubnetPaths, MinRequiredUnstakeEpochs, TotalStake, TotalSubnetDelegateStakeBalance, TotalSubnetDelegateStakeShares};
use frame_benchmarking::v2::*;
// use frame_benchmarking::{account, whitelist_account, BenchmarkError};
use frame_support::{
	assert_noop, assert_ok,
	traits::{Currency, EnsureOrigin, Get, OnInitialize, UnfilteredDispatchable},
};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::Vec;
use sp_core::OpaquePeerId as PeerId;
use scale_info::prelude::vec;
use scale_info::prelude::format;

const SEED: u32 = 0;


const DEFAULT_SCORE: u128 = 5000;
const DEFAULT_MEM_MB: u128 = 50000;
const DEFAULT_SUBNET_PATH: &str = "petals-team/StableBeluga2";
const DEFAULT_SUBNET_PATH_2: &str = "petals-team/StableBeluga3";
const DEFAULT_SUBNET_NODE_STAKE: u128 = 1000e+18 as u128;
const DEFAULT_STAKE_TO_BE_ADDED: u128 = 1000e+18 as u128;
const DEFAULT_DELEGATE_STAKE_TO_BE_ADDED: u128 = 1000e+18 as u128;

fn peer(id: u32) -> PeerId {
	// let peer_id = format!("12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA{id}");
  let peer_id = format!("QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N{id}"); 
	PeerId(peer_id.into())
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	// Give the account half of the maximum value of the `Balance` type.
	// Otherwise some transfers will fail with an overflow error.
	let deposit_amount: u128 = MinStakeBalance::<T>::get() + 10000;
	T::Currency::deposit_creating(&caller, deposit_amount.try_into().ok().expect("REASON"));
	caller
}

fn funded_initializer<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	// Give the account half of the maximum value of the `Balance` type.
	// Otherwise some transfers will fail with an overflow error.
	let deposit_amount: u128 = Network::<T>::get_subnet_initialization_cost(0) + 10000;
	T::Currency::deposit_creating(&caller, deposit_amount.try_into().ok().expect("REASON"));
	caller
}

fn build_subnet<T: Config>(subnet_path: Vec<u8>) {
	let funded_initializer = funded_account::<T>("funded_initializer", 0);

  let add_subnet_data = PreSubnetData {
    path: subnet_path.clone().into(),
    memory_mb: DEFAULT_MEM_MB,
  };
  assert_ok!(
    Network::<T>::activate_subnet(
      funded_initializer.clone(),
      funded_initializer.clone(),
      add_subnet_data,
    )
  );
}

// Returns total staked on subnet
fn build_subnet_nodes<T: Config>(subnet_id: u32, start: u32, end: u32, amount: u128) -> u128 {
  let mut amount_staked = 0;
  for n in start..end {
    let subnet_node = funded_account::<T>("subnet_node", n);
    amount_staked += amount;
    assert_ok!(
      Network::<T>::add_subnet_node(
        RawOrigin::Signed(subnet_node).into(),
        subnet_id,
        peer(n),
        amount,
      ) 
    );
  }
  amount_staked
}

pub fn u64_to_block<T: frame_system::Config>(input: u64) -> BlockNumberFor<T> {
	input.try_into().ok().expect("REASON")
}

pub fn get_current_block_as_u64<T: frame_system::Config>() -> u64 {
	TryInto::try_into(<frame_system::Pallet<T>>::block_number())
		.ok()
		.expect("blockchain will not exceed 2^64 blocks; QED.")
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn add_subnet_node() {
		// let caller: T::AccountId = whitelisted_caller();
		let subnet_node_account = funded_account::<T>("subnet_node_account", 0);
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());

		// let subnet_id = SubnetPaths::<T>::get(DEFAULT_SUBNET_PATH.into()).unwrap();
		// let subnet_id = Network::<T>::subnet_paths(DEFAULT_SUBNET_PATH.into()).unwrap();
		let subnet_id = 1;

		#[extrinsic_call]
		add_subnet_node(RawOrigin::Signed(subnet_node_account.clone()), subnet_id, peer(0), DEFAULT_SUBNET_NODE_STAKE);
		
		assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), 1);
		// let subnet_node_data = SubnetNodesData::<T>::get(subnet_id, subnet_node_account);
		let subnet_node_data = Network::<T>::subnet_nodes(subnet_id, subnet_node_account.clone());
		assert_eq!(subnet_node_data.account_id, subnet_node_account.clone());
		assert_eq!(subnet_node_data.peer_id, peer(0));
		assert_eq!(subnet_node_data.initialized, 1);

		let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node_account.clone(), subnet_id);
		assert_eq!(account_subnet_stake, DEFAULT_SUBNET_NODE_STAKE);
	}

	#[benchmark]
	fn remove_subnet_node() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;

		let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", 0);
		assert_ok!(
			Network::<T>::add_subnet_node(
        RawOrigin::Signed(subnet_node_account.clone()).into(),
        subnet_id,
        peer(0),
        DEFAULT_SUBNET_NODE_STAKE,
      )
		);
		assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), 1);

		let subnet_node_account_1: T::AccountId = funded_account::<T>("subnet_node_account", 1);
		assert_ok!(
			Network::<T>::add_subnet_node(
        RawOrigin::Signed(subnet_node_account_1.clone()).into(),
        subnet_id,
        peer(1),
        DEFAULT_SUBNET_NODE_STAKE,
      )
		);
		assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), 2);

		#[extrinsic_call]
		remove_subnet_node(RawOrigin::Signed(subnet_node_account_1.clone()), subnet_id);
		
		assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), 1);
		let subnet_node_data = Network::<T>::subnet_nodes(subnet_id, subnet_node_account_1.clone());
		assert_eq!(subnet_node_data.initialized, 0);
	}

	#[benchmark]
	fn add_to_stake() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;

		let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", 0);
		assert_ok!(
			Network::<T>::add_subnet_node(
        RawOrigin::Signed(subnet_node_account.clone()).into(),
        subnet_id,
        peer(0),
        DEFAULT_SUBNET_NODE_STAKE,
      )
		);

		T::Currency::deposit_creating(&subnet_node_account, DEFAULT_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"));

		#[extrinsic_call]
		add_to_stake(RawOrigin::Signed(subnet_node_account.clone()), subnet_id, DEFAULT_STAKE_TO_BE_ADDED);
		
		let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node_account.clone(), subnet_id);
		assert_eq!(account_subnet_stake, DEFAULT_SUBNET_NODE_STAKE + DEFAULT_STAKE_TO_BE_ADDED);
	}

	#[benchmark]
	fn remove_stake() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;

		let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", 0);
		assert_ok!(
			Network::<T>::add_subnet_node(
        RawOrigin::Signed(subnet_node_account.clone()).into(),
        subnet_id,
        peer(0),
        DEFAULT_SUBNET_NODE_STAKE,
      )
		);

		T::Currency::deposit_creating(&subnet_node_account, DEFAULT_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"));
		assert_ok!(
			Network::<T>::add_to_stake(
				RawOrigin::Signed(subnet_node_account.clone()).into(), 
				subnet_id, 
				DEFAULT_STAKE_TO_BE_ADDED
			)
		);
		let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node_account.clone(), subnet_id);
		assert_eq!(account_subnet_stake, DEFAULT_SUBNET_NODE_STAKE + DEFAULT_STAKE_TO_BE_ADDED);

		let epoch_length = T::EpochLength::get();
    let min_required_unstake_epochs = MinRequiredUnstakeEpochs::<T>::get();

		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epoch_length * min_required_unstake_epochs + 1)
		);

		#[extrinsic_call]
		remove_stake(RawOrigin::Signed(subnet_node_account.clone()), subnet_id, DEFAULT_STAKE_TO_BE_ADDED);
		
		let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node_account.clone(), subnet_id);
		assert_eq!(account_subnet_stake, DEFAULT_SUBNET_NODE_STAKE);
	}

	#[benchmark]
	fn add_to_delegate_stake() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;

		let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

		let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::<T>::convert_to_shares(
      DEFAULT_DELEGATE_STAKE_TO_BE_ADDED,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

		let starting_delegator_balance = T::Currency::free_balance(&delegate_account.clone());

		#[extrinsic_call]
		add_to_delegate_stake(RawOrigin::Signed(delegate_account.clone()), subnet_id, DEFAULT_DELEGATE_STAKE_TO_BE_ADDED);

		let post_delegator_balance = T::Currency::free_balance(&delegate_account.clone());
    assert_eq!(post_delegator_balance, starting_delegator_balance - DEFAULT_DELEGATE_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"));

    let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

    let delegate_balance = Network::<T>::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
	}

	#[benchmark]
	fn transfer_delegate_stake() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let from_subnet_id = 1;
		build_subnet::<T>(DEFAULT_SUBNET_PATH_2.into());
		let to_subnet_id = 2;

		let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

		assert_ok!(
			Network::<T>::add_to_delegate_stake(
				RawOrigin::Signed(delegate_account.clone()).into(), 
				from_subnet_id, 
				DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
			)
		);

		let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), from_subnet_id);
		let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(from_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id);

		let mut from_delegate_balance = Network::<T>::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

		#[extrinsic_call]
		transfer_delegate_stake(
			RawOrigin::Signed(delegate_account.clone()), 
			from_subnet_id, 
			to_subnet_id, 
			total_subnet_delegated_stake_shares
		);

    let from_delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), from_subnet_id);
    assert_eq!(from_delegate_shares, 0);

    let to_delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), to_subnet_id);
    assert_ne!(to_delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(to_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(to_subnet_id);

    let to_delegate_balance = Network::<T>::convert_to_balance(
      to_delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // Will lose about .01% of the transfer value on first transfer into a pool
    // The balance should be about ~99% of the ``from`` subnet to the ``to`` subnet
    assert!(
      (to_delegate_balance >= Network::<T>::percent_mul(from_delegate_balance, 9999)) &&
      (to_delegate_balance <= from_delegate_balance)
    );
	}


	// #[benchmark]
	// fn cause_error() {
	// 	Something::<T>::put(100u32);
	// 	let caller: T::AccountId = whitelisted_caller();
	// 	#[extrinsic_call]
	// 	cause_error(RawOrigin::Signed(caller));

	// 	assert_eq!(Something::<T>::get(), Some(101u32));
	// }

	impl_benchmark_test_suite!(Network, crate::mock::new_test_ext(), crate::mock::Test);
}
