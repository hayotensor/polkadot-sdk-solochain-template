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
use super::Event;
// use frame_support::traits::OriginTrait;
use sp_core::OpaquePeerId as PeerId;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use sp_runtime::traits::Header;
use log::info;
use sp_core::{H256, U256};
// use parity_scale_codec::Decode;
use frame_support::traits::{OnInitialize, Currency};
use crate::{
  Error, SubnetNodeData, AccountPenaltyCount, TotalStake, 
  SubnetPaths, MinRequiredUnstakeEpochs, MaxAccountPenaltyCount, MinSubnetNodes, TotalSubnetNodes,
  TotalActiveSubnetNodes,
  SubnetNodesData, SubnetNodeAccount, SubnetNodeClass,
  SubnetsData,
  AccountSubnetStake, MinStakeBalance,
  VotingPeriod, Proposals, ProposalsCount, ChallengePeriod, VoteType,
  AccountSubnetDelegateStakeShares, TotalSubnetDelegateStakeShares, TotalSubnetDelegateStakeBalance,
  TotalSubnets, AccountantDataCount,
  AccountantDataNodeParams, SubnetRewardsValidator, SubnetRewardsSubmission, BaseValidatorReward,
  DelegateStakeRewardsPercentage,
  SubnetPenaltyCount, MaxSequentialAbsentSubnetNode, MaxSubnetNodePenalties, 
  SubnetNodePenalties, RegistrationSubnetData,
  CurrentAccountants, TargetAccountantsLength, MinRequiredSubnetConsensusSubmitEpochs, BaseRewardPerMB,
  DelegateStakeUnbondingLedger, SubnetRemovalReason, ProposalBidAmount, BaseSubnetNodeMemoryMB,
  MinSubnetDelegateStakePercentage, MaxSubnetPenaltyCount,
  TotalAccountStake, MaxSubnetMemoryMB, SubnetStakeUnbondingLedger,
  TotalSubnetStake, MinSubnetRegistrationBlocks, MaxSubnetRegistrationBlocks, SubnetActivationEnactmentPeriod,
};
use frame_support::BoundedVec;
use strum::IntoEnumIterator;
use sp_io::crypto::sr25519_sign;
use sp_runtime::{MultiSigner, MultiSignature};
use sp_io::crypto::sr25519_generate;
use frame_support::pallet_prelude::Encode;
use sp_runtime::traits::IdentifyAccount;
use sp_core::Pair;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

type AccountIdOf<Test> = <Test as frame_system::Config>::AccountId;
// type PeerIdOf<Test> = PeerId;

fn account(id: u32) -> AccountIdOf<Test> {
	[id as u8; 32].into()
}

// it is possible to use `use libp2p::PeerId;` with `PeerId::random()`
// https://github.com/paritytech/substrate/blob/033d4e86cc7eff0066cd376b9375f815761d653c/frame/node-authorization/src/mock.rs#L90
// fn peer(id: u8) -> PeerId {
// 	PeerId(vec![id])
// }

fn peer(id: u32) -> PeerId {
   
	// let peer_id = format!("12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA{id}");
  let peer_id = format!("QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N{id}"); 
	PeerId(peer_id.into())
}
// bafzbeie5745rpv2m6tjyuugywy4d5ewrqgqqhfnf445he3omzpjbx5xqxe
// QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N
// 12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA

fn get_min_stake_balance() -> u128 {
	MinStakeBalance::<Test>::get()
}

const PERCENTAGE_FACTOR: u128 = 10000;
const DEFAULT_SCORE: u128 = 5000;
const CONSENSUS_STEPS: u64 = 2;
const DEFAULT_MEM_MB: u128 = 50000;
const MAX_SUBNET_NODES: u32 = 254;
const DEFAULT_REGISTRATION_BLOCKS: u64 = 130_000;

// fn build_subnet(subnet_path: Vec<u8>) {
//   // assert_ok!(
//   //   Network::vote_subnet(
//   //     RuntimeOrigin::signed(account(0)), 
//   //     subnet_path.clone(),
//   //   )
//   // );
//   let cost = Network::get_subnet_initialization_cost(0);
//   let _ = Balances::deposit_creating(&account(0), cost+1000);

//   let add_subnet_data = RegistrationSubnetData {
//     path: subnet_path.clone().into(),
//     memory_mb: DEFAULT_MEM_MB,
//     registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
//   };
//   assert_ok!(
//     Network::activate_subnet(
//       account(0),
//       account(0),
//       add_subnet_data,
//     )
//   );

//   // assert_ok!(
//   //   Network::add_subnet(
//   //     RuntimeOrigin::signed(account(0)),
//   //     add_subnet_data,
//   //   ) 
//   // );
// }

fn build_activated_subnet(subnet_path: Vec<u8>, start: u32, mut end: u32, deposit_amount: u128, amount: u128) {
  System::set_block_number(System::block_number() + 1);

  let cost = Network::get_subnet_initialization_cost(0);
  let _ = Balances::deposit_creating(&account(0), cost+1000);

  let add_subnet_data = RegistrationSubnetData {
    path: subnet_path.clone().into(),
    memory_mb: DEFAULT_MEM_MB,
    registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
  };

  // --- Register subnet for activation
  assert_ok!(
    Network::register_subnet(
      RuntimeOrigin::signed(account(0)),
      add_subnet_data,
    )
  );

  let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
  let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();

  let min_nodes = subnet.min_nodes;

  if end == 0 {
    end = min_nodes;
  }

  let epoch_length = EpochLength::get();
  let epoch = System::block_number() / epoch_length;

  // --- Add subnet nodes
  let block_number = System::block_number();
  let mut amount_staked = 0;
  for n in start..end {
    let _ = Balances::deposit_creating(&account(n), deposit_amount);
    amount_staked += amount;
    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(n)),
        subnet_id,
        peer(n),
        amount,
        None,
        None,
        None,
      ) 
    );

    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, account(n)).unwrap();
    assert_eq!(subnet_node_data.account_id, account(n));
    assert_eq!(subnet_node_data.hotkey, account(n));
    assert_eq!(subnet_node_data.peer_id, peer(n));
    assert_eq!(subnet_node_data.initialized, block_number);
    // --- Is ``Submittable`` if registered before subnet activation
    assert_eq!(subnet_node_data.classification.class, SubnetNodeClass::Submittable);
    assert!(subnet_node_data.has_classification(&SubnetNodeClass::Submittable, epoch));

    let subnet_node_account = SubnetNodeAccount::<Test>::get(subnet_id, peer(n));
    assert_eq!(subnet_node_account, account(n));

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(n), subnet_id);
    assert_eq!(account_subnet_stake, amount);

    // let total_account_stake = TotalAccountStake::<Test>::get(account(n));
    // assert_eq!(total_account_stake, amount);
  }

  let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
  assert_eq!(total_subnet_stake, amount_staked);

  let total_stake = TotalStake::<Test>::get();
  assert_eq!(total_subnet_stake, amount_staked);


  let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance(min_nodes);
  // --- Add the minimum required delegate stake balance to activate the subnet
  assert_ok!(
    Network::add_to_delegate_stake(
      RuntimeOrigin::signed(account(1)),
      subnet_id,
      min_subnet_delegate_stake,
    ) 
  );

  let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(1), subnet_id);
  // 1000 is for inflation attack mitigation
  assert_eq!(min_subnet_delegate_stake - 1000, delegate_shares);

  // --- Increase blocks to max registration block
  System::set_block_number(System::block_number() + subnet.registration_blocks + 1);
  let current_block_number = System::block_number();
  
  assert_ok!(
    Network::activate_subnet(
      RuntimeOrigin::signed(account(0)),
      subnet_id,
    )
  );

  // --- Check validator chosen on activation
  // let next_epoch = System::block_number() / epoch_length + 1;
  // let validator = SubnetRewardsValidator::<Test>::get(subnet_id, next_epoch as u32);
  // assert!(validator != None, "Validator is None");
}

// Returns total staked on subnet
fn build_subnet_nodes(subnet_id: u32, start: u32, end: u32, deposit_amount: u128, amount: u128) -> u128 {
  let mut amount_staked = 0;
  for n in start..end {
    let _ = Balances::deposit_creating(&account(n), deposit_amount);
    amount_staked += amount;
    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(n)),
        subnet_id,
        peer(n),
        amount,
        None,
        None,
        None,
      ) 
    );
    post_successful_add_subnet_node_asserts(n, subnet_id, amount);
  }
  amount_staked
}

fn post_subnet_removal_ensures(subnet_id: u32, start: u32, end: u32) {
  let subnet = SubnetsData::<Test>::try_get(subnet_id);
  assert_eq!(subnet, Err(()));

  for n in start..end {
    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, account(n));
    assert_eq!(subnet_node_data, Err(()));
  
    // ensure SubnetNodeAccount removed
    let subnet_node_account = SubnetNodeAccount::<Test>::try_get(subnet_id, peer(n));
    assert_eq!(subnet_node_account, Err(()));
  
    // let subnet_accounts = SubnetAccount::<Test>::get(subnet_id);
    // let subnet_account = subnet_accounts.get(&account(n));
    // assert_eq!(subnet_accounts.get(&account(n)), Some(&System::block_number()));
  
    let stake_balance = AccountSubnetStake::<Test>::get(account(n), subnet_id);
    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(n)),
        subnet_id,
        stake_balance,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n), subnet_id);
    if delegate_shares != 0 {
      assert_ok!(
        Network::remove_delegate_stake(
          RuntimeOrigin::signed(account(n)),
          subnet_id,
          delegate_shares,
        )
      );  
    }
  }

  let epoch_length = EpochLength::get();
  let stake_cooldown_epochs = StakeCooldownEpochs::get();

  let starting_block_number = System::block_number();

  

  // --- Ensure unstaking is stable
  for n in start..end {
    System::set_block_number(System::block_number() + ((epoch_length  + 1) * stake_cooldown_epochs));
    let starting_balance = Balances::free_balance(&account(n));
    let unbondings = SubnetStakeUnbondingLedger::<Test>::get(account(n), subnet_id);
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_ok!(
      Network::claim_stake_unbondings(
        RuntimeOrigin::signed(account(n)),
        subnet_id,
      )
    );
    let ending_balance = Balances::free_balance(&account(n));
    assert_eq!(starting_balance + ledger_balance, ending_balance);
    System::set_block_number(starting_block_number);
  }

  for n in start..end {
    System::set_block_number(System::block_number() + ((epoch_length  + 1) * DelegateStakeCooldownEpochs::get()));
    let starting_balance = Balances::free_balance(&account(n));

    let unbondings = DelegateStakeUnbondingLedger::<Test>::get(account(n), subnet_id);
    if unbondings.len() == 0 {
      continue
    }
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();

    assert_ok!(
      Network::claim_delegate_stake_unbondings(
        RuntimeOrigin::signed(account(n)),
        subnet_id,
      )
    );

    let ending_balance = Balances::free_balance(&account(n));
    assert_eq!(starting_balance + ledger_balance, ending_balance);
    System::set_block_number(starting_block_number);
  }
}

// fn build_for_submit_consensus_data(subnet_id: u32, start: u32, end: u32, start_data: u32, end_data: u32) {
//   let subnet_node_data_vec = subnet_node_data(start_data, end_data);

//   for n in start..end {
//     assert_ok!(
//       Network::submit_consensus_data(
//         RuntimeOrigin::signed(account(n)),
//         subnet_id,
//         subnet_node_data_vec.clone(),
//       ) 
//     );
//   }
// }

fn increase_epochs(epochs: u32) {
  let block = System::block_number();
  let epoch_length = EpochLength::get();

  // epoch_length + block - (block % epoch_length)

  let next_epoch_start_block = (epoch_length * epochs as u64) + block - (block % (epoch_length * epochs as u64));
  System::set_block_number(next_epoch_start_block);
}

fn make_subnet_submittable() {
  // increase blocks
  // let epoch_length = Network::EpochLength::get();
  let epoch_length = EpochLength::get();
  

  let min_required_subnet_consensus_submit_epochs: u64 = MinRequiredSubnetConsensusSubmitEpochs::<Test>::get();
  System::set_block_number(System::block_number() + epoch_length * min_required_subnet_consensus_submit_epochs);
}

// // increase the blocks past the consensus steps and remove subnet peer blocks span
// fn make_consensus_data_submittable() {
//   // increase blocks
//   let current_block_number = System::block_number();
//   // let subnet_node_removal_percentage = RemoveSubnetNodeEpochPercentage::<Test>::get();
//   let epoch_length = EpochLength::get();

//   let start_block_can_remove_peer = epoch_length as u128 * subnet_node_removal_percentage / PERCENTAGE_FACTOR;

//   let max_remove_subnet_node_block = start_block_can_remove_peer as u64 + (current_block_number - (current_block_number % epoch_length));

//   if current_block_number < max_remove_subnet_node_block {
//     System::set_block_number(max_remove_subnet_node_block + 1);
//   }
// }

// fn make_subnet_node_included() {
//   let epoch_length = EpochLength::get();
// 	let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Included);
//   System::set_block_number(System::block_number() + epoch_length * epochs);
// }

// fn make_subnet_node_consensus_data_submittable() {
//   // increase blocks
//   let epoch_length = EpochLength::get();
// 	let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Submittable);
//   System::set_block_number(System::block_number() + epoch_length * epochs);
//   // make_consensus_data_submittable();
// }

// fn make_subnet_node_dishonesty_consensus_proposable() {
//   // increase blocks
//   let epoch_length = EpochLength::get();
// 	let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//   System::set_block_number(System::block_number() + epoch_length * epochs);
// }

// fn make_subnet_node_removable() {
//   // increase blocks
//   let current_block_number = System::block_number();
//   let subnet_node_removal_percentage = RemoveSubnetNodeEpochPercentage::<Test>::get();
//   let epoch_length = EpochLength::get();

//   let block_span_can_remove_peer = (epoch_length as u128 * subnet_node_removal_percentage / PERCENTAGE_FACTOR) as u64;

//   let start_removal_block = (CONSENSUS_STEPS + (current_block_number - (current_block_number % epoch_length))) as u64;

//   let end_removal_block = block_span_can_remove_peer + (current_block_number - (current_block_number % epoch_length));
  
//   if current_block_number < start_removal_block {
//     System::set_block_number(start_removal_block);
//   } else if current_block_number > end_removal_block {
//     System::set_block_number(start_removal_block + epoch_length);
//   }
// }

fn subnet_node_data(start: u32, end: u32) -> Vec<SubnetNodeData> {
  // initialize peer consensus data array
  let mut subnet_node_data: Vec<SubnetNodeData> = Vec::new();
  for n in start..end {
    // let peer_subnet_node_data: SubnetNodeData<<Test as frame_system::Config>::AccountId> = SubnetNodeData {
    //   // account_id: account(n),
    //   peer_id: peer(n),
    //   score: DEFAULT_SCORE,
    // };
    let peer_subnet_node_data: SubnetNodeData = SubnetNodeData {
      peer_id: peer(n),
      score: DEFAULT_SCORE,
    };
    subnet_node_data.push(peer_subnet_node_data);
  }
  subnet_node_data
}

fn subnet_node_data_invalid_scores(start: u32, end: u32) -> Vec<SubnetNodeData> {
  // initialize peer consensus data array
  // let mut subnet_node_data: Vec<SubnetNodeData<<Test as frame_system::Config>::AccountId>> = Vec::new();
  let mut subnet_node_data: Vec<SubnetNodeData> = Vec::new();
  for n in start..end {
    // let peer_subnet_node_data: SubnetNodeData<<Test as frame_system::Config>::AccountId> = SubnetNodeData {
    //   // account_id: account(n),
    //   peer_id: peer(n),
    //   score: 10000000000,
    // };
    let peer_subnet_node_data: SubnetNodeData = SubnetNodeData {
      peer_id: peer(n),
      score: 10000000000,
    };
    subnet_node_data.push(peer_subnet_node_data);
  }
  subnet_node_data
}

fn post_successful_add_subnet_node_asserts(
  n: u32, 
  subnet_id: u32, 
  amount: u128
) {
  assert_eq!(Network::account_subnet_stake(account(n), subnet_id), amount);
  assert_eq!(Network::total_account_stake(account(n)), amount);    
  assert_eq!(Network::total_subnet_nodes(subnet_id), (n + 1) as u32);
}

// check data after adding multiple peers
// each peer must have equal staking amount per subnet
fn post_successful_add_subnet_nodes_asserts(
  total_peers: u32,
  stake_per_peer: u128,  
  subnet_id: u32, 
) {
  let amount_staked = total_peers as u128 * stake_per_peer;
  assert_eq!(Network::total_subnet_stake(subnet_id), amount_staked);
}

fn post_remove_subnet_node_ensures(n: u32, subnet_id: u32) {
  // ensure SubnetNodesData removed
  let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, account(n));
  assert_eq!(subnet_node_data, Err(()));

  // ensure SubnetNodeAccount removed
  let subnet_node_account = SubnetNodeAccount::<Test>::try_get(subnet_id, peer(n));
  assert_eq!(subnet_node_account, Err(()));

  // // ensure SubnetNodeConsensusResults removed
  // let subnet_node_consensus_results = SubnetNodeConsensusResults::<Test>::try_get(subnet_id, account(n));
  // assert_eq!(subnet_node_consensus_results, Err(()));

  // // ensure SubnetAccount u64 updated to current block
  // let subnet_accounts = SubnetAccount::<Test>::get(subnet_id);
  // let subnet_account = subnet_accounts.get(&account(n));
  // assert_eq!(subnet_accounts.get(&account(n)), Some(&System::block_number()));

  // for class_id in SubnetNodeClass::iter() {
  //   let node_sets = SubnetNodesClasses::<Test>::get(subnet_id, class_id);
  //   assert_eq!(node_sets.get(&account(n)), None);
  // }
}

fn post_remove_unstake_ensures(n: u32, subnet_id: u32) {
  // ensure SubnetAccount is removed after unstaking to 0
  // let subnet_accounts = SubnetAccount::<Test>::get(subnet_id);
  // let subnet_account = subnet_accounts.get(&account(n));
  // assert_eq!(subnet_accounts.get(&account(n)), None);
}

// The following should be ensured after form_consensus is rate
// This should work regardless if there are consensus issues or not
fn post_successful_form_consensus_ensures(subnet_id: u32) {
  // let peer_consensus_epoch_submitted = NodeConsensusEpochSubmitted::<Test>::iter().count();
  // assert_eq!(peer_consensus_epoch_submitted, 0);
  // let peer_consensus_epoch_confirmed = NodeConsensusEpochUnconfirmed::<Test>::iter().count();
  // assert_eq!(peer_consensus_epoch_confirmed, 0);
  // let subnet_total_consensus_submits = SubnetTotalConsensusSubmits::<Test>::iter().count();
  // assert_eq!(subnet_total_consensus_submits, 0);
  // let subnet_consensus_epoch_unconfirmed_count = SubnetConsensusEpochUnconfirmedCount::<Test>::try_get(subnet_id);
  // assert_eq!(subnet_consensus_epoch_unconfirmed_count, Err(()));
}

fn post_successful_generate_emissions_ensures() {
  // let subnets_in_consensus = SubnetsInConsensus::<Test>::try_get();
  // assert_eq!(subnets_in_consensus, Err(()));

  // let subnets_in_consensus = SubnetsInConsensus::<Test>::get();
  // assert_eq!(subnets_in_consensus.len(), 0);


  // let subnet_node_consensus_results = SubnetNodeConsensusResults::<Test>::iter().count();
  // assert_eq!(subnet_node_consensus_results, 0);
}

fn add_subnet_node(
  account_id: u32, 
  subnet_id: u32,
  peer_id: u32,
  ip: String,
  port: u16,
  amount: u128
) -> Result<(), sp_runtime::DispatchError> {
  Network::add_subnet_node(
    RuntimeOrigin::signed(account(account_id)),
    subnet_id,
    peer(peer_id),
    amount,
    None,
    None,
    None,
  )
}

///
///
///
///
///
///
///
/// Subnets Add/Remove
///
///
///
///
///
///
///

#[test]
fn test_register_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };
  
    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let min_nodes = subnet.min_nodes;
  
  })
}

#[test]
fn test_register_subnet_exists_error() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };
  
    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data.clone(),
      )
    );
  
    assert_err!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data.clone(),
      ),
      Error::<Test>::SubnetExist
    );

  })
}

#[test]
fn test_register_subnet_registration_blocks_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: MinSubnetRegistrationBlocks::<Test>::get() - 1,
    };
    
    assert_err!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      ),
      Error::<Test>::InvalidSubnetRegistrationBlocks
    );

    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: MaxSubnetRegistrationBlocks::<Test>::get() + 1,
    };
    
    assert_err!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      ),
      Error::<Test>::InvalidSubnetRegistrationBlocks
    );
  })
}

#[test]
fn test_register_subnet_max_subnet_mem_err() {
  new_test_ext().execute_with(|| {
    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000);

    let max_subnet_mem = MaxSubnetMemoryMB::<Test>::get();
  
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.into(),
      memory_mb: max_subnet_mem+1,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };

    assert_err!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      ),
      Error::<Test>::MaxSubnetMemory
    );
  })
}

#[test]
fn test_register_subnet_not_enough_balance_err() {
  new_test_ext().execute_with(|| {
    // let cost = Network::get_subnet_initialization_cost(0);
    // let _ = Balances::deposit_creating(&account(0), cost+1000);  
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };

    assert_err!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      ),
      Error::<Test>::NotEnoughBalanceToStake
    );
  })
}

#[test]
fn test_activate_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };
  
    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = subnet.min_nodes;
		let target_nodes = subnet.target_nodes;
		let memory_mb = subnet.memory_mb;
		let initialized = subnet.initialized;
		let registration_blocks = subnet.registration_blocks;
		let activated = subnet.activated;

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 0..min_nodes {
      let _ = Balances::deposit_creating(&account(n), deposit_amount);
      assert_ok!(
        Network::add_subnet_node(
          RuntimeOrigin::signed(account(n)),
          subnet_id,
          peer(n),
          amount,
          None,
          None,
          None,  
        ) 
      );
    }
  
    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance(min_nodes);
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        min_subnet_delegate_stake,
      ) 
    );

    // --- Increase blocks to max registration block
    System::set_block_number(System::block_number() + subnet.registration_blocks + 1);
    let current_block_number = System::block_number();

    assert_ok!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      )
    );

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet.id, subnet_id);

    // ensure subnet exists and nothing changed but the activation block
    assert_eq!(subnet.id, id);
    assert_eq!(subnet.path, path);
    assert_eq!(subnet.min_nodes, min_nodes);
    assert_eq!(subnet.target_nodes, target_nodes);
    assert_eq!(subnet.memory_mb, memory_mb);
    assert_eq!(subnet.initialized, initialized);
    assert_eq!(subnet.registration_blocks, registration_blocks);
    // ensure activated block updated
    assert_eq!(subnet.activated, current_block_number);
  })
}

#[test]
fn test_activate_subnet_invalid_subnet_id_error() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };
  
    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = subnet.min_nodes;
		let target_nodes = subnet.target_nodes;
		let memory_mb = subnet.memory_mb;
		let initialized = subnet.initialized;
		let registration_blocks = subnet.registration_blocks;
		let activated = subnet.activated;

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 0..min_nodes {
      let _ = Balances::deposit_creating(&account(n), deposit_amount);
      assert_ok!(
        Network::add_subnet_node(
          RuntimeOrigin::signed(account(n)),
          subnet_id,
          peer(n),
          amount,
          None,
          None,
          None,  
        ) 
      );
    }
  
    assert_err!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id+1,
      ),
      Error::<Test>::InvalidSubnetId
    );
  })
}

#[test]
fn test_activate_subnet_already_activated_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };
  
    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = subnet.min_nodes;
		let target_nodes = subnet.target_nodes;
		let memory_mb = subnet.memory_mb;
		let initialized = subnet.initialized;
		let registration_blocks = subnet.registration_blocks;
		let activated = subnet.activated;

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 0..min_nodes {
      let _ = Balances::deposit_creating(&account(n), deposit_amount);
      assert_ok!(
        Network::add_subnet_node(
          RuntimeOrigin::signed(account(n)),
          subnet_id,
          peer(n),
          amount,
          None,
          None,
          None,  
        ) 
      );
    }
  
    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance(min_nodes);
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        min_subnet_delegate_stake,
      ) 
    );

    // --- Increase blocks to max registration block
    System::set_block_number(System::block_number() + subnet.registration_blocks + 1);
    let current_block_number = System::block_number();

    assert_ok!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      )
    );

    assert_err!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      ),
      Error::<Test>::SubnetActivatedAlready
    );
  })
}

#[test]
fn test_activate_subnet_enactment_period_remove_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };
  
    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();

    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = subnet.min_nodes;
		let target_nodes = subnet.target_nodes;
		let memory_mb = subnet.memory_mb;
		let initialized = subnet.initialized;
		let registration_blocks = subnet.registration_blocks;
		let activated = subnet.activated;

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 0..min_nodes {
      let _ = Balances::deposit_creating(&account(n), deposit_amount);
      assert_ok!(
        Network::add_subnet_node(
          RuntimeOrigin::signed(account(n)),
          subnet_id,
          peer(n),
          amount,
          None,
          None,
          None,  
        ) 
      );
    }
  
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance(min_nodes);
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        min_subnet_delegate_stake,
      ) 
    );

    // --- Increase blocks to max registration block
    System::set_block_number(System::block_number() + subnet.registration_blocks + SubnetActivationEnactmentPeriod::<Test>::get() + 1);
    let current_block_number = System::block_number();

    assert_ok!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      )
    );

    assert_eq!(
			*network_events().last().unwrap(),
			Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::EnactmentPeriod
      }
		);

    let removed_subnet_id = SubnetPaths::<Test>::try_get(subnet_path.clone());
    assert_eq!(removed_subnet_id, Err(()));
    let subnet = SubnetsData::<Test>::try_get(subnet_id);
    assert_eq!(subnet, Err(()));

    // --- Ensure nodes can be removed and unstake
    post_subnet_removal_ensures(subnet_id, 0, total_subnet_nodes);
  })
}


#[test]
fn test_activate_subnet_initializing_error() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };
  
    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = subnet.min_nodes;
		let target_nodes = subnet.target_nodes;
		let memory_mb = subnet.memory_mb;
		let initialized = subnet.initialized;
		let registration_blocks = subnet.registration_blocks;
		let activated = subnet.activated;

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 0..min_nodes {
      let _ = Balances::deposit_creating(&account(n), deposit_amount);
      assert_ok!(
        Network::add_subnet_node(
          RuntimeOrigin::signed(account(n)),
          subnet_id,
          peer(n),
          amount,
          None,
          None,
          None,  
        ) 
      );
    }
  
    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance(min_nodes);
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        min_subnet_delegate_stake,
      ) 
    );

    // --- Increase blocks to max registration block
    // System::set_block_number(System::block_number() + subnet.registration_blocks + 1);
    // let current_block_number = System::block_number();

    assert_err!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      ),
      Error::<Test>::SubnetInitializing
    );
  })
}

#[test]
fn test_activate_subnet_min_subnet_nodes_remove_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };
  
    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = subnet.min_nodes;
		let target_nodes = subnet.target_nodes;
		let memory_mb = subnet.memory_mb;
		let initialized = subnet.initialized;
		let registration_blocks = subnet.registration_blocks;
		let activated = subnet.activated;

    // --- Increase blocks to max registration block
    System::set_block_number(System::block_number() + subnet.registration_blocks + 1);
    let current_block_number = System::block_number();

    assert_ok!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      )
    );

    assert_eq!(
			*network_events().last().unwrap(),
			Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::MinSubnetNodes
      }
		);

    let removed_subnet_id = SubnetPaths::<Test>::try_get(subnet_path.clone());
    assert_eq!(removed_subnet_id, Err(()));
    let subnet = SubnetsData::<Test>::try_get(subnet_id);
    assert_eq!(subnet, Err(()));
  })
}

#[test]
fn test_activate_subnet_min_delegate_balance_remove_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };
  
    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = subnet.min_nodes;
		let target_nodes = subnet.target_nodes;
		let memory_mb = subnet.memory_mb;
		let initialized = subnet.initialized;
		let registration_blocks = subnet.registration_blocks;
		let activated = subnet.activated;

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 0..min_nodes {
      let _ = Balances::deposit_creating(&account(n), deposit_amount);
      assert_ok!(
        Network::add_subnet_node(
          RuntimeOrigin::signed(account(n)),
          subnet_id,
          peer(n),
          amount,
          None,
          None,
          None,  
        ) 
      );
    }
  
    // --- Increase blocks to max registration block
    System::set_block_number(System::block_number() + subnet.registration_blocks + 1);
    let current_block_number = System::block_number();

    assert_ok!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      )
    );

    assert_eq!(
			*network_events().last().unwrap(),
			Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::MinSubnetDelegateStake
      }
		);

    let removed_subnet_id = SubnetPaths::<Test>::try_get(subnet_path.clone());
    assert_eq!(removed_subnet_id, Err(()));
    let subnet = SubnetsData::<Test>::try_get(subnet_id);
    assert_eq!(subnet, Err(()));
  })
}

// #[test]
// fn test_add_subnet() {
//   new_test_ext().execute_with(|| {

//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     assert_eq!(Network::total_subnets(), 1);

//     let subnet_path_2: Vec<u8> = "petals-team-2/StableBeluga2".into();

//     build_subnet(subnet_path_2.clone());

//     assert_eq!(Network::total_subnets(), 2);

//   })
// }

// #[test]
// fn test_add_subnet_err() {
//   new_test_ext().execute_with(|| {

//     // let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     // let add_subnet_data = RegistrationSubnetData {
//     //   path: subnet_path.clone().into(),
//     //   memory_mb: DEFAULT_MEM_MB,
//     // };
  
//     // assert_err!(
//     //   Network::add_subnet(
//     //     RuntimeOrigin::signed(account(0)),
//     //     add_subnet_data.clone(),
//     //   ),
//     //   Error::<Test>::SubnetNotVotedIn
//     // );

//     // build_subnet(subnet_path.clone());

//     // assert_eq!(Network::total_subnets(), 1);

//     // assert_err!(
//     //   Network::add_subnet(
//     //     RuntimeOrigin::signed(account(0)),
//     //     add_subnet_data.clone(),
//     //   ),
//     //   Error::<Test>::SubnetExist
//     // );
//   })
// }

// #[test]
// fn test_remove_subnet() {
//   new_test_ext().execute_with(|| {

//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     assert_eq!(Network::total_subnets(), 1);
//     // let add_subnet_data = RegistrationSubnetData {
//     //   path: subnet_path.clone().into(),
//     //   memory_mb: DEFAULT_MEM_MB,
//     // };
//     assert_ok!(
//       Network::deactivate_subnet(
//         subnet_path.clone().into(),
//         SubnetRemovalReason::SubnetDemocracy,
//       )
//     );

//     // Total subnets should stay constant as its an index value
//     assert_eq!(Network::total_subnets(), 1);
//   })
// }

// #[test]
// fn test_remove_subnet_subnet_initializing() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
//     build_subnet(subnet_path.clone());
//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     assert_eq!(Network::total_subnets(), subnet_id);

//     assert_err!(
//       Network::remove_subnet(
//         RuntimeOrigin::signed(account(0)),
//         255,
//       ),
//       Error::<Test>::SubnetNotExist
//     );

//     assert_err!(
//       Network::remove_subnet(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//       ),
//       Error::<Test>::SubnetInitializing
//     );

//   })
// }

// #[test]
// fn test_get_min_subnet_delegate_stake_balance() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
//     // build_subnet(subnet_path.clone());
//     let cost = Network::get_subnet_initialization_cost(0);
//     let _ = Balances::deposit_creating(&account(0), cost+1000);
  
//     let add_subnet_data = RegistrationSubnetData {
//       path: subnet_path.clone().into(),
//       memory_mb: 500_000,
//       registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
//     };
//     assert_ok!(
//       Network::activate_subnet(
//         account(0),
//         account(0),
//         add_subnet_data,
//       )
//     );
  
//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
//     let min_stake_balance = get_min_stake_balance();
//     let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
//     let min_subnet_delegate_stake_percentage = MinSubnetDelegateStakePercentage::<Test>::get();

//     let subnet_min_stake_supply = min_stake_balance * subnet.min_nodes as u128;
//     let presumed_min = Network::percent_mul(subnet_min_stake_supply, min_subnet_delegate_stake_percentage);

//     let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance(subnet.min_nodes);

//     assert_eq!(presumed_min, min_subnet_delegate_stake);
//   })
// }

// #[test]
// fn test_remove_subnet_min_delegate_stake_balance_met() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
//     build_subnet(subnet_path.clone());
    
//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();

//     let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance(subnet.min_nodes);


//     let _ = Balances::deposit_creating(&account(1), min_subnet_delegate_stake + 500);
//     let starting_delegator_balance = Balances::free_balance(&account(1));

//     assert_ok!(
//       Network::add_to_delegate_stake(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         min_subnet_delegate_stake,
//       ) 
//     );

//     let epoch_length = EpochLength::get();
//     let min_required_subnet_consensus_submit_epochs = MinRequiredSubnetConsensusSubmitEpochs::<Test>::get();

//     System::set_block_number(System::block_number() + min_required_subnet_consensus_submit_epochs * (epoch_length + 1));

//     assert_err!(
//       Network::remove_subnet(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//       ),
//       Error::<Test>::SubnetMinDelegateStakeBalanceMet
//     );
//   })
// }

// #[test]
// fn test_remove_subnet_below_min_delegate_stake_balance() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
//     build_subnet(subnet_path.clone());
//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    
//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();

//     let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance(subnet.min_nodes);


//     let _ = Balances::deposit_creating(&account(1), min_subnet_delegate_stake + 500);
//     let starting_delegator_balance = Balances::free_balance(&account(1));

//     assert_ok!(
//       Network::add_to_delegate_stake(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         min_subnet_delegate_stake,
//       ) 
//     );

//     let epoch_length = EpochLength::get();
//     let min_required_subnet_consensus_submit_epochs = MinRequiredSubnetConsensusSubmitEpochs::<Test>::get();

//     System::set_block_number(System::block_number() + min_required_subnet_consensus_submit_epochs * (epoch_length + 1));

//     let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(1), subnet_id);

//     assert_ok!(
//       Network::remove_delegate_stake(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         delegate_shares,
//       ) 
//     );

//     assert_ok!(
//       Network::remove_subnet(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//       )
//     );
//   })
// }

// // #[test]
// // fn test_add_subnet_max_subnets_err() {
// //   new_test_ext().execute_with(|| {
// //     let n_subnets: u32 = Network::max_subnets() + 1;

// //     for m in 0..n_subnets {
// //       let subnet_path = format!("petals-team-{m}/StableBeluga");
// //       let subnet_path_2 = format!("petals-team-{m}/StableBeluga2");
// //       let add_subnet_data = RegistrationSubnetData {
// //         path: subnet_path.clone().into(),
// //         memory_mb: DEFAULT_MEM_MB,
// //       };
  
// //       assert_ok!(
// //         Network::activate_subnet(
// //           account(0),
// //           account(0),
// //           add_subnet_data,
// //         )
// //       );
  
// //       // assert_ok!(
// //       //   Network::vote_subnet(
// //       //     RuntimeOrigin::signed(account(0)), 
// //       //     subnet_path.clone().into(),
// //       //   )
// //       // );
// //       let add_subnet_data = RegistrationSubnetData {
// //         path: subnet_path.clone().into(),
// //         memory_mb: DEFAULT_MEM_MB,
// //       };

// //       if m+1 < n_subnets {
// //         assert_ok!(
// //           Network::activate_subnet(
// //             account(0),
// //             account(0),
// //             add_subnet_data.clone(),
// //           )
// //         );  
// //         // assert_ok!(
// //         //   Network::add_subnet(
// //         //     RuntimeOrigin::signed(account(0)),
// //         //     add_subnet_data.clone()
// //         //   ) 
// //         // );
// //       } else {
// //         assert_err!(
// //           Network::activate_subnet(
// //             account(0),
// //             account(0),
// //             add_subnet_data.clone(),
// //           ),
// //           Error::<Test>::MaxSubnets
// //         );  
// //       }
// //     }
// //   })
// // }

///
///
///
///
///
///
///
/// Subnet Nodes Add/Remove
///
///
///
///
///
///
///

// #[test]
// fn test_add_subnet_node_max_peers_err() {
//   new_test_ext().execute_with(|| {
//     let mut n_peers: u32 = Network::max_subnet_nodes() + 1;

//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     let mut total_staked: u128 = 0;

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     for n in 0..n_peers {
//       let _ = Balances::deposit_creating(&account(n), deposit_amount);

//       if n+1 < n_peers {
//         total_staked += amount;
//         assert_ok!(
//           add_subnet_node(
//             n, 
//             subnet_id,
//             n,
//             "172.20.54.234".into(),
//             8888,
//             amount
//           )  
//         );

//         // assert_ok!(
//         //   Network::add_subnet_node(
//         //     RuntimeOrigin::signed(account(n)),
//         //     subnet_id,
//         //     peer(n),
//         //     amount,
//         //   ) 
//         // );
//         assert_eq!(Network::total_subnet_nodes(1), (n + 1) as u32);
//         assert_eq!(Network::account_subnet_stake(account(n), 1), amount);
//         assert_eq!(Network::total_account_stake(account(n)), amount);
//       } else {
//         assert_err!(
//           Network::add_subnet_node(
//             RuntimeOrigin::signed(account(n)),
//             subnet_id,
//             peer(n),
//             amount,
//           ),
//           Error::<Test>::SubnetNodesMax
//         );
//       }
//     }

//     assert_eq!(Network::total_stake(), total_staked);
//     assert_eq!(Network::total_subnet_stake(1), total_staked);
//     assert_eq!(TotalSubnetNodes::<Test>::get(1), n_peers-1);
//   });
// }

#[test]
fn test_register_subnet_node() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      )
    );

    let new_total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(new_total_subnet_nodes, total_subnet_nodes + 1);

    let new_total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(new_total_active_subnet_nodes, total_active_subnet_nodes);

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, account(total_subnet_nodes+1));
    assert_eq!(subnet_node.account_id, account(total_subnet_nodes+1));
    assert_eq!(subnet_node.hotkey, account(total_subnet_nodes+1));
    assert_eq!(subnet_node.peer_id, peer(total_subnet_nodes+1));
    assert_eq!(subnet_node.initialized, 0);
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Registered);

    let subnet_node_account = SubnetNodeAccount::<Test>::get(subnet_id, peer(total_subnet_nodes+1));
    assert_eq!(subnet_node_account, account(total_subnet_nodes+1));

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    assert_eq!(account_subnet_stake, amount);
  })
}

#[test]
fn test_register_subnet_node_subnet_registering_or_activated_error() {
  new_test_ext().execute_with(|| {

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };
  
    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    
    log::error!("subnet.activated {:?}",subnet.activated );
    log::error!("subnet.initialized {:?}",subnet.initialized );
    log::error!("subnet.registration_blocks {:?}",subnet.registration_blocks );

    System::set_block_number(System::block_number() + subnet.initialized + subnet.registration_blocks + 1);
  
    log::error!("subnet block_number {:?}", System::block_number() );

    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
        peer(0),
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::SubnetMustBeRegisteringOrActivated
    );
  })
}

#[test]
fn test_activate_subnet_node_subnet_registering_or_activated_error() {
  new_test_ext().execute_with(|| {

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000+deposit_amount);
  
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };
  
    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
        peer(0),
        amount,
        None,
        None,
        None,
      )
    );

    System::set_block_number(System::block_number() + DEFAULT_REGISTRATION_BLOCKS + 1);

    assert_err!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      ),
      Error::<Test>::SubnetMustBeRegisteringOrActivated
    );
  })
}


#[test]
fn test_register_subnet_node_activate_subnet_node() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      )
    );

    let new_total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(new_total_subnet_nodes, total_subnet_nodes + 1);

    let new_total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(new_total_active_subnet_nodes, total_active_subnet_nodes);

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, account(total_subnet_nodes+1));
    assert_eq!(subnet_node.account_id, account(total_subnet_nodes+1));
    assert_eq!(subnet_node.hotkey, account(total_subnet_nodes+1));
    assert_eq!(subnet_node.peer_id, peer(total_subnet_nodes+1));
    assert_eq!(subnet_node.initialized, 0);
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Registered);

    let subnet_node_account = SubnetNodeAccount::<Test>::get(subnet_id, peer(total_subnet_nodes+1));
    assert_eq!(subnet_node_account, account(total_subnet_nodes+1));

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    assert_eq!(account_subnet_stake, amount);

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number() / epoch_length;

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
      )
    );

    let new_new_total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(new_new_total_active_subnet_nodes, new_total_active_subnet_nodes + 1);

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, account(total_subnet_nodes+1));

    assert_eq!(subnet_node.initialized, block_number);
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Idle);
    assert_eq!(subnet_node.classification.start_epoch, epoch + 1);
  })
}

// #[test]
// fn test_deactivate_subnet_node() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//     let total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);

//     let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

//     assert_ok!(
//       Network::register_subnet_node(
//         RuntimeOrigin::signed(account(total_subnet_nodes+1)),
//         subnet_id,
//         peer(total_subnet_nodes+1),
//         amount,
//         None,
//         None,
//         None,
//       )
//     );

//     let new_total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//     assert_eq!(new_total_subnet_nodes, total_subnet_nodes + 1);

//     let new_total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
//     assert_eq!(new_total_active_subnet_nodes, total_active_subnet_nodes);

//     let subnet_node = SubnetNodesData::<Test>::get(subnet_id, account(total_subnet_nodes+1));
//     assert_eq!(subnet_node.account_id, account(total_subnet_nodes+1));
//     assert_eq!(subnet_node.hotkey, account(total_subnet_nodes+1));
//     assert_eq!(subnet_node.peer_id, peer(total_subnet_nodes+1));
//     assert_eq!(subnet_node.initialized, 0);
//     assert_eq!(subnet_node.classification.class, SubnetNodeClass::Registered);

//     let subnet_node_account = SubnetNodeAccount::<Test>::get(subnet_id, peer(total_subnet_nodes+1));
//     assert_eq!(subnet_node_account, account(total_subnet_nodes+1));

//     let account_subnet_stake = AccountSubnetStake::<Test>::get(account(total_subnet_nodes+1), subnet_id);
//     assert_eq!(account_subnet_stake, amount);

//     let epoch_length = EpochLength::get();
//     let block_number = System::block_number();
//     let epoch = System::block_number() / epoch_length;

//     assert_ok!(
//       Network::activate_subnet_node(
//         RuntimeOrigin::signed(account(total_subnet_nodes+1)),
//         subnet_id,
//       )
//     );

//     let new_new_total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
//     assert_eq!(new_new_total_active_subnet_nodes, new_total_active_subnet_nodes + 1);

//     let subnet_node = SubnetNodesData::<Test>::get(subnet_id, account(total_subnet_nodes+1));

//     assert_eq!(subnet_node.initialized, block_number);
//     assert_eq!(subnet_node.classification.class, SubnetNodeClass::Idle);
//     assert_eq!(subnet_node.classification.start_epoch, epoch + 1);


//     assert_ok!(
//       Network::deactivate_subnet_node(
//         RuntimeOrigin::signed(account(total_subnet_nodes+1)),
//         subnet_id,
//       )
//     );

//     let new_new_new_total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
//     assert_eq!(new_new_new_total_active_subnet_nodes, new_new_total_active_subnet_nodes - 1);

//   })
// }

#[test]
fn test_add_subnet_node_subnet_err() {
  new_test_ext().execute_with(|| {
    let subnet_id = 0;

    let amount: u128 = 1000;
    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
        peer(0),
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::SubnetNotExist
    );

    let subnet_id = 1;

    assert_err!(Network::add_subnet_node(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
        peer(0),
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::SubnetNotExist
    );
  })
}

#[test]
fn test_get_classification_subnet_nodes() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;
  
    let submittable = Network::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Submittable, epoch as u64);

    assert_eq!(submittable.len() as u32, total_subnet_nodes);
  })
}

#[test]
fn test_add_subnet_node_not_exists_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // add new peer_id under same account error
    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
        peer(1),
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::SubnetNodeExist
    );

    assert_eq!(Network::total_subnet_nodes(subnet_id), total_subnet_nodes);

    // add same peer_id under new account error
    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(0),
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::PeerIdExist
    );

    assert_eq!(Network::total_subnet_nodes(subnet_id), total_subnet_nodes);

    // add new peer_id under same account error
    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
        peer(1),
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::SubnetNodeExist
    );

    assert_eq!(Network::total_subnet_nodes(subnet_id), total_subnet_nodes);
  })
}

#[test]
fn test_add_subnet_node_stake_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let deposit_amount: u128 = 100000;
    let amount: u128 = 1;

    let _ = Balances::deposit_creating(&account(0), deposit_amount);
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::MinStakeNotReached
    );
  })
}

#[test]
fn test_add_subnet_node_stake_not_enough_balance_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let deposit_amount: u128 = 999999999999999999999;

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::NotEnoughBalanceToStake
    );
  })
}

#[test]
fn test_add_subnet_node_invalid_peer_id_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let peer_id = format!("2");
    let peer: PeerId = PeerId(peer_id.into());
    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::InvalidPeerId
    );
  })
}

// #[test]
// fn test_add_subnet_node_remove_readd_err() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

//     System::set_block_number(System::block_number() + 1);

//     assert_ok!(
//       Network::add_subnet_node(
//         RuntimeOrigin::signed(account(total_subnet_nodes+1)),
//         subnet_id,
//         peer(total_subnet_nodes+1),
//         amount,
//       )
//     );

//     assert_ok!(
//       Network::remove_subnet_node(
//         RuntimeOrigin::signed(account(total_subnet_nodes+1)),
//         subnet_id,
//       )
//     );

//     assert_err!(
//       Network::add_subnet_node(
//         RuntimeOrigin::signed(account(total_subnet_nodes+1)),
//         subnet_id,
//         peer(total_subnet_nodes+1),
//         amount,
//       ), 
//       Error::<Test>::RequiredUnstakeEpochsNotMet
//     );
//   });
// }

#[test]
fn test_add_subnet_node_remove_readd() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let deposit_amount: u128 = 1000000000000000000000000;

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      )
    );

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
      )
    );

    let account_subnet_stake = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account_subnet_stake,
      )
    );

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      )
    );
  });
}

#[test]
fn test_add_subnet_node_remove_readd_must_unstake_error() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let deposit_amount: u128 = 1000000000000000000000000;

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      )
    );

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
      )
    );

    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::InvalidSubnetRegistrationCooldown
    );
  });
}

#[test]
fn test_add_subnet_node_remove_stake_partial_readd() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let deposit_amount: u128 = 1000000000000000000000000;

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      )
    );

    // increase account subnet stake to simulate rewards
    AccountSubnetStake::<Test>::insert(&account(total_subnet_nodes+1), subnet_id, amount + 100);

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
      )
    );

    // once blocks have been increased, account can either remove stake in part or in full or readd subnet peer
    let epoch_length = EpochLength::get();
    let min_required_unstake_epochs = MinRequiredUnstakeEpochs::<Test>::get();

    System::set_block_number(System::block_number() + epoch_length * min_required_unstake_epochs);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account_subnet_stake,
      )
    );

    // should be able to readd after unstaking
    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      ) 
    );
  });
}

#[test]
fn test_add_subnet_node_remove_stake_readd() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      )
    );

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
      )
    );

    // once blocks have been increased, account can either remove stake in part or in full or readd subnet peer
    let epoch_length = EpochLength::get();
    let min_required_unstake_epochs = MinRequiredUnstakeEpochs::<Test>::get();
    System::set_block_number(System::block_number() + epoch_length * min_required_unstake_epochs);

    let remaining_account_stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(0), subnet_id);

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        remaining_account_stake_balance,
      )
    );

    // should be able to readd after unstaking
    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      ) 
    );
  });
}

// #[test]
// fn test_remove_peer_error() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
//     let deposit_amount: u128 = 1000000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     assert_ok!(
//       Network::add_subnet_node(
//         RuntimeOrigin::signed(account(total_subnet_nodes+1)),
//         subnet_id,
//         peer(total_subnet_nodes+1),
//         amount,
//       ) 
//     );
//     // post_successful_add_subnet_node_asserts(0, subnet_id, amount);

//     // post_successful_add_subnet_nodes_asserts(
//     //   1,
//     //   amount,
//     //   subnet_id,
//     // );

//     // assert_eq!(Network::total_stake(), amount);

//     assert_err!(
//       Network::remove_subnet_node(
//         RuntimeOrigin::signed(account(total_subnet_nodes+1)),
//         subnet_id,
//       ),
//       Error::<Test>::SubnetNodeNotExist
//     );
//   });
// }

// // #[test]
// // fn test_remove_peer_unstake_epochs_err() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());
// //     let deposit_amount: u128 = 1000000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let _ = Balances::deposit_creating(&account(0), deposit_amount);

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let epoch_length = EpochLength::get();

// //     System::set_block_number(System::block_number() + epoch_length);

// //     assert_ok!(
// //       Network::add_subnet_node(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(0),
// //         amount,
// //       ) 
// //     );
// //     post_successful_add_subnet_node_asserts(0, subnet_id, amount);
// //     assert_eq!(Network::total_subnet_nodes(1), 1);
// //     assert_eq!(Network::account_subnet_stake(account(0), 1), amount);
// //     assert_eq!(Network::total_account_stake(account(0)), amount);
// //     assert_eq!(Network::total_stake(), amount);
// //     assert_eq!(Network::total_subnet_stake(1), amount);

// //     // make_subnet_node_removable();


// //     System::set_block_number(System::block_number() + epoch_length);

// //     assert_ok!(
// //       Network::remove_subnet_node(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //       ) 
// //     );

// //     post_remove_subnet_node_ensures(0, subnet_id);

// //     assert_eq!(Network::total_subnet_nodes(1), 0);

// //     assert_err!(
// //       Network::remove_stake(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         amount,
// //       ),
// //       Error::<Test>::RequiredUnstakeEpochsNotMet,
// //     );
    
// //     let epoch_length = EpochLength::get();
// //     let min_required_unstake_epochs = MinRequiredUnstakeEpochs::<Test>::get();
// //     System::set_block_number(System::block_number() + epoch_length * min_required_unstake_epochs);
    
// //     assert_ok!(
// //       Network::remove_stake(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         amount,
// //       )
// //     );
// //   });
// // }

#[test]
fn test_remove_peer_unstake_total_balance() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      ) 
    );
    // post_successful_add_subnet_node_asserts(0, subnet_id, amount);

    assert_eq!(Network::total_subnet_nodes(subnet_id), total_subnet_nodes+1);
    assert_eq!(Network::account_subnet_stake(account(0), subnet_id), amount);
    assert_eq!(Network::total_account_stake(account(0)), amount);
    assert_eq!(Network::total_stake(), amount * (total_subnet_nodes as u128 +1));
    assert_eq!(Network::total_subnet_stake(subnet_id), amount * (total_subnet_nodes as u128 +1));

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
      ) 
    );

    // post_remove_subnet_node_ensures(0, subnet_id);

    assert_eq!(Network::total_subnet_nodes(subnet_id), total_subnet_nodes);
    
    let epoch_length = EpochLength::get();
    let min_required_unstake_epochs = MinRequiredUnstakeEpochs::<Test>::get();
    System::set_block_number(System::block_number() + epoch_length * min_required_unstake_epochs);
    
    let remaining_account_stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        remaining_account_stake_balance,
      )
    );

    // post_remove_unstake_ensures(0, subnet_id);
  });
}

#[test]
fn test_claim_stake_unbondings() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let starting_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(starting_balance, deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      ) 
    );

    let stake_balance = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);
    assert_eq!(stake_balance, amount);

    let after_stake_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(after_stake_balance, starting_balance - amount);

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)), 
        subnet_id,
      ) 
    );

    let stake_balance = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);

    // remove amount ontop
    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        stake_balance,
      )
    );

    assert_eq!(Network::account_subnet_stake(account(total_subnet_nodes+1), 1), 0);
    assert_eq!(Network::total_account_stake(account(total_subnet_nodes+1)), 0);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let unbondings: BTreeMap<u64, u128> = SubnetStakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    assert_eq!(unbondings.len(), 1);
    let (first_key, first_value) = unbondings.iter().next().unwrap();
    assert_eq!(first_key, &epoch);
    assert!(*first_value <= stake_balance);

    let stake_cooldown_epochs = StakeCooldownEpochs::get();

    System::set_block_number(System::block_number() + ((epoch_length  + 1) * stake_cooldown_epochs));

    assert_ok!(
      Network::claim_stake_unbondings(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
      )
    );

    let post_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert_eq!(post_balance, starting_balance);

    let unbondings: BTreeMap<u64, u128> = SubnetStakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    assert_eq!(unbondings.len(), 0);
  });
}

#[test]
fn test_claim_stake_unbondings_no_unbondings_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let starting_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(starting_balance, deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount,
        None,
        None,
        None,
      ) 
    );

    let stake_balance = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);
    assert_eq!(stake_balance, amount);

    let after_stake_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(after_stake_balance, starting_balance - amount);

    assert_err!(
      Network::claim_stake_unbondings(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
      ),
      Error::<Test>::NoStakeUnbondingsOrCooldownNotMet
    );
  });
}

#[test]
fn test_remove_to_stake_max_unlockings_reached_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let starting_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        peer(total_subnet_nodes+1),
        amount*2,
        None,
        None,
        None,
      ) 
    );

    let max_unlockings = MaxStakeUnlockings::get();
    for n in 0..max_unlockings+1 {
      System::set_block_number(System::block_number() + EpochLength::get() + 1);
      if n+1 > max_unlockings {
        assert_err!(
          Network::remove_stake(
            RuntimeOrigin::signed(account(total_subnet_nodes+1)),
            subnet_id,
            1000,
          ),
          Error::<Test>::MaxUnlockingsReached
        );    
      } else {
        assert_ok!(
          Network::remove_stake(
            RuntimeOrigin::signed(account(total_subnet_nodes+1)),
            subnet_id,
            1000,
          )
        );
        let unbondings: BTreeMap<u64, u128> = SubnetStakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1), subnet_id);
        assert_eq!(unbondings.len() as u32, n+1);  
      }
    }
  });
}

#[test]
fn test_remove_subnet_node() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);
    let remove_n_peers = total_subnet_nodes / 2;

    let block_number = System::block_number();
    let epoch_length = EpochLength::get();
    let epoch = block_number / epoch_length;

    for n in 0..remove_n_peers {
      assert_ok!(
        Network::remove_subnet_node(
          RuntimeOrigin::signed(account(n)),
          subnet_id,
        ) 
      );
    }

    // let node_set = Network::get_classified_accounts(subnet_id, &SubnetNodeClass::Idle, epoch);
    let node_set: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_accounts(subnet_id, &SubnetNodeClass::Idle, epoch);

    assert_eq!(node_set.len(), (total_subnet_nodes - remove_n_peers) as usize);
    assert_eq!(Network::total_stake(), amount_staked);
    assert_eq!(Network::total_subnet_stake(subnet_id), amount_staked);
    assert_eq!(TotalSubnetNodes::<Test>::get(subnet_id), total_subnet_nodes - remove_n_peers);

    for n in 0..remove_n_peers {
      let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, account(n));
      assert_eq!(subnet_node_data, Err(()));

      let subnet_node_account = SubnetNodeAccount::<Test>::try_get(subnet_id, peer(n));
      assert_eq!(subnet_node_account, Err(()));
  
      let account_subnet_stake = AccountSubnetStake::<Test>::get(account(n), subnet_id);
      assert_eq!(account_subnet_stake, amount);
  
      let total_account_stake = TotalAccountStake::<Test>::get(account(n));
      assert_eq!(total_account_stake, amount);
    }

    let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
    assert_eq!(total_subnet_stake, amount_staked);

    let total_stake = TotalStake::<Test>::get();
    assert_eq!(total_subnet_stake, amount_staked);
  });
}

///
///
///
///
///
///
///
/// Staking
///
///
///
///
///
///
///

#[test]
fn test_add_to_stake_err() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let _ = Balances::deposit_creating(&account(0), deposit_amount);

    assert_err!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(0)),
        0,
        amount,
      ),
      Error::<Test>::SubnetNotExist,
    );

    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(0), deposit_amount);

    assert_err!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        amount,
      ),
      Error::<Test>::SubnetNodeNotExist,
    );

  });
}

#[test]
fn test_add_to_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(0), deposit_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
        amount,
      ) 
    );

    assert_eq!(Network::account_subnet_stake(account(0), subnet_id), amount + amount);
    assert_eq!(Network::total_account_stake(account(0)), amount + amount);
    assert_eq!(Network::total_stake(), amount_staked + amount);
    assert_eq!(Network::total_subnet_stake(subnet_id), amount_staked + amount);
  });
}

#[test]
fn test_remove_stake_err() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    // Not a node so should have no stake to remove
    assert_err!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(255)),
        0,
        amount,
      ),
      Error::<Test>::NotEnoughStakeToWithdraw,
    );

    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);

    assert_err!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(255)),
        subnet_id,
        amount,
      ),
      Error::<Test>::NotEnoughStakeToWithdraw,
    );

    assert_err!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
        0,
      ),
      Error::<Test>::NotEnoughStakeToWithdraw,
    );
  });
}

#[test]
fn test_remove_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let _ = Balances::deposit_creating(&account(0), deposit_amount);

    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    // add double amount to stake
    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
        amount,
      ) 
    );

    assert_eq!(Network::account_subnet_stake(account(0), subnet_id), amount + amount);
    assert_eq!(Network::total_account_stake(account(0)), amount + amount);

    // let epoch_length = EpochLength::get();
    // let min_required_unstake_epochs = MinRequiredUnstakeEpochs::<Test>::get();
    // System::set_block_number(System::block_number() + epoch_length * min_required_unstake_epochs);

    // remove amount ontop
    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
        amount,
      )
    );

    assert_eq!(Network::account_subnet_stake(account(0), subnet_id), amount);
    assert_eq!(Network::total_account_stake(account(0)), amount);
  });
}

// #[test]
// fn test_remove_stake_after_remove_subnet_node() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
//     let deposit_amount: u128 = 1000000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     let _ = Balances::deposit_creating(&account(0), deposit_amount);

//     assert_ok!(
//       Network::remove_subnet_node(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//       )
//     );

//     let epoch_length = EpochLength::get();
//     let min_required_unstake_epochs = MinRequiredUnstakeEpochs::<Test>::get();
//     System::set_block_number(System::block_number() + epoch_length * min_required_unstake_epochs);

//     // remove amount ontop
//     assert_ok!(
//       Network::remove_stake(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         amount,
//       )
//     );

//     assert_eq!(Network::account_subnet_stake(account(0), 1), 0);
//     assert_eq!(Network::total_account_stake(account(0)), 0);
//     assert_eq!(Network::total_stake(), 0);
//     assert_eq!(Network::total_subnet_stake(1), 0);
//   });
// }

///
///
///
///
///
///
///
/// Delegate staking
///
///
///
///
///
///
///

#[test]
fn test_delegate_math() {
  new_test_ext().execute_with(|| {
    let test1 = Network::convert_to_balance(
      1000000000000000000000,
      6000000000000000000000,
      6000000000000000000000
    );
    
    let test2 = Network::convert_to_balance(
      1000000000000000000000,
      6000000000000000000000,
      7000000000000000000000
    );

    assert_eq!(test1, 999999999000000000000);
    assert_eq!(test2, 1166666666000000000000);
  });
}

#[test]
fn test_remove_claim_delegate_stake_after_remove_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), amount + 500);
    let starting_delegator_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        amount,
      ) 
    );

    let post_delegator_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(post_delegator_balance, starting_delegator_balance - amount);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    // assert_eq!(amount, delegate_balance);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 9999)) &&
      (delegate_balance <= amount)
    );

    assert_ok!(
      Network::deactivate_subnet(
        subnet_path.clone().into(),
        SubnetRemovalReason::SubnetDemocracy,
      )
    );

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        delegate_shares,
      )
    );

    System::set_block_number(System::block_number() + ((EpochLength::get()  + 1) * DelegateStakeCooldownEpochs::get()));

    assert_ok!(
      Network::claim_delegate_stake_unbondings(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
      )
    );

    let post_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert!(
      (post_balance >= Network::percent_mul(starting_delegator_balance, 9999)) &&
      (post_balance <= starting_delegator_balance)
    );

    let unbondings: BTreeMap<u64, u128> = DelegateStakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    assert_eq!(unbondings.len(), 0);
  });
}

#[test]
fn test_add_to_delegate_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let post_delegator_balance = Balances::free_balance(&account(n_account));
    assert_eq!(post_delegator_balance, starting_delegator_balance - amount);

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    assert_eq!(amount + starting_total_subnet_delegated_stake_balance, total_subnet_delegated_stake_balance);

    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);

    assert!(
      (delegate_balance >= Network::percent_mul(amount, 9999)) &&
      (delegate_balance <= amount)
    );
  });
}

#[test]
fn test_add_to_delegate_stake_increase_pool_check_balance() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 9999)) &&
      (delegate_balance <= amount)
    );

    let increase_delegate_stake_amount: u128 = 1000000000000000000000;
    Network::do_increase_delegate_stake(
      subnet_id,
      increase_delegate_stake_amount,
    );

    // ensure balance has increase
    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    
    let post_delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    assert!(delegate_balance < post_delegate_balance);
    assert_ne!(delegate_balance, post_delegate_balance);
    assert!(
      (post_delegate_balance >= Network::percent_mul(amount + increase_delegate_stake_amount, 9999)) &&
      (post_delegate_balance <= amount + increase_delegate_stake_amount)
    );
  });
}

#[test]
fn test_claim_removal_of_delegate_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 9999)) &&
      (delegate_balance <= amount)
    );

    let epoch_length = EpochLength::get();
    let cooldown_epochs = DelegateStakeCooldownEpochs::get();

    System::set_block_number(System::block_number() + epoch_length * cooldown_epochs);

    let balance = Balances::free_balance(&account(n_account));
    let epoch = System::block_number() / epoch_length;

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        delegate_shares,
      )
    );
    let post_balance = Balances::free_balance(&account(n_account));
    assert_eq!(post_balance, balance);

    let unbondings: BTreeMap<u64, u128> = DelegateStakeUnbondingLedger::<Test>::get(account(n_account), subnet_id);
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(ledger_epoch, &epoch);
    assert!(*ledger_balance <= delegate_balance);

    assert_err!(
      Network::claim_delegate_stake_unbondings(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
      ),
      Error::<Test>::NoDelegateStakeUnbondingsOrCooldownNotMet
    );

    System::set_block_number(System::block_number() + ((epoch_length  + 1) * cooldown_epochs));

    let pre_claim_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::claim_delegate_stake_unbondings(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
      )
    );

    let after_claim_balance = Balances::free_balance(&account(n_account));

    assert_eq!(after_claim_balance, pre_claim_balance + *ledger_balance);

    assert!(
      (post_balance >= Network::percent_mul(starting_delegator_balance, 9999)) &&
      (post_balance <= starting_delegator_balance)
    );

    let unbondings: BTreeMap<u64, u128> = DelegateStakeUnbondingLedger::<Test>::get(account(n_account), subnet_id);
    assert_eq!(unbondings.len(), 0);
  });
}

#[test]
fn test_remove_to_delegate_stake_max_unlockings_per_epoch_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        delegate_shares/2,
      )
    );
    let unbondings: BTreeMap<u64, u128> = DelegateStakeUnbondingLedger::<Test>::get(account(n_account), subnet_id);
    assert_eq!(unbondings.len(), 1);

    assert_err!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        delegate_shares/2,
      ),
      Error::<Test>::MaxUnlockingsPerEpochReached
    );
  });
}

#[test]
fn test_remove_to_delegate_stake_max_unlockings_reached_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let max_unlockings = MaxDelegateStakeUnlockings::get();
    for n in 0..max_unlockings+1 {
      System::set_block_number(System::block_number() + EpochLength::get() + 1);
      if n+1 > max_unlockings {
        assert_err!(
          Network::remove_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            subnet_id,
            1000,
          ),
          Error::<Test>::MaxUnlockingsReached
        );    
      } else {
        assert_ok!(
          Network::remove_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            subnet_id,
            1000,
          )
        );
        let unbondings: BTreeMap<u64, u128> = DelegateStakeUnbondingLedger::<Test>::get(account(n_account), subnet_id);
        assert_eq!(unbondings.len() as u32, n+1);  
      }
    }
  });
}

#[test]
fn test_switch_delegate_stake() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let from_subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    build_activated_subnet(from_subnet_path.clone(), 0, 0, deposit_amount, amount);
    let from_subnet_id = SubnetPaths::<Test>::get(from_subnet_path.clone()).unwrap();

    let to_subnet_path: Vec<u8> = "petals-team/StableBeluga3".into();
    build_activated_subnet(to_subnet_path.clone(), 0, 0, deposit_amount, amount);
    let to_subnet_id = SubnetPaths::<Test>::get(to_subnet_path.clone()).unwrap();

    let n_account = 255;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(from_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(from_subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        from_subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), from_subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(from_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(from_subnet_id);

    let mut from_delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(from_delegate_balance, delegate_stake_to_be_added_as_shares);

    assert_ok!(
      Network::transfer_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        from_subnet_id,
        to_subnet_id,
        delegate_shares,
      ) 
    );
    let from_delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), from_subnet_id);
    assert_eq!(from_delegate_shares, 0);

    let to_delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), to_subnet_id);
    // assert_eq!(to_delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(to_delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(to_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(to_subnet_id);

    let mut to_delegate_balance = Network::convert_to_balance(
      to_delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // Will lose about .01% of the transfer value on first transfer into a pool
    // The balance should be about ~99% of the ``from`` subnet to the ``to`` subnet
    assert!(
      (to_delegate_balance >= Network::percent_mul(from_delegate_balance, 9999)) &&
      (to_delegate_balance <= from_delegate_balance)
    );
  });
}

#[test]
fn test_switch_delegate_stake_not_enough_stake_err() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let from_subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    build_activated_subnet(from_subnet_path.clone(), 0, 0, deposit_amount, amount);
    let from_subnet_id = SubnetPaths::<Test>::get(from_subnet_path.clone()).unwrap();

    let to_subnet_path: Vec<u8> = "petals-team/StableBeluga3".into();
    build_activated_subnet(to_subnet_path.clone(), 0, 0, deposit_amount, amount);
    let to_subnet_id = SubnetPaths::<Test>::get(to_subnet_path.clone()).unwrap();

    let n_account = 255;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    assert_err!(
      Network::transfer_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        from_subnet_id,
        to_subnet_id,
        0,
      ),
      Error::<Test>::NotEnoughStakeToWithdraw
    );

    assert_err!(
      Network::transfer_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        from_subnet_id,
        to_subnet_id,
        1000,
      ),
      Error::<Test>::NotEnoughStakeToWithdraw
    );
  });
}

// #[test]
// fn test_remove_to_delegate_stake_epochs_not_met_err() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let _ = Balances::deposit_creating(&account(0), deposit_amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
//     let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

//     let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
//       amount,
//       total_subnet_delegated_stake_shares,
//       total_subnet_delegated_stake_balance
//     );

//     if total_subnet_delegated_stake_shares == 0 {
//       delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
//     }

//     System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

//     assert_ok!(
//       Network::add_to_delegate_stake(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         amount,
//       ) 
//     );

//     let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(0), subnet_id);
//     assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
//     assert_ne!(delegate_shares, 0);

//     let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
//     let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

//     let mut delegate_balance = Network::convert_to_balance(
//       delegate_shares,
//       total_subnet_delegated_stake_shares,
//       total_subnet_delegated_stake_balance
//     );
//     // The first depositor will lose a percentage of their deposit depending on the size
//     // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
//     assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
//     assert!(
//       (delegate_balance >= Network::percent_mul(amount, 9999)) &&
//       (delegate_balance <= amount)
//     );

//     // assert_err!(
//     //   Network::remove_delegate_stake(
//     //     RuntimeOrigin::signed(account(0)),
//     //     subnet_id,
//     //     delegate_shares,
//     //   ),
//     //   Error::<Test>::InsufficientCooldown
//     // );
//   });
// }

#[test]
fn test_remove_delegate_stake_after_subnet_remove() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 9999)) &&
      (delegate_balance <= amount)
    );

    let epoch_length = EpochLength::get();
    let cooldown_epochs = DelegateStakeCooldownEpochs::get();

    assert_ok!(
      Network::deactivate_subnet( 
        subnet_path.clone().into(),
        SubnetRemovalReason::SubnetDemocracy,
      )
    );

    // System::set_block_number(System::block_number() + epoch_length * cooldown_epochs);

    let balance = Balances::free_balance(&account(n_account));
    let epoch = System::block_number() / epoch_length;

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        delegate_shares,
      )
    );
    let post_balance = Balances::free_balance(&account(n_account));
    assert_eq!(post_balance, balance);

    let unbondings: BTreeMap<u64, u128> = DelegateStakeUnbondingLedger::<Test>::get(account(n_account), subnet_id);
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(ledger_epoch, &epoch);
    assert!(*ledger_balance <= delegate_balance);

    assert_err!(
      Network::claim_delegate_stake_unbondings(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
      ),
      Error::<Test>::NoDelegateStakeUnbondingsOrCooldownNotMet
    );

    System::set_block_number(System::block_number() + ((epoch_length  + 1) * cooldown_epochs));

    assert_ok!(
      Network::claim_delegate_stake_unbondings(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
      )
    );

    let post_balance = Balances::free_balance(&account(n_account));

    assert!(
      (post_balance >= Network::percent_mul(starting_delegator_balance, 9999)) &&
      (post_balance <= starting_delegator_balance)
    );

    let unbondings: BTreeMap<u64, u128> = DelegateStakeUnbondingLedger::<Test>::get(account(n_account), subnet_id);
    assert_eq!(unbondings.len(), 0);
  });
}

///
///
///
///
///
///
///
/// Validate / Attest / Rewards
///
///
///
///
///
///
///

/// Validate 

// #[test]
// fn test_choose_accountants() {
//   new_test_ext().execute_with(|| {
    
//     setup_blocks(38);

//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let deposit_amount: u128 = 1000000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let mut amount_staked: u128 = 0;

//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, amount + deposit_amount, amount);
//     make_subnet_submittable();

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);

//     Network::shift_node_classes(System::block_number(), epoch_length);

//     let epoch = System::block_number() / epoch_length;

//     let validator = SubnetRewardsValidator::<Test>::get(subnet_id, epoch as u32);
//     assert!(validator == None, "Validator should be None");

//     let accountants = CurrentAccountants::<Test>::get(subnet_id, epoch as u32);
//     assert!(accountants == None, "Accountant should be None");

//     Network::do_epoch_preliminaries(System::block_number(), epoch as u32, epoch_length);

//     let validator = SubnetRewardsValidator::<Test>::get(subnet_id, epoch as u32);
//     assert!(validator != None, "Validator is None");

//     let accountants = CurrentAccountants::<Test>::get(subnet_id, epoch as u32);
//     assert!(accountants != None, "Accountants is None");
//     assert_eq!(accountants.unwrap().len() as u32, TargetAccountantsLength::<Test>::get());


//     let subnet_node_data_vec = subnet_node_data(0, n_peers);
//     assert_ok!(
//       Network::validate(
//         RuntimeOrigin::signed(validator.unwrap()), 
//         subnet_id,
//         subnet_node_data_vec.clone()
//       )
//     );

//   });
// }

#[test]
fn test_validate() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch as u32, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    let validator = SubnetRewardsValidator::<Test>::get(subnet_id, epoch as u32);
    assert!(validator != None, "Validator is None");

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(0)), 
        subnet_id,
        subnet_node_data_vec.clone()
      )
    );

    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch as u32).unwrap();

    assert_eq!(submission.validator, account(0), "Err: validator");
    assert_eq!(submission.data.len(), subnet_node_data_vec.len(), "Err: data len");
    assert_eq!(submission.sum, DEFAULT_SCORE * total_subnet_nodes as u128, "Err: sum");
    assert_eq!(submission.attests.len(), 1, "Err: attests"); // validator auto-attests

    assert_err!(
      Network::validate(
        RuntimeOrigin::signed(account(0)), 
        subnet_id,
        subnet_node_data_vec.clone()
      ),
      Error::<Test>::SubnetRewardsAlreadySubmitted
    );
  });
}

#[test]
fn test_validate_invalid_validator() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);
    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch as u32, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    let mut validator = SubnetRewardsValidator::<Test>::get(subnet_id, epoch as u32);
    assert!(validator.clone() != None, "Validator is None");
    if validator.clone().unwrap() == account(0) {
      validator = Some(account(1));
    }
  
    assert_err!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone().unwrap()), 
        subnet_id,
        subnet_node_data_vec
      ),
      Error::<Test>::InvalidValidator
    );
  });
}

/// Attest

#[test]
fn test_attest() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch as u32, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    let validator = SubnetRewardsValidator::<Test>::get(subnet_id, epoch as u32);
    assert!(validator != None, "Validator is None");

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone().unwrap()), 
        subnet_id,
        subnet_node_data_vec.clone()
      )
    );

    // Attest
    for n in 0..total_subnet_nodes {
      if account(n) == validator.clone().unwrap() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch as u32).unwrap();

    assert_eq!(submission.validator, validator.clone().unwrap());
    assert_eq!(submission.data.len(), subnet_node_data_vec.len());
    assert_eq!(submission.attests.len(), total_subnet_nodes as usize);
    if account(0) == validator.clone().unwrap() {
      assert_ne!(submission.attests.get(&account(0)), None);
      assert_eq!(submission.attests.get(&account(0)), Some(&System::block_number()));
    } else {
      assert_ne!(submission.attests.get(&account(1)), None);
      assert_eq!(submission.attests.get(&account(1)), Some(&System::block_number()));
    }
  });
}


#[test]
fn test_attest_remove_exiting_attester() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch as u32, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Get validator
    let validator = SubnetRewardsValidator::<Test>::get(subnet_id, epoch as u32);
    assert!(validator != None, "Validator is None");

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(0)), 
        subnet_id,
        subnet_node_data_vec.clone()
      )
    );

    // Attest
    for n in 0..total_subnet_nodes {
      if account(n) == validator.clone().unwrap() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch as u32).unwrap();

    assert_eq!(submission.validator, account(0));
    assert_eq!(submission.data.len(), subnet_node_data_vec.len());
    assert_eq!(submission.attests.len(), total_subnet_nodes as usize);
    if account(0) == validator.clone().unwrap() {
      assert_ne!(submission.attests.get(&account(0)), None);
      assert_eq!(submission.attests.get(&account(0)), Some(&System::block_number()));
    } else {
      assert_ne!(submission.attests.get(&account(1)), None);
      assert_eq!(submission.attests.get(&account(1)), Some(&System::block_number()));
    }

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(1)), 
        subnet_id,
      )
    );

    post_remove_subnet_node_ensures(1, subnet_id);

    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch as u32).unwrap();
    assert_eq!(submission.attests.len(), (total_subnet_nodes - 1) as usize);
    // assert_eq!(submission.attests.get(&account(1)), None);
    assert_eq!(submission.attests.get(&account(1)), None);
  });
}

#[test]
fn test_attest_no_submission_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch as u32, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Get validator
    let validator = SubnetRewardsValidator::<Test>::get(subnet_id, epoch as u32);
    assert!(validator != None, "Validator is None");

    assert_err!(
      Network::attest(
        RuntimeOrigin::signed(validator.unwrap()), 
        subnet_id,
      ),
      Error::<Test>::InvalidSubnetRewardsSubmission
    );
  });
}

#[test]
fn test_attest_already_attested_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch as u32, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Get validator
    let validator = SubnetRewardsValidator::<Test>::get(subnet_id, epoch as u32);
    assert!(validator != None, "Validator is None");

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone().unwrap()), 
        subnet_id,
        subnet_node_data_vec.clone()
      )
    );

    // Attest
    for n in 0..total_subnet_nodes {
      if account(n) == validator.clone().unwrap() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch as u32).unwrap();

    assert_eq!(submission.validator, account(0));
    assert_eq!(submission.data.len(), subnet_node_data_vec.len());
    // assert_eq!(submission.sum, DEFAULT_SCORE * total_subnet_nodes as u128);
    assert_eq!(submission.attests.len(), total_subnet_nodes as usize);

    for n in 0..total_subnet_nodes {
      if account(n) == validator.clone().unwrap() {
        continue
      }
      assert_ne!(submission.attests.get(&account(n)), None);
      assert_eq!(submission.attests.get(&account(n)), Some(&System::block_number()));
    }

    for n in 0..total_subnet_nodes {
      if account(n) == validator.clone().unwrap() {
        continue
      }
      assert_err!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        ),
        Error::<Test>::AlreadyAttested
      );
    }
  });
}

///
///
///
///
///
///
///
/// Rewards
///
///
///
///
///
///
///

#[test]
fn test_reward_subnets() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch as u32, epoch_length);


    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Get validator
    let validator = SubnetRewardsValidator::<Test>::get(subnet_id, epoch as u32);
    assert!(validator != None, "Validator is None");

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone().unwrap()), 
        subnet_id,
        subnet_node_data_vec.clone()
      )
    );

    // Attest
    for n in 0..total_subnet_nodes {
      if account(n) == validator.clone().unwrap() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    Network::reward_subnets(System::block_number(), epoch as u32, epoch_length);
  });
}

#[test]
fn test_reward_subnets_remove_subnet_node() {
  new_test_ext().execute_with(|| {
    let max_absent = MaxSubnetNodePenalties::<Test>::get();
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    increase_epochs(1);

    let epoch_length = EpochLength::get();

    // shift node classes
    // validate n-1
    // attest   n-1
    // Simulate epochs
    for num in 0..max_absent+1 {
      let epoch = System::block_number() / epoch_length;
  
      let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes-1);
    
      // --- Insert validator
      SubnetRewardsValidator::<Test>::insert(subnet_id, epoch as u32, account(0));
  
      // validate without n-1
      assert_ok!(
        Network::validate(
          RuntimeOrigin::signed(account(0)), 
          subnet_id,
          subnet_node_data_vec.clone()
        )
      );
  
      // Attest without n-1
      for n in 1..total_subnet_nodes-1 {
        assert_ok!(
          Network::attest(
            RuntimeOrigin::signed(account(n)), 
            subnet_id,
          )
        );
      }
      
      // --- Get submission data and count before node is removed
      // Check rewards
      // Ensure only attestors, validators, and validated get rewards
      let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch as u32).unwrap();

      // --- Any removals impact the following epochs attestation data unless removed ahead of rewards
      let submission_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_accounts(
        subnet_id, 
        &SubnetNodeClass::Submittable, 
        epoch as u64
      );

      let submission_nodes_count = submission_nodes.len() as u128;

      Network::reward_subnets(System::block_number(), epoch as u32, epoch_length);
      let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, account(total_subnet_nodes-1));

      if num + 1 > max_absent {
        post_remove_subnet_node_ensures(total_subnet_nodes-1, subnet_id);
        // when node is removed they're SubnetNodePenalties is reset to zero
        assert_eq!(node_absent_count, 0);  
      } else {
        assert_eq!(node_absent_count, num+1);  
      }

      let base_reward_per_mb: u128 = BaseRewardPerMB::<Test>::get();
      let delegate_stake_rewards_percentage: u128 = DelegateStakeRewardsPercentage::<Test>::get();
      let overall_subnet_reward: u128 = Network::percent_mul(base_reward_per_mb, DEFAULT_MEM_MB);
      let delegate_stake_reward: u128 = Network::percent_mul(overall_subnet_reward, delegate_stake_rewards_percentage);
      let subnet_reward: u128 = overall_subnet_reward.saturating_sub(delegate_stake_reward);
  
      let reward_ratio: u128 = Network::percent_div(DEFAULT_SCORE, submission.sum);
      let account_reward: u128 = Network::percent_mul(reward_ratio, subnet_reward);
  
      let base_reward = BaseValidatorReward::<Test>::get();
  
      let submission_attestations: u128 = submission.attests.len() as u128;
      let attestation_percentage: u128 = Network::percent_div(submission_attestations, submission_nodes_count);

      // check each subnet nodes balance increased
      for n in 0..total_subnet_nodes {
        if n == 0 {
          // validator
          let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
          let validator_reward: u128 = Network::percent_mul(base_reward, attestation_percentage);
          assert_eq!(stake_balance, amount + (account_reward * (num+1) as u128) + (validator_reward * (num+1) as u128));
        } else if n == total_subnet_nodes - 1 {
          // node removed | should have no rewards
          let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
          assert!(stake_balance == amount, "Invalid subnet node staking rewards");
        } else {
          // attestors
          let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
          assert!(stake_balance == amount + (account_reward * (num+1) as u128), "Invalid subnet node staking rewards");
        }
      }

      increase_epochs(1);
    }

    // node should be removed
    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, account(total_subnet_nodes - 1));
    assert_eq!(subnet_node_data, Err(()));

    let subnet_node_account = SubnetNodeAccount::<Test>::try_get(subnet_id, peer(total_subnet_nodes - 1));
    assert_eq!(subnet_node_account, Err(()));
  
    // let subnet_accounts = SubnetAccount::<Test>::get(subnet_id);
    // let subnet_account = subnet_accounts.get(&account(total_subnet_nodes - 1));
    // // Since we increase epochs at the end, substract epoch length from current block number
    // assert_eq!(subnet_accounts.get(&account(total_subnet_nodes - 1)), Some(&(System::block_number() - epoch_length)));
  });
}

// #[test]
// fn test_reward_subnets_absent_node_increment_decrement() {
//   new_test_ext().execute_with(|| {
//     let max_absent = MaxSubnetNodePenalties::<Test>::get();
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     increase_epochs(1);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);

//     // simulate epochs
//     for num in 0..10 {
//       let epoch = System::block_number() / epoch_length;

//       // --- Insert validator
//       SubnetRewardsValidator::<Test>::insert(subnet_id, epoch as u32, account(0));
    
//       if num % 2 == 0 {
//         // increment on even epochs

//         let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes-1);
    
//         assert_ok!(
//           Network::validate(
//             RuntimeOrigin::signed(account(0)), 
//             subnet_id,
//             subnet_node_data_vec.clone()
//           )
//         );
    
//         // Attest
//         for n in 1..total_subnet_nodes-1 {
//           assert_ok!(
//             Network::attest(
//               RuntimeOrigin::signed(account(n)), 
//               subnet_id,
//             )
//           );
//         }
        
//         Network::reward_subnets(System::block_number(), epoch as u32, epoch_length);
  
//         let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, account(total_subnet_nodes-1));
//         assert_eq!(node_absent_count, 1);
//       } else {
//         // decrement on odd epochs
//         let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);
        
//         assert_ok!(
//           Network::validate(
//             RuntimeOrigin::signed(account(0)), 
//             subnet_id,
//             subnet_node_data_vec.clone()
//           )
//         );
    
//         // Attest
//         for n in 1..total_subnet_nodes {
//           assert_ok!(
//             Network::attest(
//               RuntimeOrigin::signed(account(n)), 
//               subnet_id,
//             )
//           );
//         }
        
//         Network::reward_subnets(System::block_number(), epoch as u32, epoch_length);
  
//         let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, account(total_subnet_nodes-1));
//         assert_eq!(node_absent_count, 0);  
//       }

//       increase_epochs(1);
//     }
//   });
// }

#[test]
fn test_reward_subnets_check_balances() {
  new_test_ext().execute_with(|| {
    let max_absent = MaxSubnetNodePenalties::<Test>::get();

    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);
  
    // --- Insert validator
    SubnetRewardsValidator::<Test>::insert(subnet_id, epoch as u32, account(0));

    // validate without n-1
    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(0)), 
        subnet_id,
        subnet_node_data_vec.clone()
      )
    );

    // Attest without n-1
    for n in 1..total_subnet_nodes {
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    // --- Get submission data and count before node is removed
    // Check rewards
    // Ensure only attestors, validators, and validated get rewards
    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch as u32).unwrap();

    // --- Any removals impact the following epochs attestation data unless removed ahead of rewards
    let submission_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_accounts(subnet_id, &SubnetNodeClass::Submittable, epoch);
    let submission_nodes_count = submission_nodes.len() as u128;

    Network::reward_subnets(System::block_number(), epoch as u32, epoch_length);
    let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, account(total_subnet_nodes-1));
    assert_eq!(node_absent_count, 0); 
          
    let base_reward_per_mb: u128 = BaseRewardPerMB::<Test>::get();
    let delegate_stake_rewards_percentage: u128 = DelegateStakeRewardsPercentage::<Test>::get();
    let overall_subnet_reward: u128 = Network::percent_mul(base_reward_per_mb, DEFAULT_MEM_MB);
    let delegate_stake_reward: u128 = Network::percent_mul(overall_subnet_reward, delegate_stake_rewards_percentage);
    let subnet_reward: u128 = overall_subnet_reward.saturating_sub(delegate_stake_reward);

    let reward_ratio: u128 = Network::percent_div(DEFAULT_SCORE, submission.sum);
    let account_reward: u128 = Network::percent_mul(reward_ratio, subnet_reward);

    let base_reward = BaseValidatorReward::<Test>::get();

    let submission_attestations: u128 = submission.attests.len() as u128;
    let attestation_percentage: u128 = Network::percent_div(submission_attestations, submission_nodes_count);

    // check each subnet nodes balance increased
    for n in 0..total_subnet_nodes {
      if n == 0 {
        // validator
        let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
        let validator_reward: u128 = Network::percent_mul(base_reward, attestation_percentage);
        assert_eq!(stake_balance, amount + (account_reward as u128) + (validator_reward as u128));
      } else {
        // attestors
        let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
        assert!(stake_balance == amount + (account_reward as u128), "Invalid subnet node staking rewards")  
      }
    }
  });
}

#[test]
fn test_reward_subnets_validator_slash() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch as u32, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Get validator
    let validator = SubnetRewardsValidator::<Test>::get(subnet_id, epoch as u32);
    assert!(validator != None, "Validator is None");

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone().unwrap()), 
        subnet_id,
        subnet_node_data_vec.clone()
      )
    );

    // No attests to ensure validator is slashed
    
    let before_slash_validator_stake_balance: u128 = AccountSubnetStake::<Test>::get(&validator.clone().unwrap(), subnet_id);

    Network::reward_subnets(System::block_number(), epoch as u32, epoch_length);

    let slashed_validator_stake_balance: u128 = AccountSubnetStake::<Test>::get(&validator.clone().unwrap(), subnet_id);

    // Ensure validator was slashed
    assert!(before_slash_validator_stake_balance > slashed_validator_stake_balance, "Validator was not slashed")
  });
}

#[test]
fn test_reward_subnets_subnet_penalty_count() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Insert validator
    SubnetRewardsValidator::<Test>::insert(subnet_id, epoch as u32, account(0));

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(0)), 
        subnet_id,
        Vec::new()
      )
    );

    // Attest
    for n in 1..total_subnet_nodes {
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    Network::reward_subnets(System::block_number(), epoch as u32, epoch_length);

    let subnet_penalty_count = SubnetPenaltyCount::<Test>::get(subnet_id);
    assert_eq!(subnet_penalty_count, 1);

    let subnet_node_penalty_count = SubnetNodePenalties::<Test>::get(subnet_id, account(0));
    assert_eq!(subnet_node_penalty_count, 0);

    let account_penalty_count = AccountPenaltyCount::<Test>::get(account(0));
    assert_eq!(account_penalty_count, 0);
  });
}

#[test]
fn test_reward_subnets_account_penalty_count() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Insert validator
    SubnetRewardsValidator::<Test>::insert(subnet_id, epoch as u32, account(0));

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(0)), 
        subnet_id,
        Vec::new()
      )
    );

    // No Attest

    Network::reward_subnets(System::block_number(), epoch as u32, epoch_length);

    let subnet_penalty_count = SubnetPenaltyCount::<Test>::get(subnet_id);
    assert_eq!(subnet_penalty_count, 1);

    let subnet_node_penalty_count = SubnetNodePenalties::<Test>::get(subnet_id, account(0));
    assert_eq!(subnet_node_penalty_count, 1);
  });
}

///

///



#[test]
fn test_do_epoch_preliminaries_deactivate_subnet_enactment_period() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let cost = Network::get_subnet_initialization_cost(0);
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      memory_mb: DEFAULT_MEM_MB,
      registration_blocks: DEFAULT_REGISTRATION_BLOCKS,
    };
  
    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();

    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance(subnet.min_nodes);
    let _ = Balances::deposit_creating(&account(0), min_subnet_delegate_stake+1000);
  
    let registration_blocks = subnet.registration_blocks;
    let max_registration_block = subnet.initialized + subnet.registration_blocks;

    let mut subnet_registering = true;
    let subnet_activation_enactment_period = SubnetActivationEnactmentPeriod::<Test>::get();

    while subnet_registering {
      increase_epochs(1);
      let block_number = System::block_number();

      let epoch_length = EpochLength::get();
      let epoch = System::block_number() / epoch_length;  

      Network::do_epoch_preliminaries(block_number, epoch as u32, epoch_length);
      
      if block_number > max_registration_block + subnet_activation_enactment_period {
        assert_eq!(
          *network_events().last().unwrap(),
          Event::SubnetDeactivated {
            subnet_id: subnet_id, 
            reason: SubnetRemovalReason::EnactmentPeriod
          }
        );

        let removed_subnet = SubnetsData::<Test>::try_get(subnet_id);
        assert_eq!(removed_subnet, Err(()));
        subnet_registering = false;
      } else {
        let registered_subnet = SubnetsData::<Test>::try_get(subnet_id).unwrap();
        assert_eq!(registered_subnet.id, subnet_id);
      }
    }
  });
}

#[test]
fn test_do_epoch_preliminaries_deactivate_min_subnet_delegate_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);

    // --- Remove delegate stake to force MinSubnetDelegateStake removal reason
    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(1), subnet_id);
    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        delegate_shares,
      ) 
    );

    increase_epochs(1);
    let block_number = System::block_number();

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;  

    Network::do_epoch_preliminaries(block_number, epoch as u32, epoch_length);
    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::MinSubnetDelegateStake
      }
    ); 
  });
}

#[test]
fn test_do_epoch_preliminaries_deactivate_max_penalties() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);

    let max_subnet_penalty_count = MaxSubnetPenaltyCount::<Test>::get();
    SubnetPenaltyCount::<Test>::insert(subnet_id, max_subnet_penalty_count + 1);

    increase_epochs(1);
    let block_number = System::block_number();

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(block_number, epoch as u32, epoch_length);
    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::MaxPenalties
      }
    ); 
  });
}

#[test]
fn test_do_epoch_preliminaries_choose_validator() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);

    increase_epochs(1);
    let block_number = System::block_number();

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(block_number, epoch as u32, epoch_length);
    let validator = SubnetRewardsValidator::<Test>::get(subnet_id, epoch as u32);
    assert_ne!(validator, None);
  });
}

// // #[test]
// // fn test_add_subnet_node_signature() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());
// //     assert_eq!(Network::total_subnets(), 1);

// // let mut n_peers: u32 = Network::max_subnet_nodes();
// // if n_peers > MAX_SUBNET_NODES {
// //   n_peers = MAX_SUBNET_NODES
// // }

// //     let deposit_amount: u128 = 1000000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let mut amount_staked: u128 = 0;

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let encoded_peer_id = Encode::encode(&peer(0).0.to_vec());
// //     let public = sr25519_generate(0.into(), None);
// //     let who_account: AccountIdOf<Test> = MultiSigner::Sr25519(public).into_account().into();
// //     let signature =
// //       MultiSignature::Sr25519(sr25519_sign(0.into(), &public, &encoded_peer_id).unwrap());

// //     assert_ok!(
// //       Network::add_subnet_node(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(0),
// //         amount,
// //         // signature,
// //         // who_account
// //       ) 
// //     );

// //     let node_set = SubnetNodesClasses::<Test>::get(subnet_id, SubnetNodeClass::Idle);
// //     assert_eq!(node_set.len(), n_peers as usize);

// //   })
// // }

// // #[test]
// // fn validate_signature() {
// // 	new_test_ext().execute_with(|| {
// // 		let user_1_pair = sp_core::sr25519::Pair::from_string("//Alice", None).unwrap();
// // 		let user_1_signer = MultiSigner::Sr25519(user_1_pair.public());
// //     log::error!("user_1_signer {:?}", user_1_signer);
// // 		let user_1 = user_1_signer.clone().into_account();
// //     log::error!("user_1 {:?}", user_1);
// // 		let peer_id: PeerId = peer(0);
// // 		let encoded_data = Encode::encode(&peer_id);
// // 		let signature = MultiSignature::Sr25519(user_1_pair.sign(&encoded_data));
// // 		assert_ok!(Network::validate_signature(&encoded_data, &signature, &user_1));

// // 		let mut wrapped_data: Vec<u8> = Vec::new();
// // 		wrapped_data.extend(b"<Bytes>");
// // 		wrapped_data.extend(&encoded_data);
// // 		wrapped_data.extend(b"</Bytes>");

// // 		let signature = MultiSignature::Sr25519(user_1_pair.sign(&wrapped_data));
// // 		assert_ok!(Network::validate_signature(&encoded_data, &signature, &user_1));
// // 	})
// // }

// // #[test]
// // fn validate_signature_and_peer() {
// // 	new_test_ext().execute_with(|| {
// //     // validate signature
// // 		let user_1_pair = sp_core::sr25519::Pair::from_string("//Alice", None).unwrap();
// // 		let user_1_signer = MultiSigner::Sr25519(user_1_pair.public());
// // 		let user_1 = user_1_signer.clone().into_account();
// // 		let peer_id: PeerId = peer(0);
// // 		let encoded_data = Encode::encode(&peer_id);
// // 		let signature = MultiSignature::Sr25519(user_1_pair.sign(&encoded_data));
// // 		assert_ok!(Network::validate_signature(&encoded_data, &signature, &user_1));

// // 		let mut wrapped_data: Vec<u8> = Vec::new();
// // 		wrapped_data.extend(b"<Bytes>");
// // 		wrapped_data.extend(&encoded_data);
// // 		wrapped_data.extend(b"</Bytes>");

// // 		let signature = MultiSignature::Sr25519(user_1_pair.sign(&wrapped_data));
// // 		assert_ok!(Network::validate_signature(&encoded_data, &signature, &user_1));

// //     // validate signature is the owner of the peer_id
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;

// //     let mut total_staked: u128 = 0;

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let _ = Balances::deposit_creating(&user_1, deposit_amount);
    
// //     assert_ok!(
// //       Network::add_subnet_node(
// //         RuntimeOrigin::signed(user_1),
// //         subnet_id,
// //         peer(0),
// //         amount,
// //       ) 
// //     );
// // 	})
// // }

///
///
///
///
///
///
///
/// Proposals
///
///
///
///
///
///
///

// #[test]
// fn test_propose() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     let proposal_bid_amount = ProposalBidAmount::<Test>::get();
//     let plaintiff_starting_balance = Balances::free_balance(&account(0));

//     let accountant_nodes = SubnetNodesClasses::<Test>::get(subnet_id, SubnetNodeClass::Accountant);

//     let data = Vec::new();

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         data.clone()
//       ) 
//     );
    

//     // --- Ensure bonded
//     let plaintiff_after_balance = Balances::free_balance(&account(0));
//     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);


//     let proposal = Proposals::<Test>::get(subnet_id, 0);
//     assert_eq!(proposal.subnet_id, subnet_id);
//     assert_eq!(proposal.plaintiff, account(0));
//     assert_eq!(proposal.defendant, account(1));
//     assert_eq!(proposal.plaintiff_bond, proposal_bid_amount);
//     assert_eq!(proposal.defendant_bond, 0);
//     assert_eq!(proposal.eligible_voters.len(), accountant_nodes.len());
//     assert_eq!(proposal.start_block, System::block_number());
//     assert_eq!(proposal.challenge_block, 0);
//     assert_eq!(proposal.plaintiff_data, data);
//     assert_eq!(proposal.defendant_data, data);
//     assert_eq!(proposal.complete, false);
//   })
// }

// #[test]
// fn test_propose_subnet_not_exist() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_err!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         2,
//         peer(1),
//         Vec::new()
//       ),
//       Error::<Test>::SubnetNotExist
//     );
//   })
// }

// #[test]
// fn test_propose_subnet_node_not_exist() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_err!(
//       Network::propose(
//         RuntimeOrigin::signed(account(n_peers+1)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ),
//       Error::<Test>::SubnetNodeNotExist
//     );
//   })
// }

// #[test]
// fn test_propose_not_accountant() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes() - 1;
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     let _ = Balances::deposit_creating(&account(n_peers+1), deposit_amount);
//     assert_ok!(
//       Network::add_subnet_node(
//         RuntimeOrigin::signed(account(n_peers+1)),
//         subnet_id,
//         peer(n_peers+1),
//         amount,
//       ) 
//     );

//     assert_err!(
//       Network::propose(
//         RuntimeOrigin::signed(account(n_peers+1)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ),
//       Error::<Test>::NodeAccountantEpochNotReached
//     );
//   })
// }

// #[test]
// fn test_propose_peer_id_not_exist() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes() - 1;
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_err!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(n_peers+1),
//         Vec::new()
//       ),
//       Error::<Test>::PeerIdNotExist
//     );
//   })
// }

// #[test]
// fn test_propose_min_subnet_nodes_accountants_error() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes() - 1;
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);

//     let _ = Balances::deposit_creating(&account(n_peers+1), deposit_amount);
//     assert_ok!(
//       Network::add_subnet_node(
//         RuntimeOrigin::signed(account(n_peers+1)),
//         subnet_id,
//         peer(n_peers+1),
//         amount,
//       ) 
//     );

//     // Shift node classes to accountant epoch for account(n_peers+1)
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     // Add new subnet nodes that aren't accountants yet
//     for n in 0..n_peers {
//       let _ = Balances::deposit_creating(&account(n), deposit_amount);
//       assert_ok!(
//         Network::add_subnet_node(
//           RuntimeOrigin::signed(account(n)),
//           subnet_id,
//           peer(n),
//           amount,
//         ) 
//       );
//     }
  
//     assert_err!(
//       Network::propose(
//         RuntimeOrigin::signed(account(n_peers+1)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ),
//       Error::<Test>::SubnetNodesMin
//     );
//   })
// }

// #[test]
// fn test_propose_peer_has_active_proposal() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     assert_err!(
//       Network::propose(
//         RuntimeOrigin::signed(account(2)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ),
//       Error::<Test>::NodeHasActiveProposal
//     );

//     assert_err!(
//       Network::propose(
//         RuntimeOrigin::signed(account(3)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ),
//       Error::<Test>::NodeHasActiveProposal
//     );
//   })
// }

// #[test]
// fn test_propose_not_enough_balance_to_bid() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     let proposal_bid_amount = ProposalBidAmount::<Test>::get();
//     let free_balance = Balances::free_balance(&account(0));

//     assert_ok!(
//       Balances::transfer_keep_alive(
//         RuntimeOrigin::signed(account(0)),
//         sp_runtime::MultiAddress::Id(account(1)),
//         free_balance-500,
//       )  
//     );

//     assert_err!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ),
//       Error::<Test>::NotEnoughBalanceToBid
//     );
//   })
// }

// #[test]
// fn test_cancel_proposal() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;
//     let proposal = Proposals::<Test>::get(subnet_id, proposal_index);
//     let plaintiff_bond = proposal.plaintiff_bond;

//     let proposer_balance = Balances::free_balance(&account(0));

//     assert_ok!(
//       Network::cancel_proposal(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         proposal_index,
//       )
//     );

//     // --- Ensure proposer gets bond back
//     let after_cancel_proposer_balance = Balances::free_balance(&account(0));
//     assert_eq!(proposer_balance + plaintiff_bond, after_cancel_proposer_balance);

//     let proposal = Proposals::<Test>::try_get(subnet_id, 0);
//     assert_eq!(proposal, Err(()));
//   })
// }

// #[test]
// fn test_cancel_proposal_not_plaintiff() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_err!(
//       Network::cancel_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//       ),
//       Error::<Test>::NotPlaintiff
//     );
//   })
// }

// #[test]
// fn test_cancel_proposal_already_challenged() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ) 
//     );

//     assert_err!(
//       Network::cancel_proposal(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         proposal_index,
//       ),
//       Error::<Test>::ProposalChallenged
//     );
//   })
// }

// #[test]
// fn test_cancel_proposal_already_complete() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::cancel_proposal(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         proposal_index,
//       )
//     );

//     // assert_err!(
//     //   Network::cancel_proposal(
//     //     RuntimeOrigin::signed(account(0)),
//     //     subnet_id,
//     //     proposal_index,
//     //   ),
//     //   Error::<Test>::ProposalComplete
//     // );
//     assert_err!(
//       Network::cancel_proposal(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         proposal_index,
//       ),
//       Error::<Test>::ProposalInvalid
//     );

//   })
// }

// #[test]
// fn test_challenge_proposal() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     let proposal_bid_amount = ProposalBidAmount::<Test>::get();

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;
//     let defendant_starting_balance = Balances::free_balance(&account(1));

//     assert_ok!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ) 
//     );

//     // --- Ensure bonded
//     let defendant_after_balance = Balances::free_balance(&account(1));
//     assert_eq!(defendant_starting_balance - proposal_bid_amount, defendant_after_balance);
//   })
// }

// #[test]
// fn test_challenge_proposal_invalid_proposal_id() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     assert_err!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         15,
//         Vec::new()
//       ),
//       Error::<Test>::ProposalInvalid
//     );
//   })
// }

// #[test]
// fn test_challenge_proposal_not_defendant() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_err!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(2)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ),
//       Error::<Test>::NotDefendant
//     );
//   })
// }

// #[test]
// fn test_challenge_proposal_complete() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::cancel_proposal(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         proposal_index,
//       )
//     );

//     // assert_err!(
//     //   Network::challenge_proposal(
//     //     RuntimeOrigin::signed(account(1)),
//     //     subnet_id,
//     //     proposal_index,
//     //     Vec::new()
//     //   ),
//     //   Error::<Test>::ProposalComplete
//     // );

//     assert_err!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ),
//       Error::<Test>::ProposalInvalid
//     );
//   })
// }

// #[test]
// fn test_challenge_proposal_challenge_period_passed() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     let challenge_period = ChallengePeriod::<Test>::get();
//     System::set_block_number(System::block_number() + challenge_period + 1);

//     assert_err!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ),
//       Error::<Test>::ProposalChallengePeriodPassed
//     );
//   })
// }

// #[test]
// fn test_challenge_proposal_already_challenged() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ) 
//     );

//     assert_err!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ),
//       Error::<Test>::ProposalChallenged
//     );

//   })
// }

// #[test]
// fn test_challenge_proposal_not_enough_balance_to_bid() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;
//     let proposal_bid_amount = ProposalBidAmount::<Test>::get();
//     let free_balance = Balances::free_balance(&account(1));

//     assert_ok!(
//       Balances::transfer_keep_alive(
//         RuntimeOrigin::signed(account(1)),
//         sp_runtime::MultiAddress::Id(account(2)),
//         free_balance-500,
//       )  
//     );

//     assert_err!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ),
//       Error::<Test>::NotEnoughBalanceToBid
//     );

//   })
// }

// #[test]
// fn test_proposal_voting() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ) 
//     );

//     assert_ok!(
//       Network::vote(
//         RuntimeOrigin::signed(account(2)),
//         subnet_id,
//         proposal_index,
//         VoteType::Yay
//       ) 
//     );

//     let proposal = Proposals::<Test>::get(subnet_id, 0);
//     assert_eq!(proposal.votes.yay.get(&account(2)), Some(&account(2)));
//     assert_ne!(proposal.votes.yay.get(&account(2)), None);

//   })
// }

// #[test]
// fn test_proposal_voting_invalid_proposal_id() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ) 
//     );

//     assert_err!(
//       Network::vote(
//         RuntimeOrigin::signed(account(2)),
//         subnet_id,
//         1,
//         VoteType::Yay
//       ),
//       Error::<Test>::ProposalInvalid
//     );
//   })
// }

// #[test]
// fn test_proposal_voting_subnet_node_not_exist() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ) 
//     );

//     assert_err!(
//       Network::vote(
//         RuntimeOrigin::signed(account(n_peers+1)),
//         subnet_id,
//         proposal_index,
//         VoteType::Yay
//       ),
//       Error::<Test>::SubnetNodeNotExist
//     );
//   })
// }

// #[test]
// fn test_proposal_voting_proposal_unchallenged() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_err!(
//       Network::vote(
//         RuntimeOrigin::signed(account(2)),
//         subnet_id,
//         proposal_index,
//         VoteType::Yay
//       ),
//       Error::<Test>::ProposalUnchallenged
//     );
//   })
// }

// // TODO: Need to finalize and then attempt to vote the proposal for failure
// #[test]
// fn test_proposal_voting_proposal_complete() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_err!(
//       Network::vote(
//         RuntimeOrigin::signed(account(2)),
//         subnet_id,
//         proposal_index,
//         VoteType::Yay
//       ),
//       Error::<Test>::ProposalUnchallenged
//     );
//   })
// }

// #[test]
// fn test_proposal_voting_invalid_voting_period() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );


//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ) 
//     );

//     let voting_period = VotingPeriod::<Test>::get();
//     System::set_block_number(System::block_number() + voting_period + 1);

//     assert_err!(
//       Network::vote(
//         RuntimeOrigin::signed(account(2)),
//         subnet_id,
//         proposal_index,
//         VoteType::Yay
//       ),
//       Error::<Test>::VotingPeriodInvalid
//     );
//   })
// }

// #[test]
// fn test_proposal_voting_not_eligible() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let n_peers: u32 = 12;
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ) 
//     );

//     let _ = Balances::deposit_creating(&account(n_peers+1), deposit_amount);
//     assert_ok!(
//       Network::add_subnet_node(
//         RuntimeOrigin::signed(account(n_peers+1)),
//         subnet_id,
//         peer(n_peers+1),
//         amount,
//       ) 
//     );

//     assert_err!(
//       Network::vote(
//         RuntimeOrigin::signed(account(n_peers+1)),
//         subnet_id,
//         proposal_index,
//         VoteType::Yay
//       ),
//       Error::<Test>::NotEligible
//     );
//   })
// }

// #[test]
// fn test_proposal_voting_already_voted() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ) 
//     );

//     assert_ok!(
//       Network::vote(
//         RuntimeOrigin::signed(account(2)),
//         subnet_id,
//         proposal_index,
//         VoteType::Yay
//       ) 
//     );

//     assert_ok!(
//       Network::vote(
//         RuntimeOrigin::signed(account(3)),
//         subnet_id,
//         proposal_index,
//         VoteType::Yay
//       ) 
//     );

//     assert_err!(
//       Network::vote(
//         RuntimeOrigin::signed(account(3)),
//         subnet_id,
//         proposal_index,
//         VoteType::Yay
//       ),
//       Error::<Test>::AlreadyVoted
//     );

//   })
// }

// #[test]
// fn test_proposal_finalize_proposal_plaintiff_winner() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     let proposal_bid_amount = ProposalBidAmount::<Test>::get();

//     let plaintiff_starting_balance = Balances::free_balance(&account(0));
//     let defendant_starting_balance = Balances::free_balance(&account(1));

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let plaintiff_after_balance = Balances::free_balance(&account(0));
//     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ) 
//     );

//     for n in 0..n_peers {
//       if n == 0 || n == 1 {
//         continue
//       }
//       assert_ok!(
//         Network::vote(
//           RuntimeOrigin::signed(account(n)),
//           subnet_id,
//           proposal_index,
//           VoteType::Yay
//         ) 
//       );  
//     }

//     let plaintiff_after_balance = Balances::free_balance(&account(0));
//     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);

//     let voting_period = VotingPeriod::<Test>::get();
//     System::set_block_number(System::block_number() + voting_period + 1);

//     let voter_starting_balance = Balances::free_balance(&account(3));

//     assert_ok!(
//       Network::finalize_proposal(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         proposal_index,
//       ) 
//     );

//     let mut proposal = Proposals::<Test>::get(subnet_id, proposal_index);
//     let winner_voters_len = proposal.votes.yay.len();
//     assert_eq!(winner_voters_len, (n_peers - 2) as usize);

//     let mut distributees = proposal.votes.yay;
//     // Insert winner to the distributees
//     distributees.insert(account(0));

//     let distribution_amount = proposal_bid_amount.saturating_div(distributees.len() as u128);

//     for n in 0..n_peers {
//       if n == 0 || n == 1 {
//         continue
//       }
//       let voter_balance = Balances::free_balance(&account(n));
//       assert_eq!(voter_balance, voter_starting_balance + distribution_amount);
//     }

//     let distribution_dust = proposal_bid_amount - (distribution_amount * (distributees.len() as u128));

//     // --- Plaintiff after finalization should be bond amount + distribution + dust
//     let plaintiff_after_balance = Balances::free_balance(&account(0));

//     assert_eq!(plaintiff_after_balance, plaintiff_starting_balance + distribution_amount + distribution_dust);

//     // --- Defendant after finalization should be same since they lost
//     let defendant_after_balance = Balances::free_balance(&account(1));
//     assert_eq!(defendant_starting_balance - proposal_bid_amount, defendant_after_balance);
//   })
// }

// #[test]
// fn test_proposal_finalize_proposal_defendant_winner() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     let proposal_bid_amount = ProposalBidAmount::<Test>::get();

//     let plaintiff_starting_balance = Balances::free_balance(&account(0));
//     let defendant_starting_balance = Balances::free_balance(&account(1));

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ) 
//     );

//     for n in 0..n_peers {
//       if n == 0 || n == 1 {
//         continue
//       }
//       assert_ok!(
//         Network::vote(
//           RuntimeOrigin::signed(account(n)),
//           subnet_id,
//           proposal_index,
//           VoteType::Nay
//         ) 
//       );  
//     }

//     let plaintiff_after_balance = Balances::free_balance(&account(0));
//     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);

//     let voting_period = VotingPeriod::<Test>::get();
//     System::set_block_number(System::block_number() + voting_period + 1);

//     let voter_starting_balance = Balances::free_balance(&account(3));

//     assert_ok!(
//       Network::finalize_proposal(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         proposal_index,
//       ) 
//     );

//     let mut proposal = Proposals::<Test>::get(subnet_id, proposal_index);
//     let winner_voters_len = proposal.votes.nay.len();
//     assert_eq!(winner_voters_len, (n_peers - 2) as usize);

//     let mut distributees = proposal.votes.nay;
//     // Insert winner to the distributees
//     distributees.insert(account(0));

//     let distribution_amount = proposal_bid_amount.saturating_div(distributees.len() as u128);

//     for n in 0..n_peers {
//       if n == 0 || n == 1 {
//         continue
//       }
//       let voter_balance = Balances::free_balance(&account(n));
//       assert_eq!(voter_balance, voter_starting_balance + distribution_amount);
//     }

//     let distribution_dust = proposal_bid_amount - (distribution_amount * (distributees.len() as u128));

//     // --- Plaintiff after finalization should be bond amount + distribution + dust
//     let defendant_after_balance = Balances::free_balance(&account(1));
    
//     assert_eq!(defendant_after_balance, defendant_starting_balance + distribution_amount + distribution_dust);

//     // --- Defendant after finalization should be same since they lost
//     let plaintiff_after_balance = Balances::free_balance(&account(0));
//     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);
//   })
// }

// #[test]
// fn test_proposal_finalize_proposal_unchallenged() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     let proposal_bid_amount = ProposalBidAmount::<Test>::get();

//     let plaintiff_starting_balance = Balances::free_balance(&account(0));
//     let defendant_starting_balance = Balances::free_balance(&account(1));

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_err!(
//       Network::finalize_proposal(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         proposal_index,
//       ),
//       Error::<Test>::ProposalUnchallenged
//     );

//   })
// }

// #[test]
// fn test_proposal_finalize_proposal_complete() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     let proposal_bid_amount = ProposalBidAmount::<Test>::get();

//     let plaintiff_starting_balance = Balances::free_balance(&account(0));
//     let defendant_starting_balance = Balances::free_balance(&account(1));

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let plaintiff_after_balance = Balances::free_balance(&account(0));
//     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ) 
//     );

//     for n in 0..n_peers {
//       if n == 0 || n == 1 {
//         continue
//       }
//       assert_ok!(
//         Network::vote(
//           RuntimeOrigin::signed(account(n)),
//           subnet_id,
//           proposal_index,
//           VoteType::Yay
//         ) 
//       );  
//     }

//     let plaintiff_after_balance = Balances::free_balance(&account(0));
//     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);

//     let voting_period = VotingPeriod::<Test>::get();
//     System::set_block_number(System::block_number() + voting_period + 1);

//     let voter_starting_balance = Balances::free_balance(&account(3));

//     assert_ok!(
//       Network::finalize_proposal(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         proposal_index,
//       ) 
//     );

//     let mut proposal = Proposals::<Test>::get(subnet_id, proposal_index);
//     let winner_voters_len = proposal.votes.yay.len();
//     assert_eq!(winner_voters_len, (n_peers - 2) as usize);

//     let mut distributees = proposal.votes.yay;
//     // Insert winner to the distributees
//     distributees.insert(account(0));

//     let distribution_amount = proposal_bid_amount.saturating_div(distributees.len() as u128);

//     for n in 0..n_peers {
//       if n == 0 || n == 1 {
//         continue
//       }
//       let voter_balance = Balances::free_balance(&account(n));
//       assert_eq!(voter_balance, voter_starting_balance + distribution_amount);
//     }

//     let distribution_dust = proposal_bid_amount - (distribution_amount * (distributees.len() as u128));

//     // --- Plaintiff after finalization should be bond amount + distribution + dust
//     let plaintiff_after_balance = Balances::free_balance(&account(0));

//     assert_eq!(plaintiff_after_balance, plaintiff_starting_balance + distribution_amount + distribution_dust);

//     // --- Defendant after finalization should be same since they lost
//     let defendant_after_balance = Balances::free_balance(&account(1));
//     assert_eq!(defendant_starting_balance - proposal_bid_amount, defendant_after_balance);

//     assert_err!(
//       Network::finalize_proposal(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         proposal_index,
//       ),
//       Error::<Test>::ProposalComplete
//     );
//   })
// }

// #[test]
// fn test_proposal_finalize_proposal_voting_period_invalid() {
// 	new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let mut n_peers: u32 = Network::max_subnet_nodes();
//     if n_peers > MAX_SUBNET_NODES {
//       n_peers = MAX_SUBNET_NODES
//     }
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

//     let epoch_length = EpochLength::get();
//     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
//     Network::shift_node_classes(System::block_number(), epoch_length);

//     let proposal_bid_amount = ProposalBidAmount::<Test>::get();

//     let plaintiff_starting_balance = Balances::free_balance(&account(0));
//     let defendant_starting_balance = Balances::free_balance(&account(1));

//     assert_ok!(
//       Network::propose(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(1),
//         Vec::new()
//       ) 
//     );

//     let plaintiff_after_balance = Balances::free_balance(&account(0));
//     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);

//     let proposal_index = ProposalsCount::<Test>::get() - 1;

//     assert_ok!(
//       Network::challenge_proposal(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//         proposal_index,
//         Vec::new()
//       ) 
//     );

//     for n in 0..n_peers {
//       if n == 0 || n == 1 {
//         continue
//       }
//       assert_ok!(
//         Network::vote(
//           RuntimeOrigin::signed(account(n)),
//           subnet_id,
//           proposal_index,
//           VoteType::Yay
//         ) 
//       );  
//     }

//     assert_err!(
//       Network::finalize_proposal(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         proposal_index,
//       ),
//       Error::<Test>::VotingPeriodInvalid
//     );
//   })
// }

///
///
///
///
///
///
///
/// Math
///
///
///
///
///
///
///

#[test]
fn test_percent_mul() {
  new_test_ext().execute_with(|| {
    let value = Network::percent_mul(53000000, 300000000);

    assert_eq!(value, 15900000, "percent_mul didn't round down");

    // let value = Network::percent_mul_round_up(53000000, 300000000);

    // assert_eq!(value, 15900000, "percent_mul_round_up didn't round up");

    let value = Network::percent_mul(100000000e+18 as u128, PERCENTAGE_FACTOR);

    assert_ne!(value, 0, "percent_mul didn't round down");
    assert_ne!(value, u128::MAX, "percent_mul didn't round down");

    // let value = Network::percent_mul_round_up(100000000e+18 as u128, PERCENTAGE_FACTOR);

    // assert_ne!(value, 0, "percent_mul_round_up didn't round down");
    // assert_ne!(value, u128::MAX, "percent_mul_round_up didn't round down");
  });
}

#[test]
fn test_percent_div() {
  new_test_ext().execute_with(|| {
    // // 100.00 | 10000
    // let value = Network::percent_div(1, 3000);

    // assert_eq!(value, 3, "percent_div didn't round down");

    // let value = Network::percent_div_round_up(1, 3000);

    // assert_eq!(value, 4, "percent_div_round_up didn't round up");

    // 100.0000000 | 1000000000
    let value = Network::percent_div(100000000, 300000000);

    assert_eq!(value, 333333333, "percent_div didn't round down");

    // let value = Network::percent_div_round_up(100000000, 300000000);

    // assert_eq!(value, 400000000, "percent_div_round_up didn't round up");
  });
}

#[test]
fn test_get_min_subnet_nodes() {
  new_test_ext().execute_with(|| {
    let base_node_memory: u128 = BaseSubnetNodeMemoryMB::<Test>::get();
    let min_subnet_nodes = Network::get_min_subnet_nodes(base_node_memory, 500_000);
    log::error!("min_subnet_nodes: {:?}", min_subnet_nodes);

    // assert_eq!(value, 333333333, "percent_div didn't round down");
  });
}

///
///
///
///
///
///
///
/// Randomization
///
///
///
///
///
///
///

fn setup_blocks(blocks: u64) {
  let mut parent_hash = System::parent_hash();

  for i in 1..(blocks + 1) {
    System::reset_events();
    System::initialize(&i, &parent_hash, &Default::default());
    InsecureRandomnessCollectiveFlip::on_initialize(i);

    let header = System::finalize();
    parent_hash = header.hash();
    System::set_block_number(*header.number());
  }
}

#[test]
fn test_randomness() {
  new_test_ext().execute_with(|| {
    setup_blocks(38);
    let gen_rand_num = Network::generate_random_number(1);
    let rand_num = Network::get_random_number(96, 0);
  });
}

// #[test]
// fn test_info_get_subnet_node_stake_by_peer_id() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
//     build_subnet(subnet_path.clone());

//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let _ = Balances::deposit_creating(&account(0), deposit_amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     assert_ok!(
//       Network::add_subnet_node(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         peer(0),
//         amount,
//       ) 
//     );

//     let amount_staked = Network::get_subnet_node_stake_by_peer_id(subnet_id, peer(0));
//     assert_eq!(amount_staked, amount);

//     let amount_staked = Network::get_subnet_node_stake_by_peer_id(subnet_id, peer(1));
//     assert_eq!(amount_staked, 0);
//   });
// }

#[test]
fn test_get_target_subnet_nodes() {
  new_test_ext().execute_with(|| {
    let target_nodes = Network::get_target_subnet_nodes(10);
    log::error!("target_nodes: {:?}", target_nodes);
    assert!(target_nodes < 100);

  });
}

