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
use crate as pallet_admin;
use frame_support::{
  parameter_types,
  traits::{VariantCountOf, Everything, EitherOfDiverse},
  PalletId,
  derive_impl,
  weights::constants::WEIGHT_REF_TIME_PER_SECOND,
};
use frame_system as system;
use sp_core::{ConstU128, ConstU32, ConstU64, H256, U256};
use sp_runtime::BuildStorage;
use sp_runtime::{
	traits::{
		BlakeTwo256, IdentifyAccount, Verify, IdentityLookup, AccountIdLookup
	},
	MultiSignature
};
pub use frame_system::EnsureRoot;
use sp_runtime::Perbill;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test
	{
    System: system,
    InsecureRandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
    Balances: pallet_balances,
    Network: pallet_network,
    Admin: pallet_admin,
    Collective: pallet_collective::<Instance1>
	}
);

pub const MILLISECS_PER_BLOCK: u64 = 6000;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

pub const YEAR: BlockNumber = DAYS * 365;

pub const SECS_PER_BLOCK: u64 = MILLISECS_PER_BLOCK / 1000;

pub type BalanceCall = pallet_balances::Call<Test>;

parameter_types! {
  pub const BlockHashCount: u64 = 250;
  pub const SS58Prefix: u8 = 42;
}

// pub type AccountId = U256;

pub type Signature = MultiSignature;

pub type AccountPublic = <Signature as Verify>::Signer;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

// The address format for describing accounts.
pub type Address = AccountId;

// Balance of an account.
pub type Balance = u128;

// An index to a block.
#[allow(dead_code)]
pub type BlockNumber = u64;

pub const EXISTENTIAL_DEPOSIT: u128 = 500;

impl pallet_insecure_randomness_collective_flip::Config for Test {}

impl pallet_balances::Config for Test {
  type Balance = Balance;
  type RuntimeEvent = RuntimeEvent;
  type DustRemoval = ();
  type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
  // type AccountStore = AccountData<u128>;
  // type AccountStore = StoredMap<Self::AccountId, AccountData<Self::Balance>>;
  type AccountStore = System;
  type MaxLocks = ();
  type WeightInfo = ();
  type MaxReserves = ();
  type ReserveIdentifier = [u8; 8];
  type RuntimeHoldReason = ();
  type FreezeIdentifier = ();
  // type MaxHolds = ();
  // type MaxFreezes = ();
  type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
  type RuntimeFreezeReason = ();
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
  type BaseCallFilter = Everything;
  type BlockWeights = ();
  type BlockLength = ();
  type Block = Block;
  type DbWeight = ();
  type RuntimeOrigin = RuntimeOrigin;
  type RuntimeCall = RuntimeCall;
  type Nonce = u64;
  type Hash = H256;
  type Hashing = BlakeTwo256;
  type AccountId = AccountId;
  type Lookup = AccountIdLookup<AccountId, ()>;
  type RuntimeEvent = RuntimeEvent;
  type BlockHashCount = BlockHashCount;
  type Version = ();
  type PalletInfo = PalletInfo;
  type AccountData = pallet_balances::AccountData<u128>;
  type OnNewAccount = ();
  type OnKilledAccount = ();
  type SystemWeightInfo = ();
  type SS58Prefix = SS58Prefix;
  type OnSetCode = ();
  type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const EpochLength: u64 = 100;
  pub const NetworkPalletId: PalletId = PalletId(*b"/network");
  pub const SubnetInitializationCost: u128 = 100_000_000_000_000_000_000;
  pub const MinProposalStake: u128 = 1_000_000_000_000_000_000;
  pub const DelegateStakeCooldownEpochs: u64 = 100;
  pub const StakeCooldownEpochs: u64 = 100;
	pub const DelegateStakeEpochsRemovalWindow: u64 = 10;
  pub const MaxDelegateStakeUnlockings: u32 = 32;
  pub const MaxStakeUnlockings: u32 = 32;
}

impl pallet_network::Config for Test {
  type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
  type Currency = Balances;
  type EpochLength = EpochLength;
  type StringLimit = ConstU32<100>;
	type InitialTxRateLimit = ConstU64<0>;
  type Randomness = InsecureRandomnessCollectiveFlip;
	type PalletId = NetworkPalletId;
  type SubnetInitializationCost = SubnetInitializationCost;
  type DelegateStakeCooldownEpochs = DelegateStakeCooldownEpochs;
  type StakeCooldownEpochs = DelegateStakeCooldownEpochs;
	type DelegateStakeEpochsRemovalWindow = DelegateStakeEpochsRemovalWindow;
  type MaxDelegateStakeUnlockings = MaxDelegateStakeUnlockings;
  type MaxStakeUnlockings = MaxStakeUnlockings;
  type MinProposalStake = MinProposalStake;
}

parameter_types! {
	pub const VotingPeriod: BlockNumber = DAYS * 21;
	pub const EnactmentPeriod: BlockNumber = DAYS * 7;
  pub const VerifyPeriod: BlockNumber = DAYS * 4;
  pub const MinProposerStake: u128 = 100_000_000_000_000_000_000; // 100 * 1e18
  pub const Quorum: u128 = 100_000_000_000_000_000_000; // 100 * 1e18
  pub const CancelSlashPercent: u8 = 5;
  pub const QuorumVotingPowerPercentage: u8 = 40;
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
  pub BlockWeights: frame_system::limits::BlockWeights =
    frame_system::limits::BlockWeights::with_sensible_defaults(
      Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
      NORMAL_DISPATCH_RATIO,
    );
	pub MaxCollectivesProposalWeight: Weight = Perbill::from_percent(50) * BlockWeights::get().max_block;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
	type SetMembersOrigin = EnsureRoot<AccountId>;
	type MaxProposalWeight = MaxCollectivesProposalWeight;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
  type CollectiveOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
  type NetworkAdminInterface = Network;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}
