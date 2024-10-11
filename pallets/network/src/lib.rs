//! # Template Pallet
//!
//! A pallet with minimal functionality to help developers understand the essential components of
//! writing a FRAME pallet. It is typically used in beginner tutorials or in Substrate template
//! nodes as a starting point for creating a new pallet and **not meant to be used in production**.
//!
//! ## Overview
//!
//! This template pallet contains basic examples of:
//! - declaring a storage item that stores a single `u32` value
//! - declaring and using events
//! - declaring and using errors
//! - a dispatchable function that allows a user to set a new value to storage and emits an event
//!   upon success
//! - another dispatchable function that causes a custom error to be thrown
//!
//! Each pallet section is annotated with an attribute using the `#[pallet::...]` procedural macro.
//! This macro generates the necessary code for a pallet to be aggregated into a FRAME runtime.
//!
//! Learn more about FRAME macros [here](https://docs.substrate.io/reference/frame-macros/).
//!
//! ### Pallet Sections
//!
//! The pallet sections in this template are:
//!
//! - A **configuration trait** that defines the types and parameters which the pallet depends on
//!   (denoted by the `#[pallet::config]` attribute). See: [`Config`].
//! - A **means to store pallet-specific data** (denoted by the `#[pallet::storage]` attribute).
//!   See: [`storage_types`].
//! - A **declaration of the events** this pallet emits (denoted by the `#[pallet::event]`
//!   attribute). See: [`Event`].
//! - A **declaration of the errors** that this pallet can throw (denoted by the `#[pallet::error]`
//!   attribute). See: [`Error`].
//! - A **set of dispatchable functions** that define the pallet's functionality (denoted by the
//!   `#[pallet::call]` attribute). See: [`dispatchables`].
//!
//! Run `cargo doc --package pallet-template --open` to view this pallet's documentation.

// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;
use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchResult},
	traits::{tokens::WithdrawReasons, Get, Currency, ReservableCurrency, ExistenceRequirement, Randomness},
	PalletId,
	ensure,
	storage::bounded_vec::BoundedVec,
};
use frame_system::{self as system, ensure_signed};
use scale_info::prelude::string::String;
use scale_info::prelude::vec::Vec;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use sp_core::OpaquePeerId as PeerId;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, FromRepr};
use sp_runtime::traits::TrailingZeroInput;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
#[cfg(test)]
mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
#[cfg(test)]
mod tests;
// ./target/release/solochain-template-node --dev
// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

mod utils;
mod staking;
mod delegate_staking;
mod subnet_validator;
mod math;
mod randomness;
mod accountant;
mod rewards;
mod info;
mod proposal;
mod admin;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
	// Import various useful types required by all FRAME pallets.
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec;
	use sp_std::vec::Vec;
	
	// The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
	// (`Call`s) in this pallet.
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// The pallet's configuration trait.
	///
	/// All our types and constants a pallet depends on must be declared here.
	/// These types are defined generically and made concrete when the pallet is declared in the
	/// `runtime/src/lib.rs` file of your chain.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + Send + Sync;

		#[pallet::constant]
		type EpochLength: Get<u64>;

		#[pallet::constant]
		type StringLimit: Get<u32>;
	
		#[pallet::constant] // Initial transaction rate limit.
		type InitialTxRateLimit: Get<u64>;
	
		// #[pallet::constant]
		// type SecsPerBlock: Get<u64>;
	
		// #[pallet::constant]
		// type Year: Get<u64>;
		
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type SubnetInitializationCost: Get<u128>;

		#[pallet::constant]
		type CooldownEpochs: Get<u64>;

		#[pallet::constant]
		type DelegateStakeEpochsRemovalWindow: Get<u64>;

		#[pallet::constant]
		type MaxDelegateStakeUnlockings: Get<u32>;
		
		// type OffchainSignature: Verify<Signer = Self::OffchainPublic> + Parameter;
		// type OffchainPublic: IdentifyAccount<AccountId = Self::AccountId>;

		/// Something that provides randomness in the runtime.
		type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

		#[pallet::constant]
		type MinProposalStake: Get<u128>;
	}

	/// A storage item for this pallet.
	///
	/// In this template, we are declaring a storage item called `Something` that stores a single
	/// `u32` value. Learn more about runtime storage here: <https://docs.substrate.io/build/runtime-storage/>
	#[pallet::storage]
	pub type Something<T> = StorageValue<_, u32>;

	/// Events that functions in this pallet can emit.
	///
	/// Events are a simple means of indicating to the outside world (such as dApps, chain explorers
	/// or other users) that some notable update in the runtime has occurred. In a FRAME pallet, the
	/// documentation for each event field and its parameters is added to a node's metadata so it
	/// can be used by external interfaces or tools.
	///
	///	The `generate_deposit` macro generates a function on `Pallet` called `deposit_event` which
	/// will convert the event type of your pallet into `RuntimeEvent` (declared in the pallet's
	/// [`Config`] trait) and deposit it using [`frame_system::Pallet::deposit_event`].
	/// Events for the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Subnets
		SubnetAdded { proposer: T::AccountId, activator: T::AccountId, subnet_id: u32, subnet_path: Vec<u8>, block: u64 },
		SubnetRemoved { account: T::AccountId, subnet_id: u32, subnet_path: Vec<u8>, reason: Vec<u8>, block: u64 },

		// Subnet Nodes
		SubnetNodeAdded { subnet_id: u32, account_id: T::AccountId, peer_id: PeerId, block: u64 },
		SubnetNodeUpdated { subnet_id: u32, account_id: T::AccountId, peer_id: PeerId, block: u64 },
		SubnetNodeRemoved { subnet_id: u32, account_id: T::AccountId, peer_id: PeerId, block: u64 },

		// Stake
		StakeAdded(u32, T::AccountId, u128),
		StakeRemoved(u32, T::AccountId, u128),

		DelegateStakeAdded(u32, T::AccountId, u128),
		DelegateStakeRemoved(u32, T::AccountId, u128),
		DelegateStakeSwitched(u32, u32, T::AccountId, u128),

		// Admin 
		SetVoteSubnetIn(Vec<u8>),
    SetVoteSubnetOut(Vec<u8>),
    SetMaxSubnets(u32),
    SetMinSubnetNodes(u32),
    SetMaxSubnetNodes(u32),
    SetMinStakeBalance(u128),
    SetTxRateLimit(u64),
    SetMaxZeroConsensusEpochs(u32),
    SetMinRequiredSubnetConsensusSubmitEpochs(u64),
    SetMinRequiredNodeConsensusSubmitEpochs(u64),
    SetMinRequiredNodeConsensusEpochs(u64),
		SetMinRequiredNodeAccountantEpochs(u64),
    SetMaximumOutlierDeltaPercent(u8),
    SetSubnetNodeConsensusSubmitPercentRequirement(u128),
    SetEpochLengthsInterval(u64),
    SetNodeRemovalThreshold(u128),
    SetMaxSubnetRewardsWeight(u128),
		SetStakeRewardWeight(u128),
		SetSubnetPerNodeInitCost(u128),
		SetSubnetConsensusUnconfirmedThreshold(u128),
		SetRemoveSubnetNodeEpochPercentage(u128),

		// Dishonesty Proposals
		DishonestSubnetNodeProposed { subnet_id: u32, account_id: T::AccountId, block: u64},
		DishonestSubnetNodeVote { subnet_id: u32, account_id: T::AccountId, voter_account_id: T::AccountId, block: u64 },
		DishonestAccountRemoved { subnet_id: u32, account_id: T::AccountId, block: u64},

		// Validation and Attestation
		ValidatorSubmission { subnet_id: u32, account_id: T::AccountId, epoch: u32},
		Attestation { subnet_id: u32, account_id: T::AccountId, epoch: u32},

		Slashing { subnet_id: u32, account_id: T::AccountId, amount: u128},
	}

	/// Errors that can be returned by this pallet.
	///
	/// Errors tell users that something went wrong so it's important that their naming is
	/// informative. Similar to events, error documentation is added to a node's metadata so it's
	/// equally important that they have helpful documentation associated with them.
	///
	/// This type of runtime error can be up to 4 bytes in size should you want to return additional
	/// information.
	#[pallet::error]
	pub enum Error<T> {
		/// Errors should have helpful documentation associated with them.

		/// Node hasn't been initialized for required epochs to submit consensus
		NodeConsensusSubmitEpochNotReached,
		/// Node hasn't been initialized for required epochs to be an accountant
		NodeAccountantEpochNotReached,
		/// Maximum subnets reached
		MaxSubnets,
		/// Account has subnet peer under subnet already
		SubnetNodeExist,
		/// Node ID already in use
		PeerIdExist,
		/// Node ID already in use
		PeerIdNotExist,
		/// Subnet peer doesn't exist
		SubnetNodeNotExist,
		/// Subnet already exists
		SubnetExist,
		/// Subnet doesn't exist
		SubnetNotExist,
		/// Minimum required subnet peers not reached
		SubnetNodesMin,
		/// Maximum allowed subnet peers reached
		SubnetNodesMax,
		/// Subnet has not been voted in
		SubnetNotVotedIn,
		/// Subnet not validated to be removed
		SubnetCantBeRemoved,
		/// Account is eligible
		AccountEligible,
		/// Account is ineligible
		AccountIneligible,
		// invalid submit consensus block
		/// Cannot submit consensus during invalid blocks
		InvalidSubmitEpochLength,
		/// Cannot remove subnet peer during invalid blocks
		InvalidRemoveOrUpdateSubnetNodeBlock,
		/// Transaction rate limiter exceeded
		TxRateLimitExceeded,
		/// PeerId format invalid
		InvalidPeerId,
		/// The provided signature is incorrect.
		WrongSignature,

		// Admin
		/// Consensus block epoch_length invalid, must reach minimum
		InvalidEpochLengthsInterval,
		/// Invalid maximimum subnets, must not exceed maximum allowable
		InvalidMaxSubnets,
		/// Invalid min subnet peers, must not be less than minimum allowable
		InvalidMinSubnetNodes,
		/// Invalid maximimum subnet peers, must not exceed maximimum allowable
		InvalidMaxSubnetNodes,
		/// Invalid minimum stake balance, must be greater than or equal to minimim required stake balance
		InvalidMinStakeBalance,
		/// Invalid percent number, must be in 1e4 format. Used for elements that only require correct format
		InvalidPercent,
		/// Invalid subnet peer consensus submit percent requirement
		InvalidSubnetNodeConsensusSubmitPercentRequirement,
		/// Invalid percent number based on MinSubnetNodes as `min_value = 1 / MinSubnetNodes`
		// This ensures it's possible to form consensus to remove peers
		InvalidNodeRemovalThreshold,
		/// Invalid maximimum zero consensus epochs, must not exceed maximum allowable
		InvalidMaxZeroConsensusEpochs,
		/// Invalid subnet consensus `submit` epochs, must be greater than 2 and greater than MinRequiredNodeConsensusSubmitEpochs
		InvalidSubnetConsensusSubmitEpochs,
		/// Invalid peer consensus `inclusion` epochs, must be greater than 0 and less than MinRequiredNodeConsensusSubmitEpochs
		InvalidNodeConsensusInclusionEpochs,
		/// Invalid peer consensus `submit` epochs, must be greater than 1 and greater than MinRequiredNodeConsensusInclusionEpochs
		InvalidNodeConsensusSubmitEpochs,
		/// Invalid peer consensus `dishonesty` epochs, must be greater than 2 and greater than MinRequiredNodeConsensusSubmitEpochs
		InvalidNodeConsensusDishonestyEpochs,
		/// Invalid max outlier delta percentage, must be in format convertible to f64
		InvalidMaxOutlierDeltaPercent,
		/// Invalid subnet per peer init cost, must be greater than 0 and less than 1000
		InvalidSubnetPerNodeInitCost,
		/// Invalid subnet consensus uncunfirmed threshold, must be in 1e4 format
		InvalidSubnetConsensusUnconfirmedThreshold,
		/// Invalid remove subnet peer epoch percentage, must be in 1e4 format and greater than 20.00
		InvalidRemoveSubnetNodeEpochPercentage,
		// staking
		/// u128 -> BalanceOf conversion error
		CouldNotConvertToBalance,
		/// Not enough balance on Account to stake and keep alive
		NotEnoughBalanceToStake,
		/// Required unstake epochs not met based on MinRequiredUnstakeEpochs
		RequiredUnstakeEpochsNotMet,
		/// Amount will kill account
		BalanceWithdrawalError,
		/// Not enough stake to withdraw
		NotEnoughStakeToWithdraw,
		MaxStakeReached,
		// if min stake not met on both stake and unstake
		MinStakeNotReached,
		// delegate staking
		CouldNotConvertToShares,
		// 
		MaxDelegatedStakeReached,
		//
		InsufficientCooldown,
		//
		UnstakeWindowFinished,
		//
		MaxUnlockingsPerEpochReached,
		//
		MaxUnlockingsReached,
		//
		NoDelegateStakeUnbondingsOrCooldownNotMet,
		//
		RequiredDelegateUnstakeEpochsNotMet,
		// Conversion to balance was zero
		InsufficientBalanceToSharesConversion,
		// consensus
		SubnetInitializeRequirement,
		ConsensusDataInvalidLen,
		/// Invalid consensus score, must be in 1e4 format and greater than 0
		InvalidScore,
		/// Consensus data already submitted
		ConsensusDataAlreadySubmitted,
		/// Consensus data already unconfirmed
		ConsensusDataAlreadyUnconfirmed,

		/// Math multiplication overflow
		MathMultiplicationOverflow,

		/// Dishonesty on subnet and account proposed
		DishonestyVoteAlreadyProposed,

		/// Dishonesty vote period already completed
		DishonestyVotePeriodCompleted,
		
		/// Dishonesty vote not proposed
		DishonestyVoteNotProposed,

		/// Dishonesty voting either not exists or voting period is over
		DishonestyVotingPeriodOver,

		/// Dishonesty voting not over
		DishonestyVotingPeriodNotOver,

		/// Dishonesty voting either not exists or voting period is over
		DishonestyVotingDuplicate,

		/// Not enough balance to withdraw bid for proposal
		NotEnoughBalanceToBid,

		QuorumNotReached,

		/// Dishonest propsal type
		PropsTypeInvalid,

		ProposalNotExist,
		ProposalNotChallenged,
		ProposalChallenged,
		ProposalChallengePeriodPassed,
		PropsalAlreadyChallenged,
		NotChallenger,
		ChallengePeriodPassed,
		DuplicateVote,
		NotAccountant,
		InvalidAccountantDataId,
		InvalidAccountantData,
		DataEmpty,

		InvalidSubnetRewardsSubmission,


		// Validation and Attestation
		/// Subnet rewards data already submitted by validator
		SubnetRewardsAlreadySubmitted,
		/// Not epoch validator
		InvalidValidator,
		/// Already attested validator data
		AlreadyAttested,
		/// Invalid rewards data length
		InvalidRewardsDataLength,
		/// Invalid block for submitting data
		InvalidBlock,


		ProposalInvalid,
		NotDefendant,
		NotPlaintiff,
		ProposalUnchallenged,
		/// Subnet node as defendant has proposal activated already
		NodeHasActiveProposal,
	}
	
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetNode<AccountId> {
		pub account_id: AccountId,
		pub peer_id: PeerId,
		pub initialized: u64,
	}

		// The submit consensus data format
	// Scoring is calculated off-chain between subnet peers hosting AI subnets together
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetNodeData {
		pub peer_id: PeerId,
		pub score: u128,
	}

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct RewardsData<AccountId> {
		pub validator: AccountId, // Chosen validator of the epoch
		pub nodes_count: u32, // Number of nodes expected to submit attestations
		pub sum: u128, // Sum of the data scores
		pub attests: BTreeSet<AccountId>, // Count of attestations of the submitted data
		pub data: Vec<SubnetNodeData>, // Data submitted by chosen validator
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub enum VoteType {
    Yay,
    Nay,
  }

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct PreSubnetData {
		pub path: Vec<u8>,
		pub memory_mb: u128,
	}
	
	/// Data for subnet held to be compared when adding a subnet to the network
	// This is the data from the democracy voting pallet
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct VoteSubnetData {
		pub data: PreSubnetData,
		pub active: bool,
	}

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetData {
		pub id: u32,
		pub path: Vec<u8>,
		pub min_nodes: u32,
		pub target_nodes: u32,
		pub memory_mb: u128,
		pub initialized: u64,
	}
	
	
	// `data` is an arbitrary vec of data for subnets to use for validation
	// It's up to each subnet to come up with their own format that fits within the BoundedVec
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct AccountantDataNodeParams {
		pub peer_id: PeerId,
		pub data: BoundedVec<u8, DefaultAccountantDataNodeParamsMaxLimit>,
	}

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct AccountantDataParams<AccountId> {
		pub accountant: AccountId,
		pub block: u64,
		pub epoch: u32,
		pub data: Vec<AccountantDataNodeParams>,
	}

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct VoteParams<AccountId> {
		pub yay: BTreeSet<AccountId>,
		pub nay: BTreeSet<AccountId>,
	}

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct ProposalParams<AccountId> {
		pub subnet_id: u32,
		pub plaintiff: AccountId,
		pub defendant: AccountId,
		pub plaintiff_bond: u128,
		pub defendant_bond: u128,
		pub eligible_voters: BTreeMap<AccountId, u64>, // Those eligible to vote at time of the proposal
		pub votes: VoteParams<AccountId>,
		pub start_block: u64,
		pub challenge_block: u64,
		pub plaintiff_data: Vec<u8>,
		pub defendant_data: Vec<u8>,
		pub complete: bool,
	}

	#[pallet::type_value]
	pub fn DefaultZeroU32() -> u32 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultAccountId<T: Config>() -> T::AccountId {
		T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap()
	}
	#[pallet::type_value]
	pub fn DefaultTxRateLimit<T: Config>() -> u64 {
		T::InitialTxRateLimit::get()
	}
	#[pallet::type_value]
	pub fn DefaultLastTxBlock() -> u64 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultVoteSubnetData() -> VoteSubnetData {
		let pre_subnet_data = PreSubnetData {
			path: Vec::new(),
			memory_mb: 0,
		};
		return VoteSubnetData {
			data: pre_subnet_data,
			active: false,
		}
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetPenaltyCount() -> u32 {
		100
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodesClasses<T: Config>() -> BTreeMap<T::AccountId, u64> {
		BTreeMap::new()
	}
	#[pallet::type_value]
	pub fn DefaultMinRequiredSubnetConsensusSubmitEpochs() -> u64 {
		4
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNode<T: Config>() -> SubnetNode<T::AccountId> {
		return SubnetNode {
			account_id: T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap(),
			peer_id: PeerId(Vec::new()),
			initialized: 0,
		};
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetNodes() -> u32 {
		96
	}
	#[pallet::type_value]
	pub fn DefaultMaxAccountPenaltyCount() -> u32 {
		12
	}
	#[pallet::type_value]
	pub fn DefaultAccountPenaltyCount() -> u32 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultMinRequiredUnstakeEpochs() -> u64 {
		12
	}
	#[pallet::type_value]
	pub fn DefaultAccountTake() -> u128 {
		0
	}
		#[pallet::type_value]
	pub fn DefaultMaxStakeBalance() -> u128 {
		280000000000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultMinStakeBalance() -> u128 {
		1000e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodeClassEpochs() -> u64 {
		2
	}
	#[pallet::type_value]
	pub fn DefaultMaxDelegateStakeBalance() -> u128 {
		280000000000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultMinDelegateStakeBalance() -> u128 {
		1000e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultDelegateStakeRewardsPercentage() -> u128 {
		1100
	}
	#[pallet::type_value]
	pub fn DefaultMinRequiredDelegateUnstakeEpochs() -> u64 {
		21
	}
	#[pallet::type_value]
	pub fn DefaultDelegateStakeCooldown() -> u64 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultDelegateStakeUnbondingLedger() -> BTreeMap<u64, u128> {
		// {
		// 	epoch_start: u64, // cooldown begin epoch (+ cooldown duration for unlock)
		// 	shares: u128,
		// }
		BTreeMap::new()
	}
	#[pallet::type_value]
	pub fn DefaultBaseRewardPerMB() -> u128 {
		1e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultBaseReward() -> u128 {
		1e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultBaseSubnetReward() -> u128 {
		9e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMaxSequentialAbsentSubnetNode() -> u32 {
		// Must be less than the inclusion stage in the subnet node validation sequence
		// This ensures a subnet node is inducted via consensus before they can become a validator
		3
	}
	#[pallet::type_value]
	pub fn DefaultSlashPercentage() -> u128 {
		312
	}
	#[pallet::type_value]
	pub fn DefaultMaxSlashAmount() -> u128 {
		1e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMinAttestationPercentage() -> u128 {
		6600
	}
	#[pallet::type_value]
	pub fn DefaultTargetSubnetNodesMultiplier() -> u128 {
		3333
	}
	#[pallet::type_value]
	pub fn DefaultBaseSubnetNodeMemoryMB() -> u128 {
		16000
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetMemoryMB() -> u128 {
		500000
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetNodes() -> u32 {
		3
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnets() -> u32 {
		32
	}
	#[pallet::type_value]
	pub fn DefaultAccountantDataNodeParamsMaxLimit() -> u32 {
		1024_u32
	}

	/// Count of subnets
	#[pallet::storage]
	#[pallet::getter(fn total_models)]
	pub type TotalSubnets<T> = StorageValue<_, u32, ValueQuery>;
	
	#[pallet::storage]
	#[pallet::getter(fn max_models)]
	pub type MaxSubnets<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnets>;

	// Mapping of each subnet stored by ID, uniqued by `SubnetPaths`
	// Stores subnet data by a unique id
	#[pallet::storage] // subnet_id => data struct
	pub type SubnetsData<T: Config> = StorageMap<_, Blake2_128Concat, u32, SubnetData>;

	// Ensures no duplicate subnet paths within the network at one time
	// If a subnet path is voted out, it can be voted up later on and any
	// stakes attached to the subnet_id won't impact the re-initialization
	// of the subnet path.
	#[pallet::storage]
	pub type SubnetPaths<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, u32>;

	#[pallet::storage] // subnet ID => account_id
	pub type SubnetActivated<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		VoteSubnetData,
		ValueQuery,
		DefaultVoteSubnetData,
	>;

	// Minimum amount of peers required per subnet
	// required for subnet activity
	#[pallet::storage]
	#[pallet::getter(fn min_subnet_nodes)]
	pub type MinSubnetNodes<T> = StorageValue<_, u32, ValueQuery, DefaultMinSubnetNodes>;

	// Maximim peers in a subnet at any given time
	#[pallet::storage]
	#[pallet::getter(fn max_subnet_nodes)]
	pub type MaxSubnetNodes<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnetNodes>;

	// Max epochs where consensus isn't formed before subnet being removed
	#[pallet::storage]
	pub type MaxSubnetPenaltyCount<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnetPenaltyCount>;
	
	// Count of epochs a subnet has consensus errors
	#[pallet::storage] // subnet_id => count
	pub type SubnetPenaltyCount<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		u32,
		ValueQuery,
	>;
	
	#[pallet::storage] // model_uid --> peer_data
	#[pallet::getter(fn total_subnet_nodes)]
	pub type TotalSubnetNodes<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u32, ValueQuery>;

	// Epochs required from subnet initialization block to accept consensus submissions
	// Epochs required based on EpochLength
	// Each epoch is EpochLength
	// Min required epochs for a subnet to be in storage for based on initialized
	#[pallet::storage]
	#[pallet::getter(fn min_required_model_consensus_submit_epochs)]
	pub type MinRequiredSubnetConsensusSubmitEpochs<T> = StorageValue<_, u64, ValueQuery, DefaultMinRequiredSubnetConsensusSubmitEpochs>;

	#[pallet::storage] // subnet_id --> account_id --> data
	#[pallet::getter(fn subnet_nodes)]
	pub type SubnetNodesData<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		T::AccountId,
		SubnetNode<T::AccountId>,
		ValueQuery,
		DefaultSubnetNode<T>,
	>;
	
	// Used for unique peer_ids
	#[pallet::storage] // subnet_id --> account_id --> peer_id
	#[pallet::getter(fn subnet_node_account)]
	pub type SubnetNodeAccount<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		PeerId,
		T::AccountId,
		ValueQuery,
		DefaultAccountId<T>,
	>;
		
	#[derive(EnumIter, FromRepr, Copy, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub enum SubnetNodeClass {
    Idle,
    Included,
		Processing,
		Submittable,
		Accountant
  }

	impl SubnetNodeClass {
    pub fn index(&self) -> usize {
			*self as usize
    }
	}

	// How many epochs until an account can reach the next node class
	// e.g. Idle 			2 epochs => account must be Idle for 2 epochs from their initialization epoch
	//			Included	2 epochs => account must be Included for 2 epochs from their initialization epoch
	#[pallet::storage] // subnet => account_id
	pub type SubnetNodeClassEpochs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		SubnetNodeClass,
		u64,
		ValueQuery,
		DefaultSubnetNodeClassEpochs
	>;

	#[pallet::storage] // subnet_id -> class_id -> BTreeMap(account_id, block)
	pub type SubnetNodesClasses<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		SubnetNodeClass,
		BTreeMap<T::AccountId, u64>,
		ValueQuery,
		DefaultSubnetNodesClasses<T>,
	>;

	#[pallet::storage] // subnet_id --> (account_id, (initialized or removal block))
	pub type SubnetAccount<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		BTreeMap<T::AccountId, u64>,
		ValueQuery,
	>;

	// Maximum subnet peer penalty count
	#[pallet::storage]
	pub type MaxAccountPenaltyCount<T> = StorageValue<_, u32, ValueQuery, DefaultMaxAccountPenaltyCount>;

	// Count of times a peer is against consensus
	// This includes:
	// 1. being against other peers that conclude another peer is out of consensus
	// 2. being against other peers that conclude another peer is in consensus
	// 3. score delta is too high on consensus data submission
	// 4. not submitting consensus data
	#[pallet::storage] // account_id --> u32
	#[pallet::getter(fn subnet_node_penalty_count)]
	pub type AccountPenaltyCount<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		u32,
		ValueQuery,
		DefaultAccountPenaltyCount
	>;

	#[pallet::storage]
	pub type BaseSubnetNodeMemoryMB<T> = StorageValue<_, u128, ValueQuery, DefaultBaseSubnetNodeMemoryMB>;

	#[pallet::storage]
	pub type MaxSubnetMemoryMB<T> = StorageValue<_, u128, ValueQuery, DefaultMaxSubnetMemoryMB>;


	#[pallet::storage]
	pub type TargetSubnetNodesMultiplier<T> = StorageValue<_, u128, ValueQuery, DefaultTargetSubnetNodesMultiplier>;

	// Tracks each subnet an account is a subnet peer on
	// This is used as a helper when removing accounts from all subnets they are peers on
	#[pallet::storage] // account_id --> [model_ids]
	pub type AccountSubnets<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<u32>, ValueQuery>;


	// Rate limit
	#[pallet::storage] // ( tx_rate_limit )
	pub type TxRateLimit<T> = StorageValue<_, u64, ValueQuery, DefaultTxRateLimit<T>>;

	// Last transaction on rate limited functions
	#[pallet::storage] // key --> last_block
	pub type LastTxBlock<T: Config> =
		StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultLastTxBlock>;


	//
	// Validate / Attestation
	//

	// The account responsible for validating the epochs rewards data
	#[pallet::storage] // subnet ID => epoch  => data
	pub type SubnetRewardsValidator<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		u32,
		T::AccountId,
	>;

	#[pallet::storage] // subnet ID => epoch  => data
	pub type SubnetRewardsSubmission<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		u32,
		RewardsData<T::AccountId>,
	>;

	#[pallet::storage]
	pub type MinAttestationPercentage<T> = StorageValue<_, u128, ValueQuery, DefaultMinAttestationPercentage>;


	// Rewards

	// Base reward per subnet
	#[pallet::storage]
	pub type BaseSubnetReward<T> = StorageValue<_, u128, ValueQuery, DefaultBaseSubnetReward>;
	
	// TODO: BaseReward → BaseValidatorReward
	// Base reward per epoch for validators and accountants
	// This is the base reward to subnet validators on successful attestation
	// This is the base reward to accountants when they agree to validation data.?
	#[pallet::storage]
	pub type BaseReward<T> = StorageValue<_, u128, ValueQuery, DefaultBaseReward>;

	// Base reward per MB per epoch based on 4,380 MB per year
	#[pallet::storage]
	pub type BaseRewardPerMB<T> = StorageValue<_, u128, ValueQuery, DefaultBaseRewardPerMB>;

	#[pallet::storage]
	pub type SlashPercentage<T> = StorageValue<_, u128, ValueQuery, DefaultSlashPercentage>;

	#[pallet::storage]
	pub type MaxSlashAmount<T> = StorageValue<_, u128, ValueQuery, DefaultMaxSlashAmount>;

	#[pallet::storage]
	pub type MaxSequentialAbsentSubnetNode<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSequentialAbsentSubnetNode>;

	// If subnet node is absent from inclusion in consensus information or attestings
	#[pallet::storage] // subnet_id -> class_id -> BTreeMap(account_id, block)
	pub type SequentialAbsentSubnetNode<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		T::AccountId,
		u32,
		ValueQuery,
		DefaultZeroU32,
	>;

	#[pallet::type_value]
	pub fn DefaultNodeConsensusRemovalThreshold() -> u128 {
		8500
	}

	// Attestion percentage required to increment a nodes penalty count up
	#[pallet::storage]
	pub type NodeConsensusRemovalThreshold<T: Config> = StorageValue<_, u128, ValueQuery, DefaultNodeConsensusRemovalThreshold>;


	//
	// Staking
	// 
	#[pallet::storage] // stores epoch balance of rewards from block rewards to be distributed to peers/stakers
	#[pallet::getter(fn stake_vault_balance)]
	pub type StakeVaultBalance<T> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage] // ( total_stake )
	#[pallet::getter(fn total_stake)]
	pub type TotalStake<T: Config> = StorageValue<_, u128, ValueQuery>;

	// Total stake sum of all peers in specified subnet
	#[pallet::storage] // model_uid --> peer_data
	#[pallet::getter(fn total_model_stake)]
	pub type TotalSubnetStake<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u128, ValueQuery>;

	// An accounts stake per subnet
	#[pallet::storage] // account--> subnet_id --> u128
	#[pallet::getter(fn account_model_stake)]
	pub type AccountSubnetStake<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Identity,
		u32,
		u128,
		ValueQuery,
		DefaultAccountTake,
	>;

	// Amount of epochs for removed subnets peers required to unstake
	#[pallet::storage]
	pub type MinRequiredUnstakeEpochs<T> = StorageValue<_, u64, ValueQuery, DefaultMinRequiredUnstakeEpochs>;
	
	// An accounts stake across all subnets
	#[pallet::storage] // account_id --> all subnets balance
	#[pallet::getter(fn total_account_stake)]
	pub type TotalAccountStake<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u128, ValueQuery>;

	// Maximum stake balance per subnet
	// Only checked on `do_add_stake` and ``
	// A subnet staker can have greater than the max stake balance although any rewards
	// they would receive based on their stake balance will only account up to the max stake balance allowed
	#[pallet::storage]
	pub type MaxStakeBalance<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMaxStakeBalance>;

	// Minimum required subnet peer stake balance per subnet
	#[pallet::storage]
	pub type MinStakeBalance<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMinStakeBalance>;

	// Delegate Staking
	#[pallet::storage]
	pub type MaxDelegateStakeBalance<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMaxDelegateStakeBalance>;

	#[pallet::storage]
	pub type MinDelegateStakeBalance<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMinDelegateStakeBalance>;

	#[pallet::storage] // subnet_id --> (account_id, (initialized or removal block))
	pub type SubnetAccountDelegateStake<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		BTreeMap<T::AccountId, u64>,
		ValueQuery,
	>;

	// Percentage of epoch rewards that go towards delegate stake pools
	#[pallet::storage]
	pub type DelegateStakeRewardsPercentage<T: Config> = StorageValue<_, u128, ValueQuery, DefaultDelegateStakeRewardsPercentage>;

	#[pallet::storage]
	pub type MinRequiredDelegateUnstakeEpochs<T> = StorageValue<_, u64, ValueQuery, DefaultMinRequiredDelegateUnstakeEpochs>;

	// Total stake sum of all peers in specified subnet
	#[pallet::storage] // model_uid --> peer_data
	pub type TotalSubnetDelegateStakeShares<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u128, ValueQuery>;

	// Total stake sum of all peers in specified subnet
	#[pallet::storage] // model_uid --> peer_data
	pub type TotalSubnetDelegateStakeBalance<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u128, ValueQuery>;

	// An accounts stake per subnet
	#[pallet::storage] // account --> subnet_id --> u128
	pub type AccountSubnetDelegateStakeShares<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Identity,
		u32,
		u128,
		ValueQuery,
		DefaultAccountTake,
	>;

	// An accounts stake per subnet
	#[pallet::storage] // account --> subnet_id --> u64
	pub type DelegateStakeCooldown<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Identity,
		u32,
		u64,
		ValueQuery,
		DefaultDelegateStakeCooldown,
	>;

	// An accounts stake per subnet
	#[pallet::storage] // account --> subnet_id --> u64
	pub type DelegateStakeUnbondingLedger<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Identity,
		u32,
		BTreeMap<u64, u128>,
		ValueQuery,
		DefaultDelegateStakeUnbondingLedger,
	>;
	
	//
	// Accountants
	//

	#[pallet::type_value]
	pub fn DefaultTargetAccountantsLength() -> u32 {
		2
	}

	#[pallet::storage]
	pub type TargetAccountantsLength<T> = StorageValue<_, u32, ValueQuery, DefaultTargetAccountantsLength>;

	#[pallet::storage] // subnet ID => epoch  => data
	pub type CurrentAccountants<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		u32,
		BTreeMap<T::AccountId, bool>,
	>;

	// Index for AccountantData
	#[pallet::storage]
	pub type AccountantDataCount<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		u32,
		ValueQuery,
	>;

	#[pallet::type_value]
	pub fn DefaultAccountantData<T: Config>() -> AccountantDataParams<T::AccountId> {
		return AccountantDataParams {
			accountant: T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap(),
			block: 0,
			epoch: 0,
			data: Vec::new(),
		};
	}

	#[pallet::storage] // subnet ID => data_id => data
	pub type AccountantData<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		u32,
		AccountantDataParams<T::AccountId>,
		ValueQuery,
		DefaultAccountantData<T>,
	>;

	//
	// Props
	//
	#[pallet::type_value]
	pub fn DefaultProposalParams<T: Config>() -> ProposalParams<T::AccountId> {
		return ProposalParams {
			subnet_id: 0,
			plaintiff: T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap(),
			defendant: T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap(),
			plaintiff_bond: 0,
			defendant_bond: 0,
			eligible_voters: BTreeMap::new(),
			votes: VoteParams {
				yay: BTreeSet::new(),
				nay: BTreeSet::new(),
			},
			start_block: 0,
			challenge_block: 0,
			plaintiff_data: Vec::new(),
			defendant_data: Vec::new(),
			complete: false,
		};
	}

	#[pallet::storage] // subnet => proposal_id => proposal
	pub type Proposals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		u32,
		ProposalParams<T::AccountId>,
		ValueQuery,
		DefaultProposalParams<T>,
	>;

	#[pallet::type_value]
	pub fn DefaultProposalsCount() -> u32 {
		0
	}

	#[pallet::storage] 
	pub type ProposalsCount<T> = StorageValue<_, u32, ValueQuery, DefaultProposalsCount>;

	#[pallet::type_value]
	pub fn DefaultProposalBidAmount() -> u128 {
		1e+18 as u128
	}

	// Amount required to put up as a proposer and challenger
	#[pallet::storage] 
	pub type ProposalBidAmount<T> = StorageValue<_, u128, ValueQuery, DefaultProposalBidAmount>;

	#[pallet::type_value]
	pub fn DefaultVotingPeriod() -> u64 {
		// 7 days
		100800
	}

	#[pallet::storage] // Period in blocks for votes after challenge
	pub type VotingPeriod<T> = StorageValue<_, u64, ValueQuery, DefaultVotingPeriod>;

	#[pallet::type_value]
	pub fn DefaultChallengePeriod() -> u64 {
		// 7 days in blocks
		100800
	}

	#[pallet::storage] // Period in blocks after proposal to challenge proposal
	pub type ChallengePeriod<T> = StorageValue<_, u64, ValueQuery, DefaultChallengePeriod>;

	#[pallet::type_value]
	pub fn DefaultProposalQuorum() -> u128 {
		// 7 days
		6600
	}

	#[pallet::storage] // How many votes are needed
	pub type ProposalQuorum<T> = StorageValue<_, u128, ValueQuery, DefaultProposalQuorum>;

	#[pallet::type_value]
	pub fn DefaultDishonestyProposalConsensusThreshold() -> u128 {
		// 90%
		9000
	}

	// Consensus required to pass proposal
	#[pallet::storage]
	pub type ProposalConsensusThreshold<T> = StorageValue<_, u128, ValueQuery, DefaultDishonestyProposalConsensusThreshold>;

	/// The pallet's dispatchable functions ([`Call`]s).
	///
	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsics", which are often compared to transactions.
	/// They must always return a `DispatchResult` and be annotated with a weight and call index.
	///
	/// The [`call_index`] macro is used to explicitly
	/// define an index for calls in the [`Call`] enum. This is useful for pallets that may
	/// introduce new dispatchables over time. If the order of a dispatchable changes, its index
	/// will also change which will break backwards compatibility.
	///
	/// The [`weight`] macro is used to assign a weight to each call.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn remove_subnet(
			origin: OriginFor<T>, 
			subnet_id: u32,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

			ensure!(
				SubnetsData::<T>::contains_key(subnet_id),
				Error::<T>::SubnetNotExist
			);

			let subnet = SubnetsData::<T>::get(subnet_id).unwrap();
			let subnet_path: Vec<u8> = subnet.path;

			// ----
			// Subnets can be removed by
			// 		1. Subnet can be voted off
			//		2. Subnet can reach max zero consensus count
			//		3. Subnet can be offline too many times
			//		4. Subnet has min peers after initialization period
			// ----

			// 1.
			// Check subnet voted out
			let activated: bool = match SubnetActivated::<T>::try_get(subnet_path.clone()) {
				Ok(data) => data.active,
				Err(()) => false,
			};

			let mut reason_for_removal: Vec<u8> = Vec::new();
			
			// Push into reason
			if !activated {
				reason_for_removal.push(1)
			}

			// 2.
			// Subnet can reach max zero consensus count
			let subnet_penalty_count: u32 = SubnetPenaltyCount::<T>::get(subnet_id);
			let max_subnet_penalty_count: u32 = MaxSubnetPenaltyCount::<T>::get();
			let surpassed_max_penalty_count: bool = subnet_penalty_count > max_subnet_penalty_count;

			// Push into reason
			if surpassed_max_penalty_count {
				reason_for_removal.push(2)
			}

			// 3.
			// Check if subnet is offline too many times
			let is_offline: bool = false;

			// Push into reason
			if is_offline {
				reason_for_removal.push(3)
			}

			// 4.
			// Check if subnet has min amount of peers
			// If min peers are not met and initialization epochs has surpassed
			// then subnet can be removed
			let total_subnet_nodes: u32 = TotalSubnetNodes::<T>::get(subnet_id);
			let min_subnet_nodes: u32 = subnet.min_nodes;
			let block: u64 = Self::get_current_block_as_u64();
			let mut has_min_peers: bool = true;
			if total_subnet_nodes < min_subnet_nodes {
				let model_initialized: u64 = subnet.initialized;
				let epoch_length: u64 = T::EpochLength::get();
				let min_required_model_consensus_submit_epochs = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();
				// Ensure initialization epochs have passed
				// If not return false
				let initialized: bool = block < Self::get_eligible_epoch_block(
					epoch_length, 
					model_initialized, 
					min_required_model_consensus_submit_epochs
				);
				// Push into reason
				if !initialized {
					reason_for_removal.push(4)
				}	
			}

			// Must have at least one of the possible reasons to be removed
			ensure!(
				!activated || surpassed_max_penalty_count || is_offline || !has_min_peers,
				Error::<T>::SubnetCantBeRemoved
			);

			// Remove unique path
			SubnetPaths::<T>::remove(subnet_path.clone());
			// Remove subnet data
			SubnetsData::<T>::remove(subnet_id);

			// We don't subtract TotalSubnets since it's used for ids

			// Remove all peers data
			let _ = SubnetNodesData::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let _ = TotalSubnetNodes::<T>::remove(subnet_id);
			let _ = SubnetNodeAccount::<T>::clear_prefix(subnet_id, u32::MAX, None);

			// Remove all subnet consensus data
			let _ = SubnetPenaltyCount::<T>::remove(subnet_id);
			let _ = SubnetNodesClasses::<T>::clear_prefix(subnet_id, u32::MAX, None);
	
			// Self::deposit_event(Event::SubnetRemoved { 
			// 	account: account_id, 
			// 	subnet_id: subnet_id, 
			// 	subnet_path: subnet_path.clone(),
			// 	reason: reason_for_removal,
			// 	block: block
			// });

			Ok(())
		}

				/// Add a subnet peer that is currently hosting an AI subnet (or a peer in DHT)
		/// A minimum stake balance is required
		// Before adding subnet peer you must become a peer hosting the subnet of choice
		// This fn will claim your peer_id and associate it with your account as peer_id => account_id
		// If this reverts due to `SubnetNodeExist` you must remove the peer node and try again with a new peer_id
		// It's possible someone can claim the peer_id before you do
		// due to the requirement of staking this is an unlikely scenario.
		// Once you claim the peer_id, no one else can claim it.
		// After RequiredSubnetNodeEpochs pass and the peer is in consensus, rewards will be emitted to the account
		#[pallet::call_index(1)]
		// #[pallet::weight(T::WeightInfo::add_subnet_node())]
		#[pallet::weight({0})]
		pub fn add_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			peer_id: PeerId, 
			stake_to_be_added: u128,
			// signature: T::OffchainSignature,
			// signer: T::AccountId,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin.clone())?;

			let epoch_length: u64 = T::EpochLength::get();
			let block: u64 = Self::get_current_block_as_u64();

			ensure!(
				SubnetsData::<T>::contains_key(subnet_id),
				Error::<T>::SubnetNotExist
			);

			// Ensure account is eligible
			ensure!(
				Self::is_account_eligible(account_id.clone()),
				Error::<T>::AccountIneligible
			);
			
			// Ensure max peers isn't surpassed
			let total_subnet_nodes: u32 = TotalSubnetNodes::<T>::get(subnet_id);
			let max_subnet_nodes: u32 = MaxSubnetNodes::<T>::get();
			ensure!(
				total_subnet_nodes < max_subnet_nodes,
				Error::<T>::SubnetNodesMax
			);

			// Unique subnet_id -> AccountId
			// Ensure account doesn't already have a peer within subnet
			ensure!(
				!SubnetNodesData::<T>::contains_key(subnet_id, account_id.clone()),
				Error::<T>::SubnetNodeExist
			);

			// Unique subnet_id -> PeerId
			// Ensure peer ID doesn't already exist within subnet regardless of account_id
			let peer_exists: bool = match SubnetNodeAccount::<T>::try_get(subnet_id, peer_id.clone()) {
				Ok(_) => true,
				Err(()) => false,
			};

			ensure!(
				!peer_exists,
				Error::<T>::PeerIdExist
			);

			// Validate peer_id
			ensure!(
				Self::validate_peer_id(peer_id.clone()),
				Error::<T>::InvalidPeerId
			);

			// ====================
			// Initiate stake logic
			// ====================
			Self::do_add_stake(
				origin.clone(), 
				subnet_id,
				account_id.clone(),
				stake_to_be_added,
			).map_err(|e| e)?;

			// To ensure the AccountId that owns the PeerId, they must sign the PeerId for others to verify
			// This ensures others cannot claim to own a PeerId they are not the owner of
			// Self::validate_signature(&Encode::encode(&peer_id), &signature, &signer)?;

			// ========================
			// Insert peer into storage
			// ========================
			let subnet_node: SubnetNode<T::AccountId> = SubnetNode {
				account_id: account_id.clone(),
				peer_id: peer_id.clone(),
				initialized: block,
			};
			// Insert SubnetNodesData with account_id as key
			SubnetNodesData::<T>::insert(subnet_id, account_id.clone(), subnet_node);

			// Insert subnet peer account to keep peer_ids unique within subnets
			SubnetNodeAccount::<T>::insert(subnet_id, peer_id.clone(), account_id.clone());

			// Insert unstaking reinforcements
			// This data is specifically used for allowing unstaking after being removed
			// SubnetAccount is not removed from storage until the peer has unstaked their entire stake balance
			// This stores the block they are initialized at
			// If removed, the initialized block will be replace with the removal block
			let mut model_accounts: BTreeMap<T::AccountId, u64> = SubnetAccount::<T>::get(subnet_id);
			// let model_account: Option<&u64> = model_accounts.get(&account_id.clone());
			let block_initialized_or_removed: u64 = match model_accounts.get(&account_id.clone()) {
				Some(block_initialized_or_removed) => *block_initialized_or_removed,
				None => 0,
			};

			// If previously removed or removed themselves
			// Ensure they have either unstaked or have waited enough epochs to unstake
			// to readd themselves as a subnet peer
			if block_initialized_or_removed != 0 {
				let min_required_unstake_epochs = MinRequiredUnstakeEpochs::<T>::get();
				// Ensure min required epochs have surpassed to unstake
				// Based on either initialized block or removal block
				ensure!(
					block >= Self::get_eligible_epoch_block(
						epoch_length, 
						block_initialized_or_removed, 
						min_required_unstake_epochs
					),
					Error::<T>::RequiredUnstakeEpochsNotMet
				);	
			}

			// Update to current block
			model_accounts.insert(account_id.clone(), block);
			SubnetAccount::<T>::insert(subnet_id, model_accounts);

			if let Ok(mut node_class) = SubnetNodesClasses::<T>::try_get(subnet_id, SubnetNodeClass::Idle) {
				node_class.insert(account_id.clone(), block);
				SubnetNodesClasses::<T>::insert(subnet_id, SubnetNodeClass::Idle, node_class);	
			} else {
				// If new subnet, initialize classes
				let mut node_class: BTreeMap<T::AccountId, u64> = BTreeMap::new();
				node_class.insert(account_id.clone(), block);
				SubnetNodesClasses::<T>::insert(subnet_id, SubnetNodeClass::Idle, node_class);	
			}

			// Add subnet_id to account
			// Account can only have a subnet peer per subnet so we don't check if it exists
			AccountSubnets::<T>::append(account_id.clone(), subnet_id);

			// Increase total subnet peers
			TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

			Self::deposit_event(
				Event::SubnetNodeAdded { 
					subnet_id: subnet_id, 
					account_id: account_id, 
					peer_id: peer_id,
					block: block
				}
			);

			Ok(())
		}

		/// Update a subnet peer
		// TODO: Track removed and updated nodes
		#[pallet::call_index(2)]
		// #[pallet::weight(T::WeightInfo::update_subnet_node())]
		#[pallet::weight({0})]
		pub fn update_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			peer_id: PeerId,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

			// --- Ensure subnet exists
			ensure!(
				SubnetsData::<T>::contains_key(subnet_id),
				Error::<T>::SubnetNotExist
			);

			ensure!(
				SubnetNodesData::<T>::contains_key(subnet_id, account_id.clone()),
				Error::<T>::SubnetNodeNotExist
			);
			
			// Unique subnet_id -> PeerId
			// Ensure peer ID doesn't already exist within subnet regardless of account_id
			let peer_exists: bool = match SubnetNodeAccount::<T>::try_get(subnet_id, peer_id.clone()) {
				Ok(_) => true,
				Err(()) => false,
			};

			ensure!(
				!peer_exists,
				Error::<T>::PeerIdExist
			);

			// Validate peer_id
			ensure!(
				Self::validate_peer_id(peer_id.clone()),
				Error::<T>::InvalidPeerId
			);
				
			let subnet_node = SubnetNodesData::<T>::get(subnet_id, account_id.clone());

			let block: u64 = Self::get_current_block_as_u64();
			let epoch_length: u64 = T::EpochLength::get();
			let submit_epochs = SubnetNodeClassEpochs::<T>::get(SubnetNodeClass::Submittable);

			// Check if subnet peer is eligible for consensus submission
			//
			// Updating a peer_id can only be done if the subnet peer can submit consensus data
			// Otherwise they must remove their peer and start a new one
			//
			// This is a backup incase subnets go down and subnet hosters all need to spin up
			// new nodes under new peer_id's
			ensure!(
				Self::is_epoch_block_eligible(
					block, 
					epoch_length, 
					submit_epochs, 
					subnet_node.initialized
				),
				Error::<T>::NodeConsensusSubmitEpochNotReached
			);

			// ====================
			// Mutate peer_id into storage
			// ====================
			SubnetNodesData::<T>::mutate(
				subnet_id,
				account_id.clone(),
				|params: &mut SubnetNode<T::AccountId>| {
					params.peer_id = peer_id.clone();
				}
			);

			// Update unique subnet peer_id
			SubnetNodeAccount::<T>::insert(subnet_id, peer_id.clone(), account_id.clone());

			Self::deposit_event(
				Event::SubnetNodeUpdated { 
					subnet_id: subnet_id, 
					account_id: account_id, 
					peer_id: peer_id,
					block: block
				}
			);

			Ok(())
		}
		

		/// Remove your subnet peer
		/// Unstaking must be done seperately
		#[pallet::call_index(3)]
		// #[pallet::weight(T::WeightInfo::remove_subnet_node())]
		#[pallet::weight({0})]
		pub fn remove_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

			let epoch_length: u64 = T::EpochLength::get();
			let block: u64 = Self::get_current_block_as_u64();

			// --- Ensure subnet exists
			ensure!(
				SubnetsData::<T>::contains_key(subnet_id),
				Error::<T>::SubnetNotExist
			);

			ensure!(
				SubnetNodesData::<T>::contains_key(subnet_id, account_id.clone()),
				Error::<T>::SubnetNodeNotExist
			);

			// TODO: Track removal of subnet nodes following validator consensus data submission per epoch

			// We don't check consensus steps here because a subnet peers stake isn't included in calculating rewards 
			// that hasn't reached their consensus submission epoch yet
			
			Self::do_remove_subnet_node(block, subnet_id, account_id.clone());

			Ok(())
		}

		/// Remove a subnet peer that has surpassed the max penalties allowed
		// This is redundant 
		#[pallet::call_index(4)]
		#[pallet::weight({0})]
		pub fn remove_account_subnet_nodes(
			origin: OriginFor<T>, 
			account_id: T::AccountId, 
		) -> DispatchResult {
			ensure_signed(origin)?;

			let block: u64 = Self::get_current_block_as_u64();

			// We can skip `can_remove_or_update_subnet_node` because they should not be
			// included in consensus data

			// Ensure account is not eligible to be a subnet peer
			ensure!(
				!Self::is_account_eligible(account_id.clone()),
				Error::<T>::AccountEligible
			);

			Self::do_remove_account_subnet_nodes(block, account_id);

			Ok(())
		}

		
		/// Increase stake towards the specified subnet ID
		#[pallet::call_index(5)]
		// #[pallet::weight(T::WeightInfo::add_to_stake())]
		#[pallet::weight({0})]
		pub fn add_to_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			stake_to_be_added: u128,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin.clone())?;
			// Each account can only have one peer
			// Staking is accounted for per account_id per subnet_id
			// We only check that origin exists within SubnetNodesData

			let epoch_length: u64 = T::EpochLength::get();
			let block: u64 = Self::get_current_block_as_u64();

			// --- Ensure subnet exists
			ensure!(
				SubnetsData::<T>::contains_key(subnet_id),
				Error::<T>::SubnetNotExist
			);

			// --- Ensure account has peer
			ensure!(
				SubnetNodesData::<T>::contains_key(subnet_id, account_id.clone()),
				Error::<T>::SubnetNodeNotExist
			);
			
			Self::do_add_stake(
				origin, 
				subnet_id,
				account_id.clone(),
				stake_to_be_added,
			)
		}

			/// Remove stake balance
		/// If account is a current subnet peer on the subnet ID they can only remove up to minimum required balance
		// Decrease stake on accounts peer if minimum required isn't surpassed
		// to-do: if removed through consensus, add removed_block to storage and require time 
		//				to pass until they can remove their stake
		#[pallet::call_index(6)]
		// #[pallet::weight(T::WeightInfo::remove_stake())]
		#[pallet::weight({0})]
		pub fn remove_stake(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			stake_to_be_removed: u128
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin.clone())?;

      // Get SubnetAccount (this is not deleted until stake == 0)
			let model_accounts: BTreeMap<T::AccountId, u64> = SubnetAccount::<T>::get(subnet_id);

			// Check if removed all stake yet
			let has_model_account: bool = match model_accounts.get(&account_id.clone()) {
				Some(_) => true,
				None => false,
			};

			// If SubnetAccount doesn't exist this means they have been removed due their staking balance is at zero
			// Once balance is at zero they are removed from SubnetAccount in `do_remove_stake()`
			ensure!(
				has_model_account,
				Error::<T>::SubnetNodeNotExist
			);

			let block_initialized_or_removed: u64 = match model_accounts.get(&account_id.clone()) {
				Some(block_initialized_or_removed) => *block_initialized_or_removed,
				None => 0,
			};
			let min_required_unstake_epochs = MinRequiredUnstakeEpochs::<T>::get();

			let epoch_length: u64 = T::EpochLength::get();
			let block: u64 = Self::get_current_block_as_u64();

			// Ensure min required epochs have surpassed to unstake
			// Based on either initialized block or removal block
			ensure!(
				block >= Self::get_eligible_epoch_block(
					epoch_length, 
					block_initialized_or_removed, 
					min_required_unstake_epochs
				),
				Error::<T>::RequiredUnstakeEpochsNotMet
			);

			// If account is a peer they can remove stake up to minimum required stake balance
			// Else they can remove entire balance because they are not hosting subnets according to consensus
			//		They are removed in `do_remove_subnet_node()` when self or consensus removed
			let is_peer: bool = match SubnetNodesData::<T>::try_get(subnet_id, account_id.clone()) {
				Ok(_) => true,
				Err(()) => false,
			};

			// Remove stake
			// 		if_peer: cannot remove stake below minimum required stake
			// 		else: can remove total stake balance
			// if balance is zero then SubnetAccount is removed
			Self::do_remove_stake(
				origin, 
				subnet_id,
				account_id,
				is_peer,
				stake_to_be_removed,
			)
		}

		/// Increase stake towards the specified subnet ID
		#[pallet::call_index(7)]
		// #[pallet::weight(T::WeightInfo::add_to_stake())]
		#[pallet::weight({0})]
		pub fn add_to_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			stake_to_be_added: u128,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin.clone())?;

			// --- Ensure subnet exists
			ensure!(
				SubnetsData::<T>::contains_key(subnet_id),
				Error::<T>::SubnetNotExist
			);

			// // Update accounts subnet stake add block
			// let mut model_account_delegate_stakes: BTreeMap<T::AccountId, u64> = SubnetAccountDelegateStake::<T>::get(subnet_id);
			// let block: u64 = Self::get_current_block_as_u64();

			// // Insert or update the accounts subnet stake add block
			// model_account_delegate_stakes.insert(account_id.clone(), block);
			// SubnetAccountDelegateStake::<T>::insert(subnet_id, model_account_delegate_stakes);

			Self::do_add_delegate_stake(
				origin, 
				subnet_id,
				stake_to_be_added,
			)
		}
		
		// TODO: Add test cases
		#[pallet::call_index(8)]
		#[pallet::weight({0})]
		pub fn transfer_delegate_stake(
			origin: OriginFor<T>, 
			from_subnet_id: u32, 
			to_subnet_id: u32, 
			delegate_stake_shares_to_be_switched: u128
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin.clone())?;

			// --- Ensure ``to`` subnet exists
			ensure!(
				SubnetsData::<T>::contains_key(to_subnet_id),
				Error::<T>::SubnetNotExist
			);

			Self::do_switch_delegate_stake(
				origin, 
				from_subnet_id,
				to_subnet_id,
				delegate_stake_shares_to_be_switched,
			)
		}

		#[pallet::call_index(9)]
		// #[pallet::weight(T::WeightInfo::remove_stake())]
		#[pallet::weight({0})]
		pub fn remove_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			shares_to_be_removed: u128
		) -> DispatchResult {
			Self::do_remove_delegate_stake(
				origin, 
				subnet_id,
				shares_to_be_removed,
			)
		}

		#[pallet::call_index(10)]
		// #[pallet::weight(T::WeightInfo::remove_stake())]
		#[pallet::weight({0})]
		pub fn claim_delegate_stake_unbondings(
			origin: OriginFor<T>, 
			subnet_id: u32, 
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin.clone())?;
			let successful_unbondings: u32 = Self::do_claim_unbondings(&account_id, subnet_id);
			ensure!(
				successful_unbondings > 0,
        Error::<T>::NoDelegateStakeUnbondingsOrCooldownNotMet
			);
			Ok(())
		}

		/// Remove stake balance
		/// If account is a current subnet peer on the subnet ID they can only remove up to minimum required balance
		// Decrease stake on accounts peer if minimum required isn't surpassed
		// to-do: if removed through consensus, add removed_block to storage and require time 
		//				to pass until they can remove their stake
		// #[pallet::call_index(10)]
		// // #[pallet::weight(T::WeightInfo::remove_stake())]
		// #[pallet::weight({0})]
		// pub fn remove_delegate_stake(
		// 	origin: OriginFor<T>, 
		// 	subnet_id: u32, 
		// 	shares_to_be_removed: u128
		// ) -> DispatchResult {
		// 	let account_id: T::AccountId = ensure_signed(origin.clone())?;

		// 	let cooldown_epoch: u64 = DelegateStakeCooldown::<T>::get(account_id.clone(), subnet_id);
		// 	let block: u64 = Self::get_current_block_as_u64();
		// 	let epoch_length: u64 = T::EpochLength::get();
		// 	let epoch: u64 = block / epoch_length;

		// 	ensure!(
		// 		epoch > cooldown_epoch + T::CooldownEpochs::get(),
    //     Error::<T>::InsufficientCooldown
		// 	);

		// 	ensure!(
		// 		epoch - (cooldown_epoch + T::CooldownEpochs::get()) <= T::DelegateStakeEpochsRemovalWindow::get(),
    //     Error::<T>::UnstakeWindowFinished
		// 	);

		// 	if AccountSubnetDelegateStakeShares::<T>::get(&account_id, subnet_id.clone()) == 0 {
		// 		DelegateStakeCooldown::<T>::insert(account_id.clone(), subnet_id, 0);
		// 	}

		// 	// let min_required_delegate_unstake_epochs = MinRequiredDelegateUnstakeEpochs::<T>::get();
		// 	// let epoch_length: u64 = T::EpochLength::get();
		// 	// let block: u64 = Self::get_current_block_as_u64();

		// 	// let mut model_account_delegate_stakes: BTreeMap<T::AccountId, u64> = SubnetAccountDelegateStake::<T>::get(subnet_id);

		// 	// let block_added: u64 = match model_account_delegate_stakes.get(&account_id.clone()) {
		// 	// 	Some(block_added) => *block_added,
		// 	// 	None => 0,
		// 	// };

		// 	// // We don't ensure! if the account add block is zero 
		// 	// // If they have no stake, it will be ensure!'d in the delegate_staking.rs

		// 	// // Ensure min required epochs have surpassed to unstake
		// 	// // Based on either initialized block or removal block
		// 	// ensure!(
		// 	// 	block >= Self::get_eligible_epoch_block(
		// 	// 		epoch_length, 
		// 	// 		block_added, 
		// 	// 		min_required_delegate_unstake_epochs
		// 	// 	),
		// 	// 	Error::<T>::RequiredDelegateUnstakeEpochsNotMet
		// 	// );

		// 	// Remove stake
		// 	Self::do_remove_delegate_stake(
		// 		origin, 
		// 		subnet_id,
		// 		account_id,
		// 		shares_to_be_removed,
		// 	)
		// }
		
		/// Delete proposals that are no longer live
		#[pallet::call_index(11)]
		#[pallet::weight({0})]
		pub fn validate(
			origin: OriginFor<T>, 
			subnet_id: u32,
			data: Vec<SubnetNodeData>,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

			let block: u64 = Self::get_current_block_as_u64();
			let epoch_length: u64 = T::EpochLength::get();
			let epoch: u64 = block / epoch_length;

			Self::do_validate(
				subnet_id, 
				account_id,
				block,
				epoch_length,
				epoch as u32,
				data,
			)
		}

		#[pallet::call_index(12)]
		#[pallet::weight({0})]
		pub fn attest(
			origin: OriginFor<T>, 
			subnet_id: u32,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

			let block: u64 = Self::get_current_block_as_u64();
			let epoch_length: u64 = T::EpochLength::get();
			let epoch: u64 = block / epoch_length;

			Self::do_attest(
				subnet_id, 
				account_id,
				block, 
				epoch_length,
				epoch as u32,
			)
		}

		#[pallet::call_index(13)]
		#[pallet::weight({0})]
		pub fn propose(
			origin: OriginFor<T>, 
			subnet_id: u32,
			peer_id: PeerId,
			data: Vec<u8>,
	) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_propose(
				account_id,
				subnet_id,
				peer_id,
				data
			)
		}

		#[pallet::call_index(14)]
		#[pallet::weight({0})]
		pub fn challenge_proposal(
			origin: OriginFor<T>, 
			subnet_id: u32,
			proposal_id: u32,
			data: Vec<u8>,
	) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_challenge_proposal(
				account_id,
				subnet_id,
				proposal_id,
				data
			)
		}

		#[pallet::call_index(15)]
		#[pallet::weight({0})]
		pub fn vote(
			origin: OriginFor<T>, 
			subnet_id: u32,
			proposal_id: u32,
			vote: VoteType
	) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_vote(
				account_id,
				subnet_id,
				proposal_id,
				vote
			)
		}

		/// Allows anyone to increase the subnets delegate stake pool
		#[pallet::call_index(16)]
		#[pallet::weight({0})]
		pub fn increase_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			amount: u128,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::increase_delegated_stake(
				subnet_id,
				amount,
			);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Activate subnet - called by subnet democracy logic
		pub fn activate_subnet(
			activator: T::AccountId,
			proposer: T::AccountId,
			subnet_data: PreSubnetData,
		) -> DispatchResult {
			// let activator: T::AccountId = ensure_signed(activator)?;

			// Ensure path is unique
			ensure!(
				!SubnetPaths::<T>::contains_key(subnet_data.clone().path),
				Error::<T>::SubnetExist
			);

			// Ensure max subnets not reached
			// Get total live subnets
			let total_subnets: u32 = (SubnetsData::<T>::iter().count()).try_into().unwrap();
			let max_subnets: u32 = MaxSubnets::<T>::get();
			ensure!(
				total_subnets < max_subnets,
				Error::<T>::MaxSubnets
			);

			let block: u64 = Self::get_current_block_as_u64();
			let model_cost: u128 = Self::get_model_initialization_cost(block);

			if model_cost > 0 {
				// unreserve from proposer
				let model_cost_as_balance = Self::u128_to_balance(model_cost);

				ensure!(
					Self::can_remove_balance_from_coldkey_account(&proposer, model_cost_as_balance.unwrap()),
					Error::<T>::NotEnoughBalanceToStake
				);
		
				ensure!(
					Self::remove_balance_from_coldkey_account(&proposer, model_cost_as_balance.unwrap()) == true,
					Error::<T>::BalanceWithdrawalError
				);

				// Send portion to stake rewards vault
				// Send portion to treasury

				// increase stake balance with subnet initialization cost
				StakeVaultBalance::<T>::mutate(|n: &mut u128| *n += model_cost);
			}

			// Get total subnets ever
			let subnet_len: u32 = TotalSubnets::<T>::get();
			// Start the model_ids at 1
			let subnet_id = subnet_len + 1;
			
			let base_node_memory: u128 = BaseSubnetNodeMemoryMB::<T>::get();
	
			let min_subnet_nodes: u32 = Self::get_min_subnet_nodes(base_node_memory, subnet_data.memory_mb);
			let target_subnet_nodes: u32 = Self::get_target_subnet_nodes(base_node_memory, min_subnet_nodes);
	
			let subnet_data = SubnetData {
				id: subnet_id,
				path: subnet_data.clone().path,
				min_nodes: min_subnet_nodes,
				target_nodes: target_subnet_nodes,
				memory_mb: subnet_data.memory_mb,  
				initialized: block,
			};

			// Store unique path
			SubnetPaths::<T>::insert(subnet_data.clone().path, subnet_id);
			// Store subnet data
			SubnetsData::<T>::insert(subnet_id, subnet_data.clone());
			// Increase total subnets. This is used for unique Subnet IDs
			TotalSubnets::<T>::mutate(|n: &mut u32| *n += 1);

			Self::deposit_event(Event::SubnetAdded { 
				proposer: proposer, 
				activator: activator,
				subnet_id: subnet_id, 
				subnet_path: subnet_data.path,
				block: block
			});

			Ok(())
		}

		pub fn deactivate_subnet(
			deactivator: T::AccountId,
			proposer: T::AccountId,
			subnet_data: PreSubnetData,
		) -> DispatchResult {
			ensure!(
				SubnetPaths::<T>::contains_key(subnet_data.clone().path),
				Error::<T>::SubnetNotExist
			);

			let subnet_id = SubnetPaths::<T>::get(subnet_data.clone().path).unwrap();

			ensure!(
				SubnetsData::<T>::contains_key(subnet_id),
				Error::<T>::SubnetNotExist
			);

			let subnet = SubnetsData::<T>::get(subnet_id).unwrap();
			let subnet_path: Vec<u8> = subnet.path;

			// ----
			// Subnets can be removed by
			// 		1. Subnet can be voted off
			//		2. Subnet can reach max zero consensus count
			//		3. Subnet can be offline too many times
			//		4. Subnet has min peers after initialization period
			// ----

			let mut reason_for_removal: Vec<u8> = Vec::new();

			// Remove unique path
			SubnetPaths::<T>::remove(subnet_path.clone());
			// Remove subnet data
			SubnetsData::<T>::remove(subnet_id);

			// We don't subtract TotalSubnets since it's used for ids

			// Remove all peers data
			let _ = SubnetNodesData::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let _ = TotalSubnetNodes::<T>::remove(subnet_id);
			let _ = SubnetNodeAccount::<T>::clear_prefix(subnet_id, u32::MAX, None);

			// Remove all subnet consensus data
			let _ = SubnetPenaltyCount::<T>::remove(subnet_id);
			let _ = SubnetNodesClasses::<T>::clear_prefix(subnet_id, u32::MAX, None);
	
			Self::deposit_event(Event::SubnetRemoved { 
				account: deactivator, 
				subnet_id: subnet_id, 
				subnet_path: subnet_path,
				reason: reason_for_removal,
				block: 0
			});

			Ok(())
		}

		// fn do_add_subnet_node(
		// 	origin: OriginFor<T>, 
		// 	subnet_id: u32, 
		// 	peer_id: PeerId, 
		// 	stake_to_be_added: u128,
		// ) -> DispatchResult {
		// 	let account_id: T::AccountId = ensure_signed(origin.clone())?;

		// 	let epoch_length: u64 = T::EpochLength::get();
		// 	let block: u64 = Self::get_current_block_as_u64();

		// 	ensure!(
		// 		SubnetsData::<T>::contains_key(subnet_id),
		// 		Error::<T>::SubnetNotExist
		// 	);

		// 	// Ensure account is eligible
		// 	ensure!(
		// 		Self::is_account_eligible(account_id.clone()),
		// 		Error::<T>::AccountIneligible
		// 	);
			
		// 	// Ensure max peers isn't surpassed
		// 	let total_subnet_nodes: u32 = TotalSubnetNodes::<T>::get(subnet_id);
		// 	let max_subnet_nodes: u32 = MaxSubnetNodes::<T>::get();
		// 	ensure!(
		// 		total_subnet_nodes < max_subnet_nodes,
		// 		Error::<T>::SubnetNodesMax
		// 	);

		// 	// Unique subnet_id -> AccountId
		// 	// Ensure account doesn't already have a peer within subnet
		// 	ensure!(
		// 		!SubnetNodesData::<T>::contains_key(subnet_id, account_id.clone()),
		// 		Error::<T>::SubnetNodeExist
		// 	);

		// 	// Unique subnet_id -> PeerId
		// 	// Ensure peer ID doesn't already exist within subnet regardless of account_id
		// 	let peer_exists: bool = match SubnetNodeAccount::<T>::try_get(subnet_id, peer_id.clone()) {
		// 		Ok(_) => true,
		// 		Err(()) => false,
		// 	};

		// 	ensure!(
		// 		!peer_exists,
		// 		Error::<T>::PeerIdExist
		// 	);

		// 	// Validate peer_id
		// 	ensure!(
		// 		Self::validate_peer_id(peer_id.clone()),
		// 		Error::<T>::InvalidPeerId
		// 	);

		// 	// ====================
		// 	// Initiate stake logic
		// 	// ====================
		// 	Self::do_add_stake(
		// 		origin.clone(), 
		// 		subnet_id,
		// 		account_id.clone(),
		// 		stake_to_be_added,
		// 	).map_err(|e| e)?;

		// 	// To ensure the AccountId that owns the PeerId, they must sign the PeerId for others to verify
		// 	// This ensures others cannot claim to own a PeerId they are not the owner of
		// 	// Self::validate_signature(&Encode::encode(&peer_id), &signature, &signer)?;

		// 	// ========================
		// 	// Insert peer into storage
		// 	// ========================
		// 	let subnet_node: SubnetNode<T::AccountId> = SubnetNode {
		// 		account_id: account_id.clone(),
		// 		peer_id: peer_id.clone(),
		// 		initialized: block,
		// 	};
		// 	// Insert SubnetNodesData with account_id as key
		// 	SubnetNodesData::<T>::insert(subnet_id, account_id.clone(), subnet_node);

		// 	// Insert subnet peer account to keep peer_ids unique within subnets
		// 	SubnetNodeAccount::<T>::insert(subnet_id, peer_id.clone(), account_id.clone());

		// 	// Insert unstaking reinforcements
		// 	// This data is specifically used for allowing unstaking after being removed
		// 	// SubnetAccount is not removed from storage until the peer has unstaked their entire stake balance
		// 	// This stores the block they are initialized at
		// 	// If removed, the initialized block will be replace with the removal block
		// 	let mut model_accounts: BTreeMap<T::AccountId, u64> = SubnetAccount::<T>::get(subnet_id);
		// 	// let model_account: Option<&u64> = model_accounts.get(&account_id.clone());
		// 	let block_initialized_or_removed: u64 = match model_accounts.get(&account_id.clone()) {
		// 		Some(block_initialized_or_removed) => *block_initialized_or_removed,
		// 		None => 0,
		// 	};

		// 	// If previously removed or removed themselves
		// 	// Ensure they have either unstaked or have waited enough epochs to unstake
		// 	// to readd themselves as a subnet peer
		// 	if block_initialized_or_removed != 0 {
		// 		let min_required_unstake_epochs = MinRequiredUnstakeEpochs::<T>::get();
		// 		// Ensure min required epochs have surpassed to unstake
		// 		// Based on either initialized block or removal block
		// 		ensure!(
		// 			block >= Self::get_eligible_epoch_block(
		// 				epoch_length, 
		// 				block_initialized_or_removed, 
		// 				min_required_unstake_epochs
		// 			),
		// 			Error::<T>::RequiredUnstakeEpochsNotMet
		// 		);	
		// 	}

		// 	// Update to current block
		// 	model_accounts.insert(account_id.clone(), block);
		// 	SubnetAccount::<T>::insert(subnet_id, model_accounts);

		// 	if let Ok(mut node_class) = SubnetNodesClasses::<T>::try_get(subnet_id, SubnetNodeClass::Idle) {
		// 		node_class.insert(account_id.clone(), block);
		// 		SubnetNodesClasses::<T>::insert(subnet_id, SubnetNodeClass::Idle, node_class);	
		// 	} else {
		// 		// If new subnet, initialize classes
		// 		let mut node_class: BTreeMap<T::AccountId, u64> = BTreeMap::new();
		// 		node_class.insert(account_id.clone(), block);
		// 		SubnetNodesClasses::<T>::insert(subnet_id, SubnetNodeClass::Idle, node_class);	
		// 	}

		// 	// Add subnet_id to account
		// 	// Account can only have a subnet peer per subnet so we don't check if it exists
		// 	AccountSubnets::<T>::append(account_id.clone(), subnet_id);

		// 	// Increase total subnet peers
		// 	TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

		// 	Self::deposit_event(
		// 		Event::SubnetNodeAdded { 
		// 			subnet_id: subnet_id, 
		// 			account_id: account_id.clone(), 
		// 			peer_id: peer_id.clone(),
		// 			block: block
		// 		}
		// 	);

		// 	Ok(())
		// }
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
			let block: u64 = Self::convert_block_as_u64(block_number);
			let epoch_length: u64 = T::EpochLength::get();

			// Reward validators and attestors... Shift node classes
			if block >= epoch_length && block % epoch_length == 0 {
				log::info!("Rewarding epoch");
				let epoch: u64 = block / epoch_length;

				// Reward subnets for the previous epoch
				// Reward before shifting
				Self::reward_subnets(block, (epoch - 1) as u32, epoch_length);

				// --- Update subnet nodes classifications
				Self::shift_node_classes(block, epoch_length);

				return Weight::from_parts(207_283_478_000, 22166406)
					.saturating_add(T::DbWeight::get().reads(18250_u64))
					.saturating_add(T::DbWeight::get().writes(12002_u64));
			}

			// Run the block succeeding form consensus
			if (block - 1) >= epoch_length && (block - 1) % epoch_length == 0 {
				log::info!("Generating random validator");
				let epoch: u64 = block / epoch_length;

				// Choose validators and accountants for the current epoch
				Self::do_choose_validator_and_accountants(block, epoch as u32, epoch_length);

				return Weight::from_parts(153_488_564_000, 21699450)
					.saturating_add(T::DbWeight::get().reads(6118_u64))
					.saturating_add(T::DbWeight::get().writes(6082_u64));
			}

			return Weight::from_parts(8_054_000, 1638)
				.saturating_add(T::DbWeight::get().reads(1_u64))
		}
	}

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub subnet_path: Vec<u8>,
		pub memory_mb: u128,
		pub subnet_nodes: Vec<(T::AccountId, PeerId)>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			SubnetNodeClassEpochs::<T>::insert(SubnetNodeClass::Idle, 2);
			SubnetNodeClassEpochs::<T>::insert(SubnetNodeClass::Included, 4);
			SubnetNodeClassEpochs::<T>::insert(SubnetNodeClass::Submittable, 6);
			SubnetNodeClassEpochs::<T>::insert(SubnetNodeClass::Accountant, 8);

			let subnet_id = 1;

			let base_node_memory: u128 = BaseSubnetNodeMemoryMB::<T>::get();

			// --- Get min nodes based on default memory settings
			let real_min_subnet_nodes: u128 = self.memory_mb.clone() / base_node_memory;
			let mut min_subnet_nodes: u32 = MinSubnetNodes::<T>::get();
			if real_min_subnet_nodes as u32 > min_subnet_nodes {
				min_subnet_nodes = real_min_subnet_nodes as u32;
			}
				
			let target_subnet_nodes: u32 = (min_subnet_nodes as u128).saturating_mul(TargetSubnetNodesMultiplier::<T>::get()).saturating_div(10000) as u32 + min_subnet_nodes;

			let subnet_data = SubnetData {
				id: subnet_id,
				path: self.subnet_path.clone(),
				min_nodes: min_subnet_nodes,
				target_nodes: target_subnet_nodes,
				memory_mb: self.memory_mb.clone(),
				initialized: 0,
			};

			// Activate subnet
			let pre_subnet_data = PreSubnetData {
				path: self.subnet_path.clone(),
				memory_mb: self.memory_mb.clone(),
			};
		
			let vote_subnet_data = VoteSubnetData {
				data: pre_subnet_data,
				active: true,
			};

			SubnetActivated::<T>::insert(self.subnet_path.clone(), vote_subnet_data);
			// Store unique path
			SubnetPaths::<T>::insert(self.subnet_path.clone(), subnet_id);
			// Store subnet data
			SubnetsData::<T>::insert(subnet_id, subnet_data.clone());
			// Increase total subnets count
			TotalSubnets::<T>::mutate(|n: &mut u32| *n += 1);

			// StakeVaultBalance::<T>::mutate(|n: &mut u128| *n += 10000000000000000000);

			// let mut stake_amount: u128 = MinStakeBalance::<T>::get();
			
			// let mut count = 0;
			// for (account_id, peer_id) in &self.subnet_nodes {
			// 	// Unique subnet_id -> PeerId
			// 	// Ensure peer ID doesn't already exist within subnet regardless of account_id
			// 	let peer_exists: bool = match SubnetNodeAccount::<T>::try_get(subnet_id, peer_id.clone()) {
			// 		Ok(_) => true,
			// 		Err(()) => false,
			// 	};

			// 	if peer_exists {
			// 		continue;
			// 	}

			// 	// ====================
			// 	// Initiate stake logic
			// 	// ====================
			// 	// T::Currency::withdraw(
			// 	// 	&account_id,
			// 	// 	stake_amount,
			// 	// 	WithdrawReasons::except(WithdrawReasons::TIP),
			// 	// 	ExistenceRequirement::KeepAlive,
			// 	// );

			// 	// -- increase account subnet staking balance
			// 	AccountSubnetStake::<T>::insert(
			// 		account_id,
			// 		subnet_id,
			// 		AccountSubnetStake::<T>::get(account_id, subnet_id).saturating_add(stake_amount),
			// 	);

			// 	// -- increase account_id total stake
			// 	TotalAccountStake::<T>::mutate(account_id, |mut n| *n += stake_amount);

			// 	// -- increase total subnet stake
			// 	TotalSubnetStake::<T>::mutate(subnet_id, |mut n| *n += stake_amount);

			// 	// -- increase total stake overall
			// 	TotalStake::<T>::mutate(|mut n| *n += stake_amount);

			// 	// To ensure the AccountId that owns the PeerId, they must sign the PeerId for others to verify
			// 	// This ensures others cannot claim to own a PeerId they are not the owner of
			// 	// Self::validate_signature(&Encode::encode(&peer_id), &signature, &signer)?;

			// 	// ========================
			// 	// Insert peer into storage
			// 	// ========================
			// 	let subnet_node: SubnetNode<T::AccountId> = SubnetNode {
			// 		account_id: account_id.clone(),
			// 		peer_id: peer_id.clone(),
			// 		initialized: 0,
			// 	};
			// 	// Insert SubnetNodesData with account_id as key
			// 	SubnetNodesData::<T>::insert(subnet_id, account_id.clone(), subnet_node);

			// 	// Insert subnet peer account to keep peer_ids unique within subnets
			// 	SubnetNodeAccount::<T>::insert(subnet_id, peer_id.clone(), account_id.clone());

			// 	// Insert unstaking reinforcements
			// 	// This data is specifically used for allowing unstaking after being removed
			// 	// SubnetAccount is not removed from storage until the peer has unstaked their entire stake balance
			// 	// This stores the block they are initialized at
			// 	// If removed, the initialized block will be replace with the removal block
			// 	let mut model_accounts: BTreeMap<T::AccountId, u64> = SubnetAccount::<T>::get(subnet_id);
			// 	// let model_account: Option<&u64> = model_accounts.get(&account_id.clone());
			// 	let block_initialized_or_removed: u64 = match model_accounts.get(&account_id.clone()) {
			// 		Some(block_initialized_or_removed) => *block_initialized_or_removed,
			// 		None => 0,
			// 	};

			// 	// Update to current block
			// 	model_accounts.insert(account_id.clone(), 0);
			// 	SubnetAccount::<T>::insert(subnet_id, model_accounts);

			// 	if let Ok(mut node_class) = SubnetNodesClasses::<T>::try_get(subnet_id, SubnetNodeClass::Idle) {
			// 		node_class.insert(account_id.clone(), 0);
			// 		SubnetNodesClasses::<T>::insert(subnet_id, SubnetNodeClass::Idle, node_class);	
			// 	} else {
			// 		// If new subnet, initialize classes
			// 		let mut node_class: BTreeMap<T::AccountId, u64> = BTreeMap::new();
			// 		node_class.insert(account_id.clone(), 0);
			// 		SubnetNodesClasses::<T>::insert(subnet_id, SubnetNodeClass::Idle, node_class);	
			// 	}

			// 	// Add subnet_id to account
			// 	// Account can only have a subnet peer per subnet so we don't check if it exists
			// 	AccountSubnets::<T>::append(account_id.clone(), subnet_id);

			// 	// Increase total subnet peers
			// 	TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);


			// 	count += 1;
			// }
		}
	}
}

// Staking logic from rewards pallet
impl<T: Config> IncreaseStakeVault for Pallet<T> {
	fn increase_stake_vault(amount: u128) -> DispatchResult {
		StakeVaultBalance::<T>::mutate(|n: &mut u128| *n += amount);
		Ok(())
	}
}
pub trait IncreaseStakeVault {
	fn increase_stake_vault(amount: u128) -> DispatchResult;
}


impl<T: Config<AccountId = AccountId>, AccountId> SubnetVote<AccountId> for Pallet<T> {
	fn vote_model_in(vote_subnet_data: VoteSubnetData) -> DispatchResult {
		SubnetActivated::<T>::insert(vote_subnet_data.clone().data.path, vote_subnet_data.clone());
		Ok(())
	}
	fn vote_model_out(vote_subnet_data: VoteSubnetData) -> DispatchResult {
		SubnetActivated::<T>::insert(vote_subnet_data.clone().data.path, vote_subnet_data.clone());
		Ok(())
	}
	fn vote_activated(activator: AccountId, path: Vec<u8>, proposer: AccountId, vote_subnet_data: VoteSubnetData) -> DispatchResult {
		SubnetActivated::<T>::insert(path, vote_subnet_data.clone());

		Self::activate_subnet(
			activator, 
			proposer,
			vote_subnet_data.clone().data,
		)
	}
	fn vote_deactivated(deactivator: AccountId, path: Vec<u8>, proposer: AccountId, vote_subnet_data: VoteSubnetData) -> DispatchResult {
		SubnetActivated::<T>::insert(path, vote_subnet_data.clone());

		Self::deactivate_subnet(
			deactivator, 
			proposer,
			vote_subnet_data.clone().data,
		)
	}

	fn get_total_models() -> u32 {
		TotalSubnets::<T>::get()
	}
	fn get_model_initialization_cost() -> u128 {
		let block: u64 = Self::get_current_block_as_u64();
		Self::get_model_initialization_cost(block)
	}
	fn get_model_path_exist(path: Vec<u8>) -> bool {
		if SubnetPaths::<T>::contains_key(path) {
			true
		} else {
			false
		}
	}
	fn get_model_id_by_path(path: Vec<u8>) -> u32 {
		if !SubnetPaths::<T>::contains_key(path.clone()) {
			return 0
		} else {
			return SubnetPaths::<T>::get(path.clone()).unwrap()
		}
	}
	fn get_model_id_exist(id: u32) -> bool {
		if SubnetsData::<T>::contains_key(id) {
			true
		} else {
			false
		}
	}
	// Should never be called unless contains_key is confirmed
	fn get_model_data(id: u32) -> SubnetData {
		SubnetsData::<T>::get(id).unwrap()
	}
	// fn get_min_subnet_nodes() -> u32 {
	// 	MinSubnetNodes::<T>::get()
	// }
	fn get_max_subnet_nodes() -> u32 {
		MaxSubnetNodes::<T>::get()
	}
	fn get_min_stake_balance() -> u128 {
		MinStakeBalance::<T>::get()
	}
	fn is_submittable_subnet_node_account(account_id: AccountId) -> bool {
		true
	}
	fn is_model_initialized(id: u32) -> bool {
		let subnet_data = SubnetsData::<T>::get(id).unwrap();
		let model_initialized = subnet_data.initialized;

		let epoch_length: u64 = T::EpochLength::get();
		let min_required_model_consensus_submit_epochs = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();
		let block: u64 = Self::get_current_block_as_u64();

		block >= Self::get_eligible_epoch_block(
			epoch_length, 
			model_initialized, 
			min_required_model_consensus_submit_epochs
		)
	}
	fn get_total_model_errors(id: u32) -> u32 {
		SubnetPenaltyCount::<T>::get(id)
	}
	fn get_min_subnet_nodes(memory_mb: u128) -> u32 {
		let base_node_memory: u128 = BaseSubnetNodeMemoryMB::<T>::get();
		Self::get_min_subnet_nodes(base_node_memory, memory_mb)
	}
	fn get_target_subnet_nodes(min_subnet_nodes: u32) -> u32 {
		let base_node_memory: u128 = BaseSubnetNodeMemoryMB::<T>::get();
		Self::get_target_subnet_nodes(base_node_memory, min_subnet_nodes)
	}
	fn get_stake_balance(account_id: AccountId) -> u128 {
		Self::get_account_total_stake_balance(account_id)
	}
	fn get_delegate_stake_balance(account_id: AccountId) -> u128 {
		let mut total_delegate_stake_balance = 0;
		for (subnet_id, _) in SubnetsData::<T>::iter() {
			total_delegate_stake_balance += Self::convert_account_shares_to_balance(
				&account_id,
				subnet_id
			);
		}
		total_delegate_stake_balance
	}
}

pub trait SubnetVote<AccountId> {
	fn vote_model_in(vote_subnet_data: VoteSubnetData) -> DispatchResult;
	fn vote_model_out(vote_subnet_data: VoteSubnetData) -> DispatchResult;
	fn vote_activated(activator: AccountId, path: Vec<u8>, proposer: AccountId, vote_subnet_data: VoteSubnetData) -> DispatchResult;
	fn vote_deactivated(deactivator: AccountId, path: Vec<u8>, proposer: AccountId, vote_subnet_data: VoteSubnetData) -> DispatchResult;
	fn get_total_models() -> u32;
	fn get_model_initialization_cost() -> u128;
	fn get_model_path_exist(path: Vec<u8>) -> bool;
	fn get_model_id_by_path(path: Vec<u8>) -> u32;
	fn get_model_id_exist(id: u32) -> bool;
	fn get_model_data(id: u32) -> SubnetData;
	fn get_max_subnet_nodes() -> u32;
	fn get_min_stake_balance() -> u128;
	fn is_submittable_subnet_node_account(account_id: AccountId) -> bool;
	fn is_model_initialized(id: u32) -> bool;
	fn get_total_model_errors(id: u32) -> u32;
	fn get_min_subnet_nodes(memory_mb: u128) -> u32;
	fn get_target_subnet_nodes(min_subnet_nodes: u32) -> u32;
	fn get_stake_balance(account_id: AccountId) -> u128;
	fn get_delegate_stake_balance(account_id: AccountId) -> u128;
}

// Admin logic
impl<T: Config> AdminInterface for Pallet<T> {
	fn set_vote_model_in(path: Vec<u8>, memory_mb: u128) -> DispatchResult {
		Self::set_vote_model_in(path, memory_mb)
	}
	fn set_vote_model_out(path: Vec<u8>) -> DispatchResult {
		Self::set_vote_model_out(path)
	}
	fn set_max_models(value: u32) -> DispatchResult {
		Self::set_max_models(value)
	}
	fn set_min_subnet_nodes(value: u32) -> DispatchResult {
		Self::set_min_subnet_nodes(value)
	}
	fn set_max_subnet_nodes(value: u32) -> DispatchResult {
		Self::set_max_subnet_nodes(value)
	}
	fn set_min_stake_balance(value: u128) -> DispatchResult {
		Self::set_min_stake_balance(value)
	}
	fn set_tx_rate_limit(value: u64) -> DispatchResult {
		Self::set_tx_rate_limit(value)
	}
	fn set_max_consensus_epochs_errors(value: u32) -> DispatchResult {
		Self::set_max_consensus_epochs_errors(value)
	}
	fn set_min_required_model_consensus_submit_epochs(value: u64) -> DispatchResult {
		Self::set_min_required_model_consensus_submit_epochs(value)
	}
	fn set_min_required_peer_consensus_submit_epochs(value: u64) -> DispatchResult {
		Self::set_min_required_peer_consensus_submit_epochs(value)
	}
	fn set_min_required_peer_consensus_inclusion_epochs(value: u64) -> DispatchResult {
		Self::set_min_required_peer_consensus_inclusion_epochs(value)
	}
	fn set_min_required_peer_consensus_dishonesty_epochs(value: u64) -> DispatchResult {
		Self::set_min_required_peer_consensus_dishonesty_epochs(value)
	}
	fn set_max_outlier_delta_percent(value: u8) -> DispatchResult {
		Self::set_max_outlier_delta_percent(value)
	}
	fn set_subnet_node_consensus_submit_percent_requirement(value: u128) -> DispatchResult {
		Self::set_subnet_node_consensus_submit_percent_requirement(value)
	}
	fn set_consensus_blocks_interval(value: u64) -> DispatchResult {
		Self::set_consensus_blocks_interval(value)
	}
	fn set_peer_removal_threshold(value: u128) -> DispatchResult {
		Self::set_peer_removal_threshold(value)
	}
	fn set_max_model_rewards_weight(value: u128) -> DispatchResult {
		Self::set_max_model_rewards_weight(value)
	}
	fn set_stake_reward_weight(value: u128) -> DispatchResult {
		Self::set_stake_reward_weight(value)
	}
	fn set_model_per_peer_init_cost(value: u128) -> DispatchResult {
		Self::set_model_per_peer_init_cost(value)
	}
	fn set_model_consensus_unconfirmed_threshold(value: u128) -> DispatchResult {
		Self::set_model_consensus_unconfirmed_threshold(value)
	}
	fn set_remove_subnet_node_epoch_percentage(value: u128) -> DispatchResult {
		Self::set_remove_subnet_node_epoch_percentage(value)
	}
}

pub trait AdminInterface {
	fn set_vote_model_in(path: Vec<u8>, memory_mb: u128) -> DispatchResult;
	fn set_vote_model_out(path: Vec<u8>) -> DispatchResult;
	fn set_max_models(value: u32) -> DispatchResult;
	fn set_min_subnet_nodes(value: u32) -> DispatchResult;
	fn set_max_subnet_nodes(value: u32) -> DispatchResult;
	fn set_min_stake_balance(value: u128) -> DispatchResult;
	fn set_tx_rate_limit(value: u64) -> DispatchResult;
	fn set_max_consensus_epochs_errors(value: u32) -> DispatchResult;
	fn set_min_required_model_consensus_submit_epochs(value: u64) -> DispatchResult;
	fn set_min_required_peer_consensus_submit_epochs(value: u64) -> DispatchResult;
	fn set_min_required_peer_consensus_inclusion_epochs(value: u64) -> DispatchResult;
	fn set_min_required_peer_consensus_dishonesty_epochs(value: u64) -> DispatchResult;	
	fn set_max_outlier_delta_percent(value: u8) -> DispatchResult;
	fn set_subnet_node_consensus_submit_percent_requirement(value: u128) -> DispatchResult;
	fn set_consensus_blocks_interval(value: u64) -> DispatchResult;
	fn set_peer_removal_threshold(value: u128) -> DispatchResult;
	fn set_max_model_rewards_weight(value: u128) -> DispatchResult;
	fn set_stake_reward_weight(value: u128) -> DispatchResult;
	fn set_model_per_peer_init_cost(value: u128) -> DispatchResult;
	fn set_model_consensus_unconfirmed_threshold(value: u128) -> DispatchResult;
	fn set_remove_subnet_node_epoch_percentage(value: u128) -> DispatchResult;
}