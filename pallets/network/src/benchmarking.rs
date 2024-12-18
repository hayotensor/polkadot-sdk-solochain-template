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
use crate::{
	SubnetPaths, 
	MinRequiredUnstakeEpochs, 
	TotalStake, TotalSubnetDelegateStakeBalance, TotalSubnetDelegateStakeShares, DelegateStakeUnbondingLedger};
use frame_benchmarking::v2::*;
// use frame_benchmarking::{account, whitelist_account, BenchmarkError};
use frame_support::{
	assert_noop, assert_ok,
	traits::{EnsureOrigin, Get, OnInitialize, UnfilteredDispatchable},
};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::Vec;
use sp_core::OpaquePeerId as PeerId;
use scale_info::prelude::vec;
use scale_info::prelude::format;
use sp_runtime::SaturatedConversion;
const SEED: u32 = 0;


const DEFAULT_SCORE: u128 = 5000;
const DEFAULT_SUBNET_MEM_MB: u128 = 50000;
const DEFAULT_SUBNET_PATH: &str = "petals-team/StableBeluga2";
const DEFAULT_SUBNET_PATH_2: &str = "petals-team/StableBeluga3";
const DEFAULT_SUBNET_NODE_STAKE: u128 = 1000e+18 as u128;
const DEFAULT_STAKE_TO_BE_ADDED: u128 = 1000e+18 as u128;
const DEFAULT_DELEGATE_STAKE_TO_BE_ADDED: u128 = 1000e+18 as u128;

pub type BalanceOf<T> = <T as Config>::Currency;

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

fn get_min_subnet_nodes<T: Config>() -> u32 {
	let base_node_memory: u128 = BaseSubnetNodeMemoryMB::<T>::get();
	Network::<T>::get_min_subnet_nodes(base_node_memory, DEFAULT_SUBNET_MEM_MB)
}
fn build_subnet<T: Config>(subnet_path: Vec<u8>) {
	let funded_initializer = funded_account::<T>("funded_initializer", 0);

  let add_subnet_data = RegistrationSubnetData {
    path: subnet_path.clone().into(),
    memory_mb: DEFAULT_SUBNET_MEM_MB,
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

fn subnet_node_data(start: u32, end: u32) -> Vec<SubnetNodeData> {
  // initialize peer consensus data array
  let mut subnet_node_data: Vec<SubnetNodeData> = Vec::new();
  for n in start..end {
    let peer_subnet_node_data: SubnetNodeData = SubnetNodeData {
      peer_id: peer(n),
      score: DEFAULT_SCORE,
    };
    subnet_node_data.push(peer_subnet_node_data);
  }
  subnet_node_data
}

pub fn u64_to_block<T: frame_system::Config>(input: u64) -> BlockNumberFor<T> {
	input.try_into().ok().expect("REASON")
}

pub fn get_current_block_as_u64<T: frame_system::Config>() -> u64 {
	TryInto::try_into(<frame_system::Pallet<T>>::block_number())
		.ok()
		.expect("blockchain will not exceed 2^64 blocks; QED.")
}

pub fn u128_to_balance<T: frame_system::Config + pallet::Config>(
	input: u128,
) -> Option<
	<<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
> {
	input.try_into().ok()
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

		assert_eq!(Network::<T>::total_account_stake(subnet_node_account.clone()), DEFAULT_SUBNET_NODE_STAKE);    
		assert_eq!(Network::<T>::total_subnet_nodes(subnet_id), 1 as u32);
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
		assert!(
      (delegate_balance >= Network::<T>::percent_mul(DEFAULT_DELEGATE_STAKE_TO_BE_ADDED, 9999)) &&
      (delegate_balance <= DEFAULT_DELEGATE_STAKE_TO_BE_ADDED)
    );
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

		let from_delegate_balance = Network::<T>::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

		#[extrinsic_call]
		transfer_delegate_stake(
			RawOrigin::Signed(delegate_account.clone()), 
			from_subnet_id, 
			to_subnet_id, 
			delegate_shares
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

	#[benchmark]
	fn remove_delegate_stake() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;
		let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
		assert_ok!(
			Network::<T>::add_to_delegate_stake(
				RawOrigin::Signed(delegate_account.clone()).into(), 
				subnet_id, 
				DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
			)
		);
		let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);

		#[extrinsic_call]
		remove_delegate_stake(
			RawOrigin::Signed(delegate_account.clone()), 
			subnet_id, 
			delegate_shares
		);

    let unbondings: BTreeMap<u64, u128> = DelegateStakeUnbondingLedger::<T>::get(delegate_account.clone(), subnet_id);
    assert_eq!(unbondings.len(), 1);

		let (epoch, balance) = unbondings.iter().next().unwrap();
    assert_eq!(*epoch, 0);
    assert_eq!(*balance, delegate_shares);
	}

	#[benchmark]
	fn claim_delegate_stake_unbondings() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;
		let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
		assert_ok!(
			Network::<T>::add_to_delegate_stake(
				RawOrigin::Signed(delegate_account.clone()).into(), 
				subnet_id, 
				DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
			)
		);
		let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);

		assert_ok!(
			Network::<T>::remove_delegate_stake(
				RawOrigin::Signed(delegate_account.clone()).into(), 
				subnet_id, 
				delegate_shares
			)
		);

		let epoch_length = T::EpochLength::get();
    let delegate_stake_cooldown_epochs = T::DelegateStakeCooldownEpochs::get();

		let unbondings: BTreeMap<u64, u128> = DelegateStakeUnbondingLedger::<T>::get(delegate_account.clone(), subnet_id);
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();

		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>((epoch_length + 1) * delegate_stake_cooldown_epochs)
		);

		let balance = T::Currency::free_balance(&delegate_account.clone());

		#[extrinsic_call]
		claim_delegate_stake_unbondings(
			RawOrigin::Signed(delegate_account.clone()), 
			subnet_id, 
		);

		let after_claim_balance = T::Currency::free_balance(&delegate_account.clone());
		let ledger_balance_as_balance = u128_to_balance::<T>(*ledger_balance);
    assert_eq!(after_claim_balance, balance + ledger_balance_as_balance.unwrap());

    let unbondings: BTreeMap<u64, u128> = DelegateStakeUnbondingLedger::<T>::get(delegate_account.clone(), subnet_id);
    assert_eq!(unbondings.len(), 0);
	}

	#[benchmark]
	fn increase_delegate_stake() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;
		let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
		assert_ok!(
			Network::<T>::add_to_delegate_stake(
				RawOrigin::Signed(delegate_account.clone()).into(), 
				subnet_id, 
				DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
			)
		);

		let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);
		let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

		let delegate_balance = Network::<T>::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

		let funder = funded_account::<T>("funder", 0);

		#[extrinsic_call]
		increase_delegate_stake(RawOrigin::Signed(funder), subnet_id, DEFAULT_SUBNET_NODE_STAKE);
		
		let increased_delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);
		let increased_total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
    let increased_total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

		let increased_delegate_balance = Network::<T>::convert_to_balance(
      increased_delegate_shares,
      increased_total_subnet_delegated_stake_shares,
      increased_total_subnet_delegated_stake_balance
    );
		assert_eq!(increased_total_subnet_delegated_stake_balance, total_subnet_delegated_stake_balance + DEFAULT_SUBNET_NODE_STAKE);

		assert_ne!(increased_delegate_balance, delegate_balance);
		assert!(increased_delegate_balance > delegate_balance);
	}

	#[benchmark]
	fn validate() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;

		let epoch_length = T::EpochLength::get();

		let min_required_subnet_consensus_submit_epochs: u64 = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();	
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epoch_length * min_required_subnet_consensus_submit_epochs)
		);

		let n_peers: u32 = Network::<T>::max_subnet_nodes();

    let amount: u128 = 				 1000000000000000000000;

		build_subnet_nodes::<T>(subnet_id, 0, n_peers, amount);

		let epochs = SubnetNodeClassEpochs::<T>::get(SubnetNodeClass::Submittable);
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epochs * (epoch_length + 1))
		);

		let block_number = get_current_block_as_u64::<T>();

		// shift node classes
    Network::<T>::shift_node_classes(block_number, epoch_length);

		let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;

    Network::<T>::do_choose_validator_and_accountants(
			get_current_block_as_u64::<T>(), 
			epoch as u32, 
			epoch_length
		);

		let validator = SubnetRewardsValidator::<T>::get(subnet_id, epoch as u32);
    assert!(validator != None, "Validator is None");

		let subnet_node_data_vec = subnet_node_data(0, n_peers);

		#[extrinsic_call]
		validate(RawOrigin::Signed(validator.clone().unwrap()), subnet_id, subnet_node_data_vec.clone());

		let submission = SubnetRewardsSubmission::<T>::get(subnet_id, epoch as u32).unwrap();

    assert_eq!(submission.validator, validator.clone().unwrap(), "Err: validator");
    assert_eq!(submission.data.len(), subnet_node_data_vec.clone().len(), "Err: data len");
    assert_eq!(submission.attests.len(), 1, "Err: attests"); // validator auto-attests
	}

	#[benchmark]
	fn attest() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;

		let epoch_length = T::EpochLength::get();

		let min_required_subnet_consensus_submit_epochs: u64 = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();	
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epoch_length * min_required_subnet_consensus_submit_epochs)
		);

		let n_peers: u32 = Network::<T>::max_subnet_nodes();

    let amount: u128 = 				 1000000000000000000000;

		build_subnet_nodes::<T>(subnet_id, 0, n_peers, amount);

		let epochs = SubnetNodeClassEpochs::<T>::get(SubnetNodeClass::Submittable);
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epochs * (epoch_length + 1))
		);

		let block_number = get_current_block_as_u64::<T>();

		// shift node classes
    Network::<T>::shift_node_classes(block_number, epoch_length);

		let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;

    Network::<T>::do_choose_validator_and_accountants(
			get_current_block_as_u64::<T>(), 
			epoch as u32, 
			epoch_length
		);

		let validator = SubnetRewardsValidator::<T>::get(subnet_id, epoch as u32);
    assert!(validator != None, "Validator is None");

		let subnet_node_data_vec = subnet_node_data(0, n_peers);

		assert_ok!(
			Network::<T>::validate(
				RawOrigin::Signed(validator.clone().unwrap()).into(), 
				subnet_id, 
				subnet_node_data_vec.clone()
			)
		);
	
		let attester = funded_account::<T>("subnet_node", 1);

		#[extrinsic_call]
		attest(RawOrigin::Signed(attester.clone()), subnet_id);

		let submission = SubnetRewardsSubmission::<T>::get(subnet_id, epoch as u32).unwrap();

		// validator + attester
    assert_eq!(submission.attests.len(), 2 as usize);
    assert_eq!(submission.attests.get(&attester.clone()), Some(&attester.clone()));
	}

	#[benchmark]
	fn propose() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;

		let epoch_length = T::EpochLength::get();

		let min_required_subnet_consensus_submit_epochs: u64 = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();	
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epoch_length * min_required_subnet_consensus_submit_epochs)
		);

		let n_peers: u32 = Network::<T>::max_subnet_nodes();

    let amount: u128 = 				 1000000000000000000000;

		build_subnet_nodes::<T>(subnet_id, 0, n_peers, amount);

		let epochs = SubnetNodeClassEpochs::<T>::get(SubnetNodeClass::Accountant);
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epochs * (epoch_length + 1))
		);

		let block_number = get_current_block_as_u64::<T>();

		// shift node classes
    Network::<T>::shift_node_classes(block_number, epoch_length);

		let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;

		let proposer = funded_account::<T>("subnet_node", 0);
		let defendant = funded_account::<T>("subnet_node", 1);

		let proposal_bid_amount = ProposalBidAmount::<T>::get();
    let plaintiff_starting_balance = T::Currency::free_balance(&proposer.clone());
		let data = Vec::new();

		let accountant_nodes = SubnetNodesClasses::<T>::get(subnet_id, SubnetNodeClass::Accountant);

		#[extrinsic_call]
		propose(RawOrigin::Signed(proposer.clone()), subnet_id, peer(1), data.clone());

    let plaintiff_after_balance = T::Currency::free_balance(&proposer.clone());
    assert_eq!(plaintiff_starting_balance - u128_to_balance::<T>(proposal_bid_amount).unwrap(), plaintiff_after_balance);

		let proposal = Proposals::<T>::get(subnet_id, 0);
    assert_eq!(proposal.subnet_id, subnet_id);
    assert_eq!(proposal.plaintiff, proposer.clone());
    assert_eq!(proposal.defendant, defendant);
    assert_eq!(proposal.plaintiff_bond, proposal_bid_amount);
    assert_eq!(proposal.defendant_bond, 0);
    assert_eq!(proposal.eligible_voters.len(), accountant_nodes.len());
    assert_eq!(proposal.start_block, get_current_block_as_u64::<T>());
    assert_eq!(proposal.challenge_block, 0);
    assert_eq!(proposal.plaintiff_data, data);
    assert_eq!(proposal.defendant_data, data);
    assert_eq!(proposal.complete, false);
	}

	#[benchmark]
	fn cancel_proposal() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;

		let epoch_length = T::EpochLength::get();

		let min_required_subnet_consensus_submit_epochs: u64 = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();	
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epoch_length * min_required_subnet_consensus_submit_epochs)
		);

		let n_peers: u32 = Network::<T>::max_subnet_nodes();

    let amount: u128 = 				 1000000000000000000000;

		build_subnet_nodes::<T>(subnet_id, 0, n_peers, amount);

		let epochs = SubnetNodeClassEpochs::<T>::get(SubnetNodeClass::Accountant);
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epochs * (epoch_length + 1))
		);

		let block_number = get_current_block_as_u64::<T>();

		// shift node classes
    Network::<T>::shift_node_classes(block_number, epoch_length);

		let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;

		let proposer = funded_account::<T>("subnet_node", 0);
		let defendant = funded_account::<T>("subnet_node", 1);

		let proposal_bid_amount = ProposalBidAmount::<T>::get();
    let plaintiff_starting_balance = T::Currency::free_balance(&proposer.clone());
		let data = Vec::new();

		let accountant_nodes = SubnetNodesClasses::<T>::get(subnet_id, SubnetNodeClass::Accountant);

		assert_ok!(
			Network::<T>::propose(
				RawOrigin::Signed(proposer.clone()).into(), 
				subnet_id, 
				peer(1), 
				data.clone()
			)
		);

		#[extrinsic_call]
		cancel_proposal(RawOrigin::Signed(proposer.clone()), subnet_id, 0);

    let plaintiff_after_balance = T::Currency::free_balance(&proposer.clone());
    assert_eq!(plaintiff_starting_balance, plaintiff_after_balance);

    let proposal = Proposals::<T>::try_get(subnet_id, 0);
    assert_eq!(proposal, Err(()));
	}

	#[benchmark]
	fn challenge_proposal() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;

		let epoch_length = T::EpochLength::get();

		let min_required_subnet_consensus_submit_epochs: u64 = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();	
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epoch_length * min_required_subnet_consensus_submit_epochs)
		);

		let n_peers: u32 = Network::<T>::max_subnet_nodes();

    let amount: u128 = 				 1000000000000000000000;

		build_subnet_nodes::<T>(subnet_id, 0, n_peers, amount);

		let epochs = SubnetNodeClassEpochs::<T>::get(SubnetNodeClass::Accountant);
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epochs * (epoch_length + 1))
		);

		let block_number = get_current_block_as_u64::<T>();

		// shift node classes
    Network::<T>::shift_node_classes(block_number, epoch_length);

		let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;

		let proposer = funded_account::<T>("subnet_node", 0);
		let defendant = funded_account::<T>("subnet_node", 1);

		let proposal_bid_amount = ProposalBidAmount::<T>::get();
    let plaintiff_starting_balance = T::Currency::free_balance(&proposer.clone());
		let data = Vec::new();

		let accountant_nodes = SubnetNodesClasses::<T>::get(subnet_id, SubnetNodeClass::Accountant);

		assert_ok!(
			Network::<T>::propose(
				RawOrigin::Signed(proposer.clone()).into(), 
				subnet_id, 
				peer(1), 
				data.clone()
			)
		);

		let challenger_starting_balance = T::Currency::free_balance(&defendant.clone());

		#[extrinsic_call]
		challenge_proposal(RawOrigin::Signed(defendant.clone()), subnet_id, 0, Vec::new());

    let challenger_after_balance = T::Currency::free_balance(&defendant.clone());
    assert_eq!(challenger_starting_balance - u128_to_balance::<T>(proposal_bid_amount).unwrap(), challenger_after_balance);

		let proposal = Proposals::<T>::get(subnet_id, 0);
    assert_eq!(proposal.subnet_id, subnet_id);
    assert_eq!(proposal.plaintiff, proposer.clone());
    assert_eq!(proposal.defendant, defendant.clone());
    assert_eq!(proposal.plaintiff_bond, proposal_bid_amount);
    assert_eq!(proposal.defendant_bond, proposal_bid_amount);
    assert_eq!(proposal.eligible_voters.len(), accountant_nodes.len());
    assert_eq!(proposal.start_block, get_current_block_as_u64::<T>());
    assert_eq!(proposal.challenge_block, get_current_block_as_u64::<T>());
    assert_eq!(proposal.plaintiff_data, data);
    assert_eq!(proposal.defendant_data, data);
    assert_eq!(proposal.complete, false);
	}

	#[benchmark]
	fn vote() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;

		let epoch_length = T::EpochLength::get();

		let min_required_subnet_consensus_submit_epochs: u64 = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();	
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epoch_length * min_required_subnet_consensus_submit_epochs)
		);

		let n_peers: u32 = Network::<T>::max_subnet_nodes();

    let amount: u128 = 				 1000000000000000000000;

		build_subnet_nodes::<T>(subnet_id, 0, n_peers, amount);

		let epochs = SubnetNodeClassEpochs::<T>::get(SubnetNodeClass::Accountant);
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epochs * (epoch_length + 1))
		);

		let block_number = get_current_block_as_u64::<T>();

		// shift node classes
    Network::<T>::shift_node_classes(block_number, epoch_length);

		let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;

		let proposer = funded_account::<T>("subnet_node", 0);
		let defendant = funded_account::<T>("subnet_node", 1);

		let proposal_bid_amount = ProposalBidAmount::<T>::get();
    let plaintiff_starting_balance = T::Currency::free_balance(&proposer.clone());
		let data = Vec::new();

		let accountant_nodes = SubnetNodesClasses::<T>::get(subnet_id, SubnetNodeClass::Accountant);

		assert_ok!(
			Network::<T>::propose(
				RawOrigin::Signed(proposer.clone()).into(), 
				subnet_id, 
				peer(1), 
				data.clone()
			)
		);

		assert_ok!(
			Network::<T>::challenge_proposal(
				RawOrigin::Signed(defendant.clone()).into(), 
				subnet_id, 
				0, 
				data.clone()
			)
		);

		let voter = funded_account::<T>("subnet_node", 2);

		#[extrinsic_call]
		vote(RawOrigin::Signed(voter.clone()), subnet_id, 0, VoteType::Yay);

    let proposal = Proposals::<T>::get(subnet_id, 0);
    assert_eq!(proposal.votes.yay.get(&voter.clone()), Some(&voter.clone()));
    assert_ne!(proposal.votes.yay.get(&voter.clone()), None);
	}

	#[benchmark]
	fn finalize_proposal() {
		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;

		let epoch_length = T::EpochLength::get();

		let min_required_subnet_consensus_submit_epochs: u64 = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();	
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epoch_length * min_required_subnet_consensus_submit_epochs)
		);

		let n_peers: u32 = Network::<T>::max_subnet_nodes();

    let amount: u128 = 				 1000000000000000000000;

		build_subnet_nodes::<T>(subnet_id, 0, n_peers, amount);

		let epochs = SubnetNodeClassEpochs::<T>::get(SubnetNodeClass::Accountant);
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epochs * (epoch_length + 1))
		);

		let block_number = get_current_block_as_u64::<T>();

		// shift node classes
    Network::<T>::shift_node_classes(block_number, epoch_length);

		let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;

		let proposer = funded_account::<T>("subnet_node", 0);
		let defendant = funded_account::<T>("subnet_node", 1);

		let proposal_bid_amount = ProposalBidAmount::<T>::get();
    let plaintiff_starting_balance = T::Currency::free_balance(&proposer.clone());
		let data = Vec::new();

		let accountant_nodes = SubnetNodesClasses::<T>::get(subnet_id, SubnetNodeClass::Accountant);

		assert_ok!(
			Network::<T>::propose(
				RawOrigin::Signed(proposer.clone()).into(), 
				subnet_id, 
				peer(1), 
				data.clone()
			)
		);

		assert_ok!(
			Network::<T>::challenge_proposal(
				RawOrigin::Signed(defendant.clone()).into(), 
				subnet_id, 
				0, 
				data.clone()
			)
		);

		for n in 0..n_peers {
      if n == 0 || n == 1 {
        continue
      }
			let voter = funded_account::<T>("subnet_node", n);
			assert_ok!(
				Network::<T>::vote(
					RawOrigin::Signed(voter.clone()).into(), 
					subnet_id, 
					0, 
					VoteType::Yay
				)
			);
    }

		let voting_period = VotingPeriod::<T>::get();
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(voting_period + 1)
		);

		// anone can call this
		let finalizer = funded_account::<T>("subnet_node", n_peers);

		#[extrinsic_call]
		finalize_proposal(RawOrigin::Signed(finalizer.clone()), subnet_id, 0);

		let plaintiff_after_balance = T::Currency::free_balance(&proposer.clone());
    assert!(plaintiff_after_balance > plaintiff_starting_balance);

    let proposal = Proposals::<T>::get(subnet_id, 0);
    assert_eq!(proposal.votes.yay.len(), (n_peers-2) as usize);
    assert_eq!(proposal.plaintiff_bond, 0);
    assert_eq!(proposal.defendant_bond, 0);
    assert_eq!(proposal.complete, true);
	}

	#[benchmark]
	fn reward_subnet() {
		let rewarder = funded_account::<T>("rewarder", 0);

		let n_peers: u32 = Network::<T>::max_subnet_nodes();
    let amount: u128 = 				 1000000000000000000000;

		build_subnet::<T>(DEFAULT_SUBNET_PATH.into());
		let subnet_id = 1;

		build_subnet_nodes::<T>(subnet_id, 0, n_peers, amount);

		let epoch_length = T::EpochLength::get();

		let min_required_subnet_consensus_submit_epochs: u64 = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();	
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epoch_length * min_required_subnet_consensus_submit_epochs)
		);

		let class_epochs = SubnetNodeClassEpochs::<T>::get(SubnetNodeClass::Submittable);
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(class_epochs * (epoch_length + 1))
		);

		let block_number = get_current_block_as_u64::<T>();
    Network::<T>::shift_node_classes(block_number, epoch_length);
		let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;

		// --- get validator
    Network::<T>::do_choose_validator_and_accountants(
			get_current_block_as_u64::<T>(), 
			epoch as u32, 
			epoch_length
		);

		let validator = SubnetRewardsValidator::<T>::get(subnet_id, epoch as u32);
		assert!(validator != None, "Validator is None");

		let subnet_node_data_vec = subnet_node_data(0, n_peers);

		// --- validate
		assert_ok!(
			Network::<T>::validate(
				RawOrigin::Signed(validator.clone().unwrap()).into(), 
				subnet_id, 
				subnet_node_data_vec.clone()
			)
		);

		// Attest
		for n in 1..n_peers {
			let attester = funded_account::<T>("subnet_node", n);
			assert_ok!(
				Network::<T>::attest(
					RawOrigin::Signed(attester).into(), 
					subnet_id,
				)
			);
		}	

		// --- push to next epoch block where ``on_initialize`` is called		
		let curr_block_number = get_current_block_as_u64::<T>();
		let next_epoch_block = curr_block_number - (curr_block_number % epoch_length) + epoch_length;

		frame_system::Pallet::<T>::set_block_number(u64_to_block::<T>(next_epoch_block + 1));

		
		#[extrinsic_call]
		reward_subnet(RawOrigin::Signed(rewarder.clone()), subnet_id, epoch as u32);

		for n in 1..n_peers {
			let subnet_node = funded_account::<T>("subnet_node", n);
			let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node.clone(), subnet_id);
			assert!(account_subnet_stake > amount);
		}	
	}

	// #[benchmark]
	// fn on_initialize_reward_subnets() {
	// 	let max_subnets: u32 = Network::<T>::max_subnets();
	// 	let n_peers: u32 = Network::<T>::max_subnet_nodes();
  //   let amount: u128 = 				 1000000000000000000000;

	// 	for s in 0..max_subnets {
	// 		let subnet_path = format!("subnet-path-{s}");
	// 		build_subnet::<T>(subnet_path.clone().into());

	// 		let subnet_id = s+1;
	// 		build_subnet_nodes::<T>(subnet_id, 0, n_peers, amount);
	// 	}

	// 	let epoch_length = T::EpochLength::get();

	// 	let min_required_subnet_consensus_submit_epochs: u64 = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();	
	// 	frame_system::Pallet::<T>::set_block_number(
	// 		frame_system::Pallet::<T>::block_number() + 
	// 		u64_to_block::<T>(epoch_length * min_required_subnet_consensus_submit_epochs)
	// 	);

	// 	let class_epochs = SubnetNodeClassEpochs::<T>::get(SubnetNodeClass::Submittable);
	// 	frame_system::Pallet::<T>::set_block_number(
	// 		frame_system::Pallet::<T>::block_number() + 
	// 		u64_to_block::<T>(class_epochs * (epoch_length + 1))
	// 	);

	// 	let block_number = get_current_block_as_u64::<T>();
  //   Network::<T>::shift_node_classes(block_number, epoch_length);
	// 	let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;

	// 	// --- get validator
  //   Network::<T>::do_choose_validator_and_accountants(
	// 		get_current_block_as_u64::<T>(), 
	// 		epoch as u32, 
	// 		epoch_length
	// 	);

	// 	for s in 0..max_subnets {
	// 		let subnet_id = s+1;

	// 		let validator = SubnetRewardsValidator::<T>::get(subnet_id, epoch as u32);
	// 		assert!(validator != None, "Validator is None");
	
	// 		let subnet_node_data_vec = subnet_node_data(0, n_peers);
	
	// 		// --- validate
	// 		assert_ok!(
	// 			Network::<T>::validate(
	// 				RawOrigin::Signed(validator.clone().unwrap()).into(), 
	// 				subnet_id, 
	// 				subnet_node_data_vec.clone()
	// 			)
	// 		);
	
	// 		// Attest
	// 		for n in 1..n_peers {
	// 			let attester = funded_account::<T>("subnet_node", n);
	// 			assert_ok!(
	// 				Network::<T>::attest(
	// 					RawOrigin::Signed(attester).into(), 
	// 					subnet_id,
	// 				)
	// 			);
	// 		}	
	// 	}

	// 	// --- push to next epoch block where ``on_initialize`` is called		
	// 	let curr_block_number = get_current_block_as_u64::<T>();
	// 	let next_epoch_block = curr_block_number - (curr_block_number % epoch_length) + epoch_length;

	// 	frame_system::Pallet::<T>::set_block_number(u64_to_block::<T>(next_epoch_block));

	// 	#[block]
	// 	{
	// 		let block = get_current_block_as_u64::<T>();
	// 		let epoch_length = T::EpochLength::get();
	// 		let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;
	// 		Network::<T>::reward_subnets(block, (epoch - 1) as u32, epoch_length);
	// 		Network::<T>::shift_node_classes(block_number, epoch_length);
	// 	}

	// 	// ensure nodes were rewarded
	// 	for s in 0..max_subnets {
	// 		let subnet_id = s+1;

	// 		for n in 1..n_peers {
	// 			let subnet_node = funded_account::<T>("subnet_node", n);
	// 			let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node.clone(), subnet_id);
	// 			assert!(account_subnet_stake > amount);
	// 		}	
	// 	}
	// }

	// #[benchmark]
	// fn on_initialize_reward_subnets_subnet_nodes_removed_linear(
	// 	x: Linear<1, { Network::<T>::max_subnets() }>, 
	// 	y: Linear<{ get_min_subnet_nodes::<T>() }, { Network::<T>::max_subnet_nodes() }>
	// ) {
	// 	let mut max_subnets: u32 = x;
	// 	let mut n_peers: u32 = y;
  //   let amount: u128 = 				 1000000000000000000000;

	// 	for s in 0..max_subnets {
	// 		let subnet_path = format!("subnet-path-{s}");
	// 		build_subnet::<T>(subnet_path.clone().into());

	// 		let subnet_id = s+1;
	// 		build_subnet_nodes::<T>(subnet_id, 0, n_peers, amount);
	// 	}

	// 	let epoch_length = T::EpochLength::get();

	// 	let min_required_subnet_consensus_submit_epochs: u64 = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();	
	// 	frame_system::Pallet::<T>::set_block_number(
	// 		frame_system::Pallet::<T>::block_number() + 
	// 		u64_to_block::<T>(epoch_length * min_required_subnet_consensus_submit_epochs)
	// 	);

	// 	let class_epochs = SubnetNodeClassEpochs::<T>::get(SubnetNodeClass::Submittable);
	// 	frame_system::Pallet::<T>::set_block_number(
	// 		frame_system::Pallet::<T>::block_number() + 
	// 		u64_to_block::<T>(class_epochs * (epoch_length + 1))
	// 	);

	// 	let block_number = get_current_block_as_u64::<T>();
  //   Network::<T>::shift_node_classes(block_number, epoch_length);
	// 	let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;

	// 	// --- get validator
  //   Network::<T>::do_choose_validator_and_accountants(
	// 		get_current_block_as_u64::<T>(), 
	// 		epoch as u32, 
	// 		epoch_length
	// 	);

	// 	for s in 0..max_subnets {
	// 		let subnet_id = s+1;

	// 		let validator = SubnetRewardsValidator::<T>::get(subnet_id, epoch as u32);
	// 		assert!(validator != None, "Validator is None");
	
	// 		let subnet_node_data_vec = subnet_node_data(0, n_peers);
	
	// 		// --- validate
	// 		assert_ok!(
	// 			Network::<T>::validate(
	// 				RawOrigin::Signed(validator.clone().unwrap()).into(), 
	// 				subnet_id, 
	// 				subnet_node_data_vec.clone()
	// 			)
	// 		);
	
	// 		// Attest
	// 		for n in 1..n_peers {
	// 			let attester = funded_account::<T>("subnet_node", n);
	// 			assert_ok!(
	// 				Network::<T>::attest(
	// 					RawOrigin::Signed(attester).into(), 
	// 					subnet_id,
	// 				)
	// 			);
	// 		}	
	// 	}

	// 	// --- push to next epoch block where ``on_initialize`` is called		
	// 	let curr_block_number = get_current_block_as_u64::<T>();
	// 	let next_epoch_block = curr_block_number - (curr_block_number % epoch_length) + epoch_length;

	// 	frame_system::Pallet::<T>::set_block_number(u64_to_block::<T>(next_epoch_block));

	// 	#[block]
	// 	{
	// 		let block = get_current_block_as_u64::<T>();
	// 		let epoch_length = T::EpochLength::get();
	// 		let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;
	// 		Network::<T>::reward_subnets(block, (epoch - 1) as u32, epoch_length);
	// 		Network::<T>::shift_node_classes(block_number, epoch_length);
	// 	}

	// 	// ensure nodes were rewarded
	// 	for s in 0..max_subnets {
	// 		let subnet_id = s+1;

	// 		for n in 1..n_peers {
	// 			let subnet_node = funded_account::<T>("subnet_node", n);
	// 			let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node.clone(), subnet_id);
	// 			assert!(account_subnet_stake > amount);
	// 		}	
	// 	}
	// }

	// #[benchmark]
	// fn on_initialize_reward_subnets_linear(
	// 	x: Linear<1, { Network::<T>::max_subnets() }>, 
	// 	y: Linear<{ get_min_subnet_nodes::<T>() }, { Network::<T>::max_subnet_nodes() }>
	// ) {
	// 	let mut max_subnets: u32 = x;
	// 	let mut n_peers: u32 = y;
  //   let amount: u128 = 				 1000000000000000000000;

	// 	for s in 0..max_subnets {
	// 		let subnet_path = format!("subnet-path-{s}");
	// 		build_subnet::<T>(subnet_path.clone().into());

	// 		let subnet_id = s+1;
	// 		build_subnet_nodes::<T>(subnet_id, 0, n_peers, amount);
	// 	}

	// 	let epoch_length = T::EpochLength::get();

	// 	let min_required_subnet_consensus_submit_epochs: u64 = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();	
	// 	frame_system::Pallet::<T>::set_block_number(
	// 		frame_system::Pallet::<T>::block_number() + 
	// 		u64_to_block::<T>(epoch_length * min_required_subnet_consensus_submit_epochs)
	// 	);

	// 	let class_epochs = SubnetNodeClassEpochs::<T>::get(SubnetNodeClass::Submittable);
	// 	frame_system::Pallet::<T>::set_block_number(
	// 		frame_system::Pallet::<T>::block_number() + 
	// 		u64_to_block::<T>(class_epochs * (epoch_length + 1))
	// 	);

	// 	let block_number = get_current_block_as_u64::<T>();
  //   Network::<T>::shift_node_classes(block_number, epoch_length);
	// 	let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;

	// 	// --- get validator
  //   Network::<T>::do_choose_validator_and_accountants(
	// 		get_current_block_as_u64::<T>(), 
	// 		epoch as u32, 
	// 		epoch_length
	// 	);

	// 	for s in 0..max_subnets {
	// 		let subnet_id = s+1;

	// 		let validator = SubnetRewardsValidator::<T>::get(subnet_id, epoch as u32);
	// 		assert!(validator != None, "Validator is None");
	
	// 		let subnet_node_data_vec = subnet_node_data(0, n_peers);
	
	// 		// --- validate
	// 		assert_ok!(
	// 			Network::<T>::validate(
	// 				RawOrigin::Signed(validator.clone().unwrap()).into(), 
	// 				subnet_id, 
	// 				subnet_node_data_vec.clone()
	// 			)
	// 		);
	
	// 		// Attest
	// 		for n in 1..n_peers {
	// 			let attester = funded_account::<T>("subnet_node", n);
	// 			assert_ok!(
	// 				Network::<T>::attest(
	// 					RawOrigin::Signed(attester).into(), 
	// 					subnet_id,
	// 				)
	// 			);
	// 		}	
	// 	}

	// 	// --- push to next epoch block where ``on_initialize`` is called		
	// 	let curr_block_number = get_current_block_as_u64::<T>();
	// 	let next_epoch_block = curr_block_number - (curr_block_number % epoch_length) + epoch_length;

	// 	frame_system::Pallet::<T>::set_block_number(u64_to_block::<T>(next_epoch_block));

	// 	#[block]
	// 	{
	// 		let block = get_current_block_as_u64::<T>();
	// 		let epoch_length = T::EpochLength::get();
	// 		let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;
	// 		Network::<T>::reward_subnets(block, (epoch - 1) as u32, epoch_length);
	// 		Network::<T>::shift_node_classes(block_number, epoch_length);
	// 	}

	// 	// ensure nodes were rewarded
	// 	for s in 0..max_subnets {
	// 		let subnet_id = s+1;

	// 		for n in 1..n_peers {
	// 			let subnet_node = funded_account::<T>("subnet_node", n);
	// 			let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node.clone(), subnet_id);
	// 			assert!(account_subnet_stake > amount);
	// 		}	
	// 	}
	// }

	#[benchmark]
	fn on_initialize_do_choose_validator_and_accountants() {
		let max_subnets: u32 = Network::<T>::max_subnets();
		let n_peers: u32 = Network::<T>::max_subnet_nodes();
    let amount: u128 = 				 1000000000000000000000;

		for s in 0..max_subnets {
			let subnet_path = format!("subnet-path-{s}");
			build_subnet::<T>(subnet_path.clone().into());

			let subnet_id = s+1;
			build_subnet_nodes::<T>(subnet_id, 0, n_peers, amount);
		}

		let epoch_length = T::EpochLength::get();

		let min_required_subnet_consensus_submit_epochs: u64 = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();	
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(epoch_length * min_required_subnet_consensus_submit_epochs)
		);

		let class_epochs = SubnetNodeClassEpochs::<T>::get(SubnetNodeClass::Submittable);
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 
			u64_to_block::<T>(class_epochs * (epoch_length + 1))
		);

		let block_number = get_current_block_as_u64::<T>();
    Network::<T>::shift_node_classes(block_number, epoch_length);
		let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;

		#[block]
		{
			let block = get_current_block_as_u64::<T>();
			let epoch_length = T::EpochLength::get();
			let epoch = get_current_block_as_u64::<T>() / epoch_length as u64;
			Network::<T>::do_choose_validator_and_accountants(
				block, 
				epoch as u32, 
				epoch_length
			);
		}

		// ensure nodes were rewarded
		for s in 0..max_subnets {
			let subnet_id = s+1;

			let validator = SubnetRewardsValidator::<T>::get(subnet_id, epoch as u32);
			assert!(validator != None, "Validator is None");
		}
	}

	#[benchmark]
	fn on_initialize() {
		// get to a block where none of the functions will be ran
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + u64_to_block::<T>(1)
		);

		#[block]
		{
			let block = frame_system::Pallet::<T>::block_number();
			Network::<T>::on_initialize(block);
		}
	}

	impl_benchmark_test_suite!(Network, crate::mock::new_test_ext(), crate::mock::Test);
}
