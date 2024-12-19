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
	fail,
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
use frame_system::pallet_prelude::OriginFor;
use sp_std::ops::BitAnd;
use sp_runtime::Saturating;

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
			
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type SubnetInitializationCost: Get<u128>;

		#[pallet::constant]
		type DelegateStakeCooldownEpochs: Get<u64>;

		#[pallet::constant]
		type StakeCooldownEpochs: Get<u64>;

		#[pallet::constant]
		type DelegateStakeEpochsRemovalWindow: Get<u64>;

		#[pallet::constant]
		type MaxDelegateStakeUnlockings: Get<u32>;
		
		#[pallet::constant]
		type MaxStakeUnlockings: Get<u32>;

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
		SubnetAdded { activator: T::AccountId, subnet_id: u32, subnet_path: Vec<u8>, block: u64 },
		SubnetRemoved { subnet_id: u32, subnet_path: Vec<u8>, reason: SubnetRemovalReason },

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

		// Proposals
		Proposal { subnet_id: u32, proposal_id: u32, epoch: u32, plaintiff: T::AccountId, defendant: T::AccountId, plaintiff_data: Vec<u8> },
		ProposalChallenged { subnet_id: u32, proposal_id: u32, defendant: T::AccountId, defendant_data: Vec<u8> },
		ProposalVote { subnet_id: u32, proposal_id: u32, account_id: T::AccountId, vote: VoteType },
		ProposalFinalized { subnet_id: u32, proposal_id: u32 },
		ProposalCanceled { subnet_id: u32, proposal_id: u32 },

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
		/// Subnet node already activated
		SubnetNodeAlreadyActivated,
		///
		SubnetNodeNotActivated,
		/// Node ID already in use
		PeerIdExist,
		/// Node ID already in use
		PeerIdNotExist,
		/// Subnet peer doesn't exist
		SubnetNodeNotExist,
		/// Subnet already exists
		SubnetExist,
		/// Max subnet memory size exceeded
		MaxSubnetMemory,
		/// Invalid registration block
		InvalidSubnetRegistrationBlocks,
		/// Subnet node must be unstaked to re-register to use the same balance
		InvalidSubnetRegistrationCooldown,
		/// Subnet minimum delegate stake balance is met
		SubnetMinDelegateStakeBalanceMet,
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
		InvalidEpoch,
		SubnetRewardsSubmissionComplete,
		InvalidSubnetId,
		SubnetAlreadyActivated,

		DelegateStakeTransferPeriodExceeded,

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
		InvalidMaxSubnetMemoryMB,
		// staking
		/// u128 -> BalanceOf conversion error
		CouldNotConvertToBalance,
		/// Not enough balance on Account to stake and keep alive
		NotEnoughBalanceToStake,
		NotEnoughBalance,
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
		NoStakeUnbondingsOrCooldownNotMet,
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

		PartiesCannotVote,

		ProposalNotExist,
		ProposalNotChallenged,
		ProposalChallenged,
		ProposalChallengePeriodPassed,
		PropsalAlreadyChallenged,
		NotChallenger,
		NotEligible,
		AlreadyVoted,
		VotingPeriodInvalid,
		ChallengePeriodPassed,
		DuplicateVote,
		NotAccountant,
		InvalidAccountantDataId,
		DataEmpty,

		InvalidAccountantData,
		InvalidSubnetRewardsSubmission,
		SubnetInitializing,
		SubnetActivatedAlready,
		InvalidSubnetRemoval,
		SubnetNodeNotSubmittable,


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
		ProposalComplete,
		/// Subnet node as defendant has proposal activated already
		NodeHasActiveProposal,
	}
	
	/// Subnet node classification
	/// u64 is epoch subnet node is inducted as each classification 
	// #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	// pub struct SubnetNodeClassification {
	// 	pub idle: u64,
	// 	pub inclusion: u64,
	// 	pub submission: u64,
	// 	pub accountant: u64,
	// }

	/// account_id: 		Coldkey of subnet node
	/// hotkey: 				Hotkey of subnet node for interacting with subnet on-chain communication
	/// peer_id: 				Peer ID of subnet node within subnet
	/// initialized:		Block initialized
	/// classification:	Subnet node classification for on-chain permissions
	/// a:							(Optional) Unique data for subnet to use and lookup via RPC, can only be added at registration
	/// b:							(Optional) Data for subnet to use and lookup via RPC
	/// c:							(Optional) Data for subnet to use and lookup via RPC
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetNode<AccountId> {
		pub account_id: AccountId,
		pub hotkey: AccountId,
		pub peer_id: PeerId,
		pub initialized: u64,
		pub classification: SubnetNodeClassification,
		pub a: Vec<u8>,
		pub b: Vec<u8>,
		pub c: Vec<u8>,
	}

	/// Registered: Subnet node registered, not included in consensus
	/// Idle: Subnet node is activated as idle, unless subnet is registering, and automatically updates on the first successful consensus epoch
	/// Included: Subnet node automatically updates to Included from Idle on the first successful consensus epoch after being Idle
	/// Submittable: Subnet node updates to Submittble from Included on the first successful consensus epoch they are included in consensus data
	/// Accountant:  Subnet node updates to Accountant after multiple successful validations
	#[derive(Default, EnumIter, FromRepr, Copy, Encode, Decode, Clone, PartialOrd, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub enum SubetNodeClass {
		#[default] Registered,
    Idle,
    Included,
		Submittable,
		Accountant
  }

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetNodeClassification {
		pub class: SubetNodeClass,
		pub start_epoch: u64,
	}

	impl<AccountId> SubnetNode<AccountId> {
    pub fn dude(&self) -> bool {
			self.initialized > 0
    }
		// pub fn has_classification(&self, required: &SubetNodeClass, epoch: u64) -> bool {
		// 	self.classification.class >= *required && self.classification.start_epoch <= epoch && self.initialized > 0
		// }

		pub fn has_classification(&self, required: &SubetNodeClass, epoch: u64) -> bool {
			self.classification.class >= *required && self.classification.start_epoch <= epoch
		}
	}


	// The submit consensus data format
	// Scoring is calculated off-chain between subnet peers hosting AI subnets together
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetNodeData {
		pub peer_id: PeerId,
		pub score: u128,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub enum SubnetRemovalReason {
    SubnetDemocracy,
    MaxPenalties,
		MinSubnetNodes,
		MinSubnetDelegateStake,
		Council,
		EnactmentPeriod,
  }

	/// Attests format for consensus
	/// ``u64`` is the block number of the accounts attestation for subnets to utilize to measure attestation speed
	/// The blockchain itself doesn't utilize this data
	pub type Attests<AccountId> = BTreeMap<AccountId, u64>;

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct RewardsData<AccountId> {
		pub validator: AccountId, // Chosen validator of the epoch
		pub sum: u128, // Sum of the data scores
		pub attests: Attests<AccountId>, // Count of attestations of the submitted data
		pub data: Vec<SubnetNodeData>, // Data submitted by chosen validator
		pub complete: bool, // Data submitted by chosen validator
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub struct MinNodesCurveParametersSet {
		pub x_curve_start: u128, // The range of ``max-min`` to start descending the curve
		pub y_end: u128, // The ``y`` end point on descending curve
		pub y_start: u128, // The ``y`` start point on descending curve
		pub x_rise: u128, // The rise from 0, usually should be 1/100
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub enum VoteType {
    Yay,
    Nay,
  }

	/// Subnet data used before activation
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct RegistrationSubnetData {
		pub path: Vec<u8>,
		pub memory_mb: u128,
		pub registration_blocks: u64,
	}
	
	/// Data for subnet held to be compared when adding a subnet to the network
	// This is the data from the democracy voting pallet
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetDemocracySubnetData {
		pub data: RegistrationSubnetData,
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
		pub registration_blocks: u64,
		pub activated: u64,
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
		pub attests: Attests<AccountId>,
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
		pub eligible_voters: BTreeSet<AccountId>, // Those eligible to vote at time of the proposal
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
	pub fn DefaultZeroU64() -> u64 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultAccountId<T: Config>() -> T::AccountId {
		T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap()
	}
	#[pallet::type_value]
	pub fn DefaultPeerId() -> PeerId {
		PeerId(Vec::new())
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
	pub fn DefaultSubnetDemocracySubnetData() -> SubnetDemocracySubnetData {
		let pre_subnet_data = RegistrationSubnetData {
			path: Vec::new(),
			memory_mb: 0,
			registration_blocks: 0,
		};
		return SubnetDemocracySubnetData {
			data: pre_subnet_data,
			active: false,
		}
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetPenaltyCount() -> u32 {
		16
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodeRegistrationEpochs() -> u64 {
		16
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodesClasses<T: Config>() -> BTreeMap<T::AccountId, u64> {
		BTreeMap::new()
	}
	#[pallet::type_value]
	pub fn DefaultMinRequiredSubnetConsensusSubmitEpochs() -> u64 {
		// This needs to be greater than the amount of epochs it takes to become a submittable subnet node
		16
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNode<T: Config>() -> SubnetNode<T::AccountId> {
		return SubnetNode {
			account_id: T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap(),
			hotkey: T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap(),
			peer_id: PeerId(Vec::new()),
			initialized: 0,
			classification: SubnetNodeClassification {
				class: SubetNodeClass::Registered,
				start_epoch: 0,
			},
			a: Vec::new(),
			b: Vec::new(),
      c: Vec::new(),
		};
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetNodes() -> u32 {
		// 94
		// 1024
		254
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
	pub fn DefaultMinSubnetDelegateStakePercentage() -> u128 {
		// 10000
		1000000000
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetDelegateStake() -> u128 {
		1000e+18 as u128
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
	pub fn DefaultDelegateStakeTransferPeriod() -> u64 {
		1000
	}
	#[pallet::type_value]
	pub fn DefaultDelegateStakeRewardsPercentage() -> u128 {
		// 1100
		110000000
	}
	// #[pallet::type_value]
	// pub fn DefaultMinRequiredDelegateUnstakeEpochs() -> u64 {
	// 	21
	// }
	#[pallet::type_value]
	pub fn DefaultDelegateStakeCooldown() -> u64 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultDelegateStakeUnbondingLedger() -> BTreeMap<u64, u128> {
		// We use epochs because cooldowns are based on epochs
		// {
		// 	epoch_start: u64, // cooldown begin epoch (+ cooldown duration for unlock)
		// 	balance: u128,
		// }
		BTreeMap::new()
	}
	#[pallet::type_value]
	pub fn DefaultSubnetStakeUnbondingLedger() -> BTreeMap<u64, u128> {
		// We use epochs because cooldowns are based on epochs
		// {
		// 	epoch_start: u64, // cooldown begin epoch (+ cooldown duration for unlock)
		// 	balance: u128,
		// }
		BTreeMap::new()
	}
	#[pallet::type_value]
	pub fn DefaultBaseRewardPerMB() -> u128 {
		1e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultBaseValidatorReward() -> u128 {
		1e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultBaseAccountantReward() -> u128 {
		1e+9 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMaxSequentialAbsentSubnetNode() -> u32 {
		// Must be less than the inclusion stage in the subnet node validation sequence
		// This ensures a subnet node is inducted via consensus before they can become a validator
		// 3 is for testing
		// production will more higher matching inclusion epochs
		3
	}
	#[pallet::type_value]
	pub fn DefaultSlashPercentage() -> u128 {
		// 312
		31250000
	}
	#[pallet::type_value]
	pub fn DefaultMaxSlashAmount() -> u128 {
		1e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMinAttestationPercentage() -> u128 {
		// 2/3
		660000000
	}
	#[pallet::type_value]
	pub fn DefaultMinVastMajorityAttestationPercentage() -> u128 {
		// 7/8
		875000000
	}
	#[pallet::type_value]
	pub fn DefaultTargetSubnetNodesMultiplier() -> u128 {
		// 3333
		333333333
	}
	#[pallet::type_value]
	pub fn DefaultBaseSubnetNodeMemoryMB() -> u128 {
		16_000
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetMemoryMB() -> u128 {
		1_000_000
	}
	#[pallet::type_value]
	pub fn DefaultTotalMaxSubnetMemoryMB() -> u128 {
		1_000_000
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetNodes() -> u32 {
		5
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetRegistrationBlocks() -> u64 {
		// 9 days at 6s blocks
		129_600
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetRegistrationBlocks() -> u64 {
		// 21 days at 6s blocks
		302_400
	}
	#[pallet::type_value]
	pub fn DefaultSubnetActivationEnactmentPeriod() -> u64 {
		// 3 days at 6s blocks
		43_200
	}
	#[pallet::type_value]
	pub fn DefaultMinNodesCurveParameters() -> MinNodesCurveParametersSet {
		// math.rs PERCENT_FACTOR format
		return MinNodesCurveParametersSet {
			x_curve_start: 15 * 1000000000 / 100, // 0.15
			y_end: 10 * 1000000000 / 100, // 0.10
			y_start: 75 * 1000000000 / 100, // 0.75
			x_rise: 1000000000 / 100, // 0.01
		}
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnets() -> u32 {
		64
	}
	#[pallet::type_value]
	pub fn DefaultTotalSubnetMemoryMB() -> u128 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultAccountantDataNodeParamsMaxLimit() -> u32 {
		1024_u32
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodeParamLimit() -> u32 {
		2024
	}

	/// Count of subnets
	#[pallet::storage]
	#[pallet::getter(fn total_subnets)]
	pub type TotalSubnets<T> = StorageValue<_, u32, ValueQuery>;
	
	#[pallet::storage]
	#[pallet::getter(fn max_subnets)]
	pub type MaxSubnets<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnets>;

	// Mapping of each subnet stored by ID, uniqued by `SubnetPaths`
	// Stores subnet data by a unique id
	#[pallet::storage] // subnet_id => data struct
	pub type SubnetsData<T: Config> = StorageMap<_, Blake2_128Concat, u32, SubnetData>;

	/// Total subnet memory across all subnets
	#[pallet::storage]
	pub type TotalSubnetMemoryMB<T: Config> = StorageValue<_, u128, ValueQuery, DefaultTotalSubnetMemoryMB>;

	// Ensures no duplicate subnet paths within the network at one time
	// If a subnet path is voted out, it can be voted up later on and any
	// stakes attached to the subnet_id won't impact the re-initialization
	// of the subnet path.
	#[pallet::storage]
	#[pallet::getter(fn subnet_paths)]
	pub type SubnetPaths<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, u32>;

	#[pallet::storage] // subnet ID => account_id
	pub type SubnetActivated<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		SubnetDemocracySubnetData,
		ValueQuery,
		DefaultSubnetDemocracySubnetData,
	>;

	/// Minimum blocks required from subnet registration to activation
	#[pallet::storage]
	pub type MinSubnetRegistrationBlocks<T> = StorageValue<_, u64, ValueQuery, DefaultMinSubnetRegistrationBlocks>;

	/// Maximum blocks required from subnet registration to activation
	#[pallet::storage]
	pub type MaxSubnetRegistrationBlocks<T> = StorageValue<_, u64, ValueQuery, DefaultMaxSubnetRegistrationBlocks>;

	/// Time period allowable for subnet activation following registration period
	#[pallet::storage]
	pub type SubnetActivationEnactmentPeriod<T> = StorageValue<_, u64, ValueQuery, DefaultSubnetActivationEnactmentPeriod>;

	// Minimum amount of peers required per subnet
	// required for subnet activity
	#[pallet::storage]
	#[pallet::getter(fn min_subnet_nodes)]
	pub type MinSubnetNodes<T> = StorageValue<_, u32, ValueQuery, DefaultMinSubnetNodes>;

	#[pallet::storage]
	pub type MinNodesCurveParameters<T> = StorageValue<_, MinNodesCurveParametersSet, ValueQuery, DefaultMinNodesCurveParameters>;

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
	
	#[pallet::storage] // subnet_uid --> u32
	#[pallet::getter(fn total_subnet_nodes)]
	pub type TotalSubnetNodes<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u32, ValueQuery>;

	#[pallet::storage] // subnet_uid --> u32
	#[pallet::getter(fn total_active_subnet_nodes)]
	pub type TotalActiveSubnetNodes<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u32, ValueQuery>;
	
	/// Total epochs a subnet node can stay in registration phase. If surpassed, they are removed on the first successful
	/// consensus epoch
	#[pallet::storage]
	pub type SubnetNodeRegistrationEpochs<T: Config> = StorageValue<_, u64, ValueQuery, DefaultSubnetNodeRegistrationEpochs>;
	
	// Epochs required from subnet initialization block to accept consensus submissions and choose validators and accountants
	// Epochs required based on EpochLength
	// Each epoch is EpochLength
	// Min required epochs for a subnet to be in storage for based on initialized
	#[pallet::storage]
	#[pallet::getter(fn min_required_subnet_consensus_submit_epochs)]
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
	
	#[pallet::storage] // subnet_id --> account_id --> data
	#[pallet::getter(fn subnet_nodes_v2)]
	pub type SubnetNodesDataV2<T: Config> = StorageDoubleMap<
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
	#[pallet::storage] // subnet_id --> peer_id --> account_id
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

	// Used for unique peer_ids
	#[pallet::storage] // subnet_id --> param --> peer_id
	pub type SubnetNodeParam<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		Vec<u8>,
		PeerId,
		ValueQuery,
		DefaultPeerId,
	>;
	
	#[derive(Default, EnumIter, FromRepr, Copy, Encode, Decode, Clone, PartialOrd, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub enum SubnetNodeClass {
    #[default] Idle,
    Included,
		Submittable,
		Accountant
  }

	impl SubnetNodeClass {
    pub fn index(&self) -> usize {
			*self as usize
    }
		pub fn has_classification(&self, required: &SubnetNodeClass) -> bool {
			self >= required
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

	/// Base subnet node memory used for calculating minimum and target nodes for a subnet
	#[pallet::storage]
	pub type BaseSubnetNodeMemoryMB<T> = StorageValue<_, u128, ValueQuery, DefaultBaseSubnetNodeMemoryMB>;

	/// Maximum subnet memory per subnet
	#[pallet::storage]
	pub type MaxSubnetMemoryMB<T> = StorageValue<_, u128, ValueQuery, DefaultMaxSubnetMemoryMB>;

	/// Total sum of subnet memory available in the network
	#[pallet::storage]
	pub type TotalMaxSubnetMemoryMB<T> = StorageValue<_, u128, ValueQuery, DefaultTotalMaxSubnetMemoryMB>;

	#[pallet::storage]
	pub type TargetSubnetNodesMultiplier<T> = StorageValue<_, u128, ValueQuery, DefaultTargetSubnetNodesMultiplier>;

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

	#[pallet::storage]
	pub type MinVastMajorityAttestationPercentage<T> = StorageValue<_, u128, ValueQuery, DefaultMinVastMajorityAttestationPercentage>;

	// Rewards (validator, scoring consensus)
	
	// Base reward per epoch for validators
	// This is the base reward to subnet validators on successful attestation
	#[pallet::storage]
	pub type BaseValidatorReward<T> = StorageValue<_, u128, ValueQuery, DefaultBaseValidatorReward>;

	// Base reward per MB per epoch based on 4,380 MB per year
	#[pallet::storage]
	pub type BaseRewardPerMB<T> = StorageValue<_, u128, ValueQuery, DefaultBaseRewardPerMB>;

	#[pallet::storage]
	pub type SlashPercentage<T> = StorageValue<_, u128, ValueQuery, DefaultSlashPercentage>;

	#[pallet::storage]
	pub type MaxSlashAmount<T> = StorageValue<_, u128, ValueQuery, DefaultMaxSlashAmount>;

	// Maximum epochs in a row a subnet node can be absent from validator submitted consensus data
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

	// Rewards (accountant, scoring consensus)

	// Base reward per epoch for accountants
	// This is the base reward to subnet accountants on successful attestation
	#[pallet::storage]
	pub type BaseAccountantReward<T> = StorageValue<_, u128, ValueQuery, DefaultBaseAccountantReward>;

	// Base reward per epoch for accountants
	// This is the base reward to subnet accountants on successful attestation
	#[pallet::storage]
	pub type BaseAccountingReward<T> = StorageValue<_, u128, ValueQuery, DefaultBaseAccountantReward>;


	// Maximum epochs in a row a subnet node can be absent from validator submitted consensus data
	#[pallet::storage]
	pub type MaxSubnetNodePenalties<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSequentialAbsentSubnetNode>;
	
	// If subnet node is absent from inclusion in consensus information or attestings, or validator data isn't attested
	// We don't count penalties per account because a user can bypass this by having multiple accounts
	#[pallet::storage] // subnet_id -> class_id -> BTreeMap(account_id, block)
	pub type SubnetNodePenalties<T: Config> = StorageDoubleMap<
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
	pub fn DefaultNodeAttestationRemovalThreshold() -> u128 {
		// 8500
		850000000
	}

	// Attestion percentage required to increment a nodes penalty count up
	#[pallet::storage]
	pub type NodeAttestationRemovalThreshold<T: Config> = StorageValue<_, u128, ValueQuery, DefaultNodeAttestationRemovalThreshold>;


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
	#[pallet::storage] // subnet_uid --> peer_data
	#[pallet::getter(fn total_subnet_stake)]
	pub type TotalSubnetStake<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u128, ValueQuery>;

	// An accounts stake per subnet
	#[pallet::storage] // account--> subnet_id --> u128
	#[pallet::getter(fn account_subnet_stake)]
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

	#[pallet::storage]
	pub type SubnetStakeUnbondingLedger<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Identity,
		u32,
		BTreeMap<u64, u128>,
		ValueQuery,
		DefaultSubnetStakeUnbondingLedger,
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

	//
	// Delegate Staking
	// 

	/// The minimum delegate stake percentage for a subnet to have at any given time
	// Based on the minimum subnet stake balance based on minimum nodes for a subnet
	// i.e. if a subnet has a minimum node required of 10 and 100 TENSOR per node, and a MinSubnetDelegateStakePercentage of 100%
	//			then the minimum delegate stake balance towards a subnet must be 1000 TENSOR
	#[pallet::storage]
	pub type MinSubnetDelegateStakePercentage<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMinSubnetDelegateStakePercentage>;

	/// The absolute minimum delegate stake balance required for a subnet to stay activated
	#[pallet::storage]
	pub type MinSubnetDelegateStake<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMinSubnetDelegateStake>;

	#[pallet::storage]
	pub type MinDelegateStakeBalance<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMinDelegateStakeBalance>;

	#[pallet::storage]
	pub type MaxDelegateStakeBalance<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMaxDelegateStakeBalance>;

	/// The required blocks between delegate stake transfers
	#[pallet::storage]
	pub type DelegateStakeTransferPeriod<T: Config> = StorageValue<_, u64, ValueQuery, DefaultDelegateStakeTransferPeriod>;

	#[pallet::storage]
	pub type LastDelegateStakeTransfer<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery, DefaultZeroU64>;

	// #[pallet::storage] // subnet_id --> (account_id, (initialized or removal block))
	// pub type SubnetAccountDelegateStake<T: Config> = StorageMap<
	// 	_,
	// 	Blake2_128Concat,
	// 	u32,
	// 	BTreeMap<T::AccountId, u64>,
	// 	ValueQuery,
	// >;

	// Percentage of epoch rewards that go towards delegate stake pools
	#[pallet::storage]
	pub type DelegateStakeRewardsPercentage<T: Config> = StorageValue<_, u128, ValueQuery, DefaultDelegateStakeRewardsPercentage>;

	// #[pallet::storage]
	// pub type MinRequiredDelegateUnstakeEpochs<T> = StorageValue<_, u64, ValueQuery, DefaultMinRequiredDelegateUnstakeEpochs>;

	// Total stake sum of all peers in specified subnet
	#[pallet::storage] // subnet_uid --> peer_data
	pub type TotalSubnetDelegateStakeShares<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u128, ValueQuery>;

	// Total stake sum of all peers in specified subnet
	#[pallet::storage] // subnet_uid --> peer_data
	pub type TotalSubnetDelegateStakeBalance<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u128, ValueQuery>;

	// An accounts delegate stake per subnet
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

	#[pallet::storage]
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
			attests: BTreeMap::new(),
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
			eligible_voters: BTreeSet::new(),
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
	pub fn DefaultProposalMinSubnetNodes() -> u32 {
		16
	}

	#[pallet::storage] 
	pub type ProposalMinSubnetNodes<T> = StorageValue<_, u32, ValueQuery, DefaultProposalMinSubnetNodes>;

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
		// 75.0%
		750000000
	}

	#[pallet::storage] // How many voters are needed in a subnet proposal
	pub type ProposalQuorum<T> = StorageValue<_, u128, ValueQuery, DefaultProposalQuorum>;

	#[pallet::type_value]
	pub fn DefaultProposalConsensusThreshold() -> u128 {
		// 66.0%
		660000000
	}

	// Consensus required to pass proposal
	#[pallet::storage]
	pub type ProposalConsensusThreshold<T> = StorageValue<_, u128, ValueQuery, DefaultProposalConsensusThreshold>;

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
		/// Anyone can remove a subnet if:
		/// - The time period allotted to initialize a subnet to reach consensus submissions
		///   and
		/// - The delegate stake balance is below the minimum required threshold
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn remove_subnet(
			origin: OriginFor<T>, 
			subnet_id: u32,
		) -> DispatchResult {
			// let account_id: T::AccountId = ensure_signed(origin)?;
			ensure_signed(origin)?;

			// let account_id = match origin {
			// 	Ok(ensure_signed(origin)) => account_id
			// };

			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
			};

			// ----
			// Subnets can be removed by
			// 		1. Subnet can be voted off
			//		2. Subnet can reach max penalty
			//		2.a. Subnet has min peers after initialization period (increases penalty score)
			// ----
			
			let min_required_subnet_consensus_submit_epochs = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();
			let block: u64 = Self::get_current_block_as_u64();

			// --- Ensure the subnet has passed it's required period to begin consensus submissions
			ensure!(
				block > Self::get_eligible_epoch_block(
					T::EpochLength::get(), 
					subnet.initialized, 
					min_required_subnet_consensus_submit_epochs
				),
				Error::<T>::SubnetInitializing
			);

			let penalties = SubnetPenaltyCount::<T>::get(subnet_id);

			let subnet_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
			let min_subnet_delegate_stake_balance = Self::get_min_subnet_delegate_stake_balance(subnet.min_nodes);

			// TODO: Check min nodes count against submittable nodes classification

			if penalties > MaxSubnetPenaltyCount::<T>::get() {
				// --- If the subnet has reached max penalty, remove it
        Self::deactivate_subnet(
          subnet.path,
          SubnetRemovalReason::MaxPenalties,
        ).map_err(|e| e)?;
			} else if subnet_delegate_stake_balance < min_subnet_delegate_stake_balance {
				// --- If the delegate stake balance is below minimum threshold, remove it
        Self::deactivate_subnet(
          subnet.path,
          SubnetRemovalReason::MinSubnetDelegateStake,
        ).map_err(|e| e)?;
			}

			// --- If we make it to here, fail the extrinsic
			Err(Error::<T>::InvalidSubnetRemoval.into())
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
			a: Option<BoundedVec<u8, DefaultSubnetNodeParamLimit>>,
			b: Option<BoundedVec<u8, DefaultSubnetNodeParamLimit>>,
			c: Option<BoundedVec<u8, DefaultSubnetNodeParamLimit>>,
			// signature: T::OffchainSignature,
			// signer: T::AccountId,
		) -> DispatchResult {
			Self::do_register_subnet_node(
				origin.clone(),
				subnet_id,
				peer_id,
				stake_to_be_added,
				a,
				b,
				c,
			).map_err(|e| e)?;

			Self::do_activate_subnet_node(
				origin.clone(),
				subnet_id,
			)
		}

		#[pallet::call_index(2)]
		#[pallet::weight({0})]
		pub fn register_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			peer_id: PeerId, 
			stake_to_be_added: u128,
			a: Option<BoundedVec<u8, DefaultSubnetNodeParamLimit>>,
			b: Option<BoundedVec<u8, DefaultSubnetNodeParamLimit>>,
			c: Option<BoundedVec<u8, DefaultSubnetNodeParamLimit>>,
		) -> DispatchResult {
			Self::do_register_subnet_node(
				origin,
				subnet_id,
				peer_id,
				stake_to_be_added,
				a,
				b,
				c,
			)
		}
		
		#[pallet::call_index(3)]
		#[pallet::weight({0})]
		pub fn activate_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
		) -> DispatchResult {
			Self::do_activate_subnet_node(
				origin,
				subnet_id,
			)
		}	

		/// Update a subnet peer
		#[pallet::call_index(4)]
		#[pallet::weight({0})]
		pub fn deactivate_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
		) -> DispatchResult {
			Self::do_deactivate_subnet_node(
				origin,
				subnet_id,
			)
		}
		
		/// Remove your subnet peer
		/// Unstaking must be done seperately
		/// Infallible
		#[pallet::call_index(5)]
		// #[pallet::weight(T::WeightInfo::remove_subnet_node())]
		#[pallet::weight({0})]
		pub fn remove_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

			Self::do_remove_subnet_node(account_id, subnet_id)
		}

		/// Remove a subnet peer that has surpassed the max penalties allowed
		// This is redundant 
		#[pallet::call_index(6)]
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

			// Self::do_remove_account_subnet_nodes(block, account_id);

			Ok(())
		}

		
		/// Increase stake towards the specified subnet ID
		#[pallet::call_index(7)]
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
		#[pallet::call_index(8)]
		#[pallet::weight({0})]
		pub fn remove_stake(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			stake_to_be_removed: u128
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin.clone())?;

			// If account is a peer they can remove stake up to minimum required stake balance
			// Else they can remove entire balance because they are not hosting subnets according to consensus
			//		They are removed in `do_remove_subnet_node()` when self or consensus removed
			let is_subnet_node: bool = match SubnetNodesData::<T>::try_get(subnet_id, account_id.clone()) {
				Ok(_) => true,
				Err(()) => false,
			};

			// Remove stake
			// 		if_peer: cannot remove stake below minimum required stake
			// 		else: can remove total stake balance
			Self::do_remove_stake(
				origin, 
				subnet_id,
				account_id,
				is_subnet_node,
				stake_to_be_removed,
			)
		}

		#[pallet::call_index(9)]
		#[pallet::weight({0})]
		pub fn claim_stake_unbondings(
			origin: OriginFor<T>, 
			subnet_id: u32, 
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
			let successful_unbondings: u32 = Self::do_claim_stake_unbondings(&account_id, subnet_id);
			ensure!(
				successful_unbondings > 0,
        Error::<T>::NoStakeUnbondingsOrCooldownNotMet
			);
			Ok(())
		}

		/// Increase stake towards subnet ID
		#[pallet::call_index(10)]
		// #[pallet::weight(T::WeightInfo::add_to_delegate_stake())]
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

			Self::do_add_delegate_stake(
				origin, 
				subnet_id,
				stake_to_be_added,
			)
		}
		
		/// Swaps the balance of the ``from_subnet_id`` shares to ``to_subnet_id``
		#[pallet::call_index(11)]
		#[pallet::weight({0})]
		pub fn transfer_delegate_stake(
			origin: OriginFor<T>, 
			from_subnet_id: u32, 
			to_subnet_id: u32, 
			delegate_stake_shares_to_be_switched: u128
		) -> DispatchResult {
			// --- Ensure ``to`` subnet exists
			ensure!(
				SubnetsData::<T>::contains_key(to_subnet_id),
				Error::<T>::SubnetNotExist
			);

			// TODO: Ensure users aren't hopping from one subnet to another to get both rewards
			// Check if both subnets have generated rewards
			// --- Only allow one ``transfer_delegate_stake`` per epoch or every other epoch

			// Handles ``ensure_signed``
			Self::do_switch_delegate_stake(
				origin, 
				from_subnet_id,
				to_subnet_id,
				delegate_stake_shares_to_be_switched,
			)
		}

		/// Remove delegate stake and add to delegate stake unboding ledger
		/// Enter shares and will convert to balance automatically
		#[pallet::call_index(12)]
		// #[pallet::weight(T::WeightInfo::remove_delegate_stake())]
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

		#[pallet::call_index(13)]
		// #[pallet::weight(T::WeightInfo::claim_delegate_stake_unbondings())]
		#[pallet::weight({0})]
		pub fn claim_delegate_stake_unbondings(
			origin: OriginFor<T>, 
			subnet_id: u32, 
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
			let successful_unbondings: u32 = Self::do_claim_delegate_stake_unbondings(&account_id, subnet_id);
			ensure!(
				successful_unbondings > 0,
        Error::<T>::NoDelegateStakeUnbondingsOrCooldownNotMet
			);
			Ok(())
		}
		
		/// Allows anyone to increase a subnets delegate stake pool
		#[pallet::call_index(14)]
		#[pallet::weight({0})]
		pub fn increase_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			amount: u128,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
			
			// --- Ensure subnet exists, otherwise at risk of burning tokens
			ensure!(
				SubnetsData::<T>::contains_key(subnet_id),
				Error::<T>::SubnetNotExist
			);

			let amount_as_balance = Self::u128_to_balance(amount);

			ensure!(
				amount_as_balance.is_some(),
				Error::<T>::CouldNotConvertToBalance
			);
	
			// --- Ensure the callers account_id has enough balance to perform the transaction.
			ensure!(
				Self::can_remove_balance_from_coldkey_account(&account_id, amount_as_balance.unwrap()),
				Error::<T>::NotEnoughBalance
			);
	
			// --- Ensure the remove operation from the account_id is a success.
			ensure!(
				Self::remove_balance_from_coldkey_account(&account_id, amount_as_balance.unwrap()) == true,
				Error::<T>::BalanceWithdrawalError
			);
			
			Self::do_increase_delegate_stake(
				subnet_id,
				amount,
			);

			Ok(())
		}

		/// Delete proposals that are no longer live
		#[pallet::call_index(15)]
		#[pallet::weight({0})]
		pub fn validate(
			origin: OriginFor<T>, 
			subnet_id: u32,
			data: Vec<SubnetNodeData>,
		) -> DispatchResultWithPostInfo {
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

		#[pallet::call_index(16)]
		#[pallet::weight({0})]
		pub fn attest(
			origin: OriginFor<T>, 
			subnet_id: u32,
		) -> DispatchResultWithPostInfo {
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

		#[pallet::call_index(17)]
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

		#[pallet::call_index(18)]
		#[pallet::weight({0})]
		pub fn attest_proposal(
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

		#[pallet::call_index(19)]
		#[pallet::weight({0})]
		pub fn cancel_proposal(
			origin: OriginFor<T>, 
			subnet_id: u32,
			proposal_id: u32,
	) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_cancel_proposal(
				account_id,
				subnet_id,
				proposal_id,
			)
		}

		#[pallet::call_index(20)]
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

		#[pallet::call_index(21)]
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

		#[pallet::call_index(22)]
		#[pallet::weight({0})]
		pub fn finalize_proposal(
			origin: OriginFor<T>, 
			subnet_id: u32,
			proposal_id: u32,
	) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_finalize_proposal(
				account_id,
				subnet_id,
				proposal_id,
			)
		}

		#[pallet::call_index(23)]
		#[pallet::weight({0})]
		pub fn reward_subnet(
			origin: OriginFor<T>, 
			subnet_id: u32,
			epoch: u32,
	) -> DispatchResultWithPostInfo {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			// Self::do_reward_subnet(
			// 	subnet_id,
			// 	epoch,
			// )
			Ok(None.into())
		}

		#[pallet::call_index(24)]
		#[pallet::weight({0})]
		pub fn register_subnet(
			origin: OriginFor<T>, 
			subnet_data: RegistrationSubnetData,
	) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_register_subnet(
				account_id,
				subnet_data,
			)
		}

		#[pallet::call_index(25)]
		#[pallet::weight({0})]
		pub fn activate_subnet(
			origin: OriginFor<T>, 
			subnet_id: u32,
	) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_activate_subnet(subnet_id)
		}
	}

	impl<T: Config> Pallet<T> {
		/// Activate subnet - called by subnet democracy logic
		pub fn do_register_subnet(
			activator: T::AccountId,
			subnet_data: RegistrationSubnetData,
		) -> DispatchResult {
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

			// --- Ensure registration time period is allowed
			ensure!(
				subnet_data.registration_blocks >= MinSubnetRegistrationBlocks::<T>::get() && 
				subnet_data.registration_blocks <= MaxSubnetRegistrationBlocks::<T>::get(),
				Error::<T>::InvalidSubnetRegistrationBlocks
			);

			// --- Ensure memory under max
			ensure!(
				subnet_data.memory_mb <= MaxSubnetMemoryMB::<T>::get(),
				Error::<T>::MaxSubnetMemory
			);

			let block: u64 = Self::get_current_block_as_u64();
			let subnet_cost: u128 = Self::get_subnet_initialization_cost(block);

			if subnet_cost > 0 {
				// unreserve from activator
				let subnet_cost_as_balance = Self::u128_to_balance(subnet_cost);

				ensure!(
					Self::can_remove_balance_from_coldkey_account(&activator, subnet_cost_as_balance.unwrap()),
					Error::<T>::NotEnoughBalanceToStake
				);
		
				ensure!(
					Self::remove_balance_from_coldkey_account(&activator, subnet_cost_as_balance.unwrap()) == true,
					Error::<T>::BalanceWithdrawalError
				);

				// TODO
				// Send portion to stake rewards vault
				// Send portion to treasury

				// increase stake balance with subnet initialization cost
				StakeVaultBalance::<T>::mutate(|n: &mut u128| *n += subnet_cost);
			}

			// Get total subnets ever
			let subnet_len: u32 = TotalSubnets::<T>::get();
			// Start the subnet_ids at 1
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
				registration_blocks: subnet_data.clone().registration_blocks,
				initialized: block,
				activated: 0,
			};

			// Store unique path
			SubnetPaths::<T>::insert(subnet_data.clone().path, subnet_id);
			// Store subnet data
			SubnetsData::<T>::insert(subnet_id, subnet_data.clone());
			// Increase total subnets. This is used for unique Subnet IDs
			TotalSubnets::<T>::mutate(|n: &mut u32| *n += 1);

			Self::deposit_event(Event::SubnetAdded { 
				activator: activator,
				subnet_id: subnet_id, 
				subnet_path: subnet_data.path,
				block: block
			});

			Ok(())
		}

		pub fn do_activate_subnet(subnet_id: u32) -> DispatchResult {
			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::InvalidSubnetId.into()),
			};

			ensure!(
				subnet.activated == 0,
				Error::<T>::SubnetActivatedAlready
			);

			let block: u64 = Self::get_current_block_as_u64();

			// --- Ensure the subnet has passed it's required period to begin consensus submissions
			// --- Ensure the subnet is within the enactment period
			ensure!(
				block > subnet.initialized + subnet.registration_blocks,
				Error::<T>::SubnetInitializing
			);

			// --- If subnet not activated yet and is outside the enactment period, remove subnet
			if block > subnet.initialized + subnet.registration_blocks + SubnetActivationEnactmentPeriod::<T>::get() {
				return Self::deactivate_subnet(
					subnet.path,
					SubnetRemovalReason::EnactmentPeriod,
				)
			}

			// --- Ensure minimum nodes are required
			if TotalSubnetNodes::<T>::get(subnet_id) < subnet.min_nodes {
				return Self::deactivate_subnet(
					subnet.path,
					SubnetRemovalReason::MinSubnetNodes,
				)
			}

			// --- Ensure minimum delegate stake achieved 
			let subnet_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
			let min_subnet_delegate_stake_balance = Self::get_min_subnet_delegate_stake_balance(subnet.min_nodes);

			// --- Ensure delegate stake balance is below minimum threshold required
			if subnet_delegate_stake_balance < min_subnet_delegate_stake_balance {
				return Self::deactivate_subnet(
					subnet.path,
					SubnetRemovalReason::MinSubnetDelegateStake,
				)
			}

			// --- Activate subnet
			SubnetsData::<T>::try_mutate(
				subnet_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetId)?;
					ensure!(
						params.activated == 0,
						Error::<T>::SubnetAlreadyActivated
					);
					params.activated = block;
					Ok(())
				}
			)?;

			// --- Choose validator for next epoch
			let epoch_length: u64 = T::EpochLength::get();
			let next_epoch: u64 = block / epoch_length + 1;

			Self::choose_validator(block, subnet_id, subnet.min_nodes, next_epoch as u32);

      // Self::deposit_event(Event::SubnetActivated { 
      //   subnet_id: subnet_id, 
      //   validator: validator,
      //   block: block
      // });
	
			Ok(())
		}

		pub fn deactivate_subnet(
			path: Vec<u8>,
			reason: SubnetRemovalReason,
		) -> DispatchResult {
			ensure!(
				SubnetPaths::<T>::contains_key(path.clone()),
				Error::<T>::SubnetNotExist
			);

			let subnet_id = SubnetPaths::<T>::get(path.clone()).unwrap();

			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
			};

			// Remove unique path
			SubnetPaths::<T>::remove(path.clone());
			// Remove subnet data
			SubnetsData::<T>::remove(subnet_id);

			// We don't subtract TotalSubnets since it's used for ids

			// Remove all peers data
			let _ = SubnetNodesData::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let _ = TotalSubnetNodes::<T>::remove(subnet_id);
			let _ = SubnetNodeAccount::<T>::clear_prefix(subnet_id, u32::MAX, None);

			// Remove all subnet consensus data
			let _ = SubnetPenaltyCount::<T>::remove(subnet_id);

			// Remove consensus data
			let _ = SubnetRewardsSubmission::<T>::clear_prefix(subnet_id, u32::MAX, None);

			// Remove accounting data
			let _ = AccountantData::<T>::clear_prefix(subnet_id, u32::MAX, None);

			// Remove proposals
			let _ = Proposals::<T>::clear_prefix(subnet_id, u32::MAX, None);
	
			Self::deposit_event(Event::SubnetRemoved { 
				subnet_id: subnet_id, 
				subnet_path: path,
				reason: reason,
			});

			Ok(())
		}

		pub fn do_remove_subnet_node(
			account_id: T::AccountId,
			subnet_id: u32,
		) -> DispatchResult {
			let block: u64 = Self::get_current_block_as_u64();

			// TODO: Track removal of subnet nodes following validator consensus data submission per epoch

			// We don't check consensus steps here because a subnet peers stake isn't included in calculating rewards 
			// that hasn't reached their consensus submission epoch yet
			Self::perform_remove_subnet_node(block, subnet_id, account_id.clone());

			Ok(())
		}

		pub fn do_register_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			peer_id: PeerId, 
			stake_to_be_added: u128,
			a: Option<BoundedVec<u8, DefaultSubnetNodeParamLimit>>,
			b: Option<BoundedVec<u8, DefaultSubnetNodeParamLimit>>,
			c: Option<BoundedVec<u8, DefaultSubnetNodeParamLimit>>,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin.clone())?;

			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
			};
			
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

			// Unique ``a``
			// [here]
			if a.is_some() {
				ensure!(
					!SubnetNodeParam::<T>::contains_key(subnet_id, a.unwrap()),
					Error::<T>::SubnetNodeExist
				);	
			}

			// Unique subnet_id -> PeerId
			// Ensure peer ID doesn't already exist within subnet regardless of account_id
			match SubnetNodeAccount::<T>::try_get(subnet_id, peer_id.clone()) {
				Ok(_) => return Err(Error::<T>::PeerIdExist.into()),
				Err(()) => (),
			};
	
			// Validate peer_id
			ensure!(
				Self::validate_peer_id(peer_id.clone()),
				Error::<T>::InvalidPeerId
			);

			// --- Ensure they have no stake on registration
			// If a subnet node deregisters, then they must fully unstake its stake balance to register again using that same balance
			ensure!(
				AccountSubnetStake::<T>::get(account_id.clone(), subnet_id) == 0,
				Error::<T>::InvalidSubnetRegistrationCooldown
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
			let epoch_length: u64 = T::EpochLength::get();
			let block: u64 = Self::get_current_block_as_u64();
			let epoch: u64 = block / epoch_length;

			// ========================
			// Insert peer into storage
			// ========================
			let mut classification: SubnetNodeClassification = SubnetNodeClassification {
				class: SubetNodeClass::Registered,
				start_epoch: epoch,
			};
			// --- Initial nodes before activation are entered as ``submittable`` nodes
			// They initiate the first consensus epoch and are responsible for increasing classifications
			// of other nodes that come in post activation
			if subnet.activated == 0 {
				classification = SubnetNodeClassification {
					class: SubetNodeClass::Submittable,
					start_epoch: epoch,
				};
			}

			let subnet_node: SubnetNode<T::AccountId> = SubnetNode {
				account_id: account_id.clone(),
				hotkey: account_id.clone(),
				peer_id: peer_id.clone(),
				initialized: 0,
				classification: classification,
				a: Vec::new(),
				b: Vec::new(),
				c: Vec::new(),
			};
			// Insert SubnetNodesData with account_id as key
			SubnetNodesData::<T>::insert(subnet_id, account_id.clone(), subnet_node);

			// Insert subnet peer account to keep peer_ids unique within subnets
			SubnetNodeAccount::<T>::insert(subnet_id, peer_id.clone(), account_id.clone());

			// --- If subnet not activated, send subnet node to submission class, otherwise send to idle
			let mut classification = SubnetNodeClass::Idle;
			if subnet.activated == 0 {
				classification = SubnetNodeClass::Submittable;
			}

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

		pub fn do_activate_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

			let epoch_length: u64 = T::EpochLength::get();
			let block: u64 = Self::get_current_block_as_u64();
			let epoch: u64 = block / epoch_length;

			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
			};

			SubnetNodesData::<T>::try_mutate_exists(
				subnet_id,
				account_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::SubnetNodeExist)?;	
					ensure!(
						params.initialized == 0,
            Error::<T>::SubnetNodeAlreadyActivated
					);
					let mut class = SubetNodeClass::Idle;
					let mut epoch_increase = 0;
					if subnet.activated == 0 {
						class = SubetNodeClass::Submittable
					} else {
						epoch_increase += 1;
					}
					params.initialized = block;
					params.classification = SubnetNodeClassification {
						class: class,
						start_epoch: epoch + epoch_increase,
					};
					Ok(())
				}
			)?;

			TotalActiveSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
	
			Ok(())
		}

		/// This should be called by a user facing extrinsic
		pub fn do_deactivate_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

			let epoch_length: u64 = T::EpochLength::get();
			let block: u64 = Self::get_current_block_as_u64();
			let epoch: u64 = block / epoch_length;

			SubnetNodesData::<T>::try_mutate_exists(
				subnet_id,
				account_id.clone(),
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::SubnetNodeExist)?;	
					ensure!(
						params.initialized != 0,
            Error::<T>::SubnetNodeNotActivated
					);
					params.initialized = 0;
					params.classification = SubnetNodeClassification {
						class: SubetNodeClass::Registered,
						start_epoch: epoch,
					};
					Ok(())
				}
			)?;

			TotalActiveSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| n.saturating_dec());

			Ok(())
		}

	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
			let block: u64 = Self::convert_block_as_u64(block_number);
			let epoch_length: u64 = T::EpochLength::get();

			// Reward validators and attestors... Shift node classes
			if block >= epoch_length && block % epoch_length == 0 {
				let epoch: u64 = block / epoch_length;

				// Reward subnets for the previous epoch
				// Reward before shifting
				Self::reward_subnets(block, (epoch - 1) as u32, epoch_length);

				// return T::WeightInfo::on_initialize_reward_subnets();
				return Weight::from_parts(207_283_478_000, 22166406)
					.saturating_add(T::DbWeight::get().reads(18250_u64))
					.saturating_add(T::DbWeight::get().writes(12002_u64));

			} else if (block - 1) >= epoch_length && (block - 1) % epoch_length == 0 {
				// We save some weight by waiting one more block to choose validators
				// Run the block succeeding form consensus
				let epoch: u64 = block / epoch_length;

				// Choose validators and accountants for the current epoch
				Self::do_choose_validator_and_accountants(block, epoch as u32, epoch_length);

				// return T::WeightInfo::on_initialize_do_choose_validator_and_accountants();
				return Weight::from_parts(207_283_478_000, 22166406)
					.saturating_add(T::DbWeight::get().reads(18250_u64))
					.saturating_add(T::DbWeight::get().writes(12002_u64));
			}

			// return T::WeightInfo::on_initialize()
			return Weight::from_parts(207_283_478_000, 22166406)
				.saturating_add(T::DbWeight::get().reads(18250_u64))
				.saturating_add(T::DbWeight::get().writes(12002_u64))
		}

		fn on_idle(block_number: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
			let block: u64 = Self::convert_block_as_u64(block_number);

			if remaining_weight.any_lt(T::DbWeight::get().reads(2)) {
				return Weight::from_parts(0, 0)
			}

			return Weight::from_parts(0, 0)

			// Self::do_on_idle(remaining_weight)
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn do_on_idle(remaining_weight: Weight) -> Weight {
			// any weight that is unaccounted for
			let mut unaccounted_weight = Weight::from_parts(0, 0);

			unaccounted_weight
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
			// SubnetNodeClassEpochs::<T>::insert(SubnetNodeClass::Idle, 2);
			// SubnetNodeClassEpochs::<T>::insert(SubnetNodeClass::Included, 4);
			// SubnetNodeClassEpochs::<T>::insert(SubnetNodeClass::Submittable, 6);
			// SubnetNodeClassEpochs::<T>::insert(SubnetNodeClass::Accountant, 8);

			// let subnet_id = 1;

			// let base_node_memory: u128 = BaseSubnetNodeMemoryMB::<T>::get();

			// // --- Get min nodes based on default memory settings
			// let real_min_subnet_nodes: u128 = self.memory_mb.clone() / base_node_memory;
			// let mut min_subnet_nodes: u32 = MinSubnetNodes::<T>::get();
			// if real_min_subnet_nodes as u32 > min_subnet_nodes {
			// 	min_subnet_nodes = real_min_subnet_nodes as u32;
			// }
				
			// let target_subnet_nodes: u32 = (min_subnet_nodes as u128).saturating_mul(TargetSubnetNodesMultiplier::<T>::get()).saturating_div(10000) as u32 + min_subnet_nodes;

			// let subnet_data = SubnetData {
			// 	id: subnet_id,
			// 	path: self.subnet_path.clone(),
			// 	min_nodes: min_subnet_nodes,
			// 	target_nodes: target_subnet_nodes,
			// 	memory_mb: self.memory_mb.clone(),
			// 	initialized: 0,
			// };

			// // Activate subnet
			// let pre_subnet_data = RegistrationSubnetData {
			// 	path: self.subnet_path.clone(),
			// 	memory_mb: self.memory_mb.clone(),
			// };
		
			// let vote_subnet_data = SubnetDemocracySubnetData {
			// 	data: pre_subnet_data,
			// 	active: true,
			// };

			// SubnetActivated::<T>::insert(self.subnet_path.clone(), vote_subnet_data);
			// // Store unique path
			// SubnetPaths::<T>::insert(self.subnet_path.clone(), subnet_id);
			// // Store subnet data
			// SubnetsData::<T>::insert(subnet_id, subnet_data.clone());
			// // Increase total subnets count
			// TotalSubnets::<T>::mutate(|n: &mut u32| *n += 1);

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
			// 		hotkey: account_id.clone(),
			// 		peer_id: peer_id.clone(),
			// 		initialized: 0,
			// 	};
			// 	// Insert SubnetNodesData with account_id as key
			// 	SubnetNodesData::<T>::insert(subnet_id, account_id.clone(), subnet_node);

			// 	// Insert subnet peer account to keep peer_ids unique within subnets
			// 	SubnetNodeAccount::<T>::insert(subnet_id, peer_id.clone(), account_id.clone());

			// 	if let Ok(mut node_class) = SubnetNodesClasses::<T>::try_get(subnet_id, SubnetNodeClass::Idle) {
			// 		node_class.insert(account_id.clone(), 0);
			// 		SubnetNodesClasses::<T>::insert(subnet_id, SubnetNodeClass::Idle, node_class);	
			// 	} else {
			// 		// If new subnet, initialize classes
			// 		let mut node_class: BTreeMap<T::AccountId, u64> = BTreeMap::new();
			// 		node_class.insert(account_id.clone(), 0);
			// 		SubnetNodesClasses::<T>::insert(subnet_id, SubnetNodeClass::Idle, node_class);	
			// 	}

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

impl<T: Config> SubnetVote<OriginFor<T>, T::AccountId> for Pallet<T> {
	fn vote_subnet_in(vote_subnet_data: SubnetDemocracySubnetData) -> DispatchResult {
		SubnetActivated::<T>::insert(vote_subnet_data.clone().data.path, vote_subnet_data.clone());
		Ok(())
	}
	fn vote_subnet_out(vote_subnet_data: SubnetDemocracySubnetData) -> DispatchResult {
		SubnetActivated::<T>::insert(vote_subnet_data.clone().data.path, vote_subnet_data.clone());
		Ok(())
	}
	fn vote_activated(activator: T::AccountId, path: Vec<u8>, proposer: T::AccountId, vote_subnet_data: SubnetDemocracySubnetData) -> DispatchResult {
		SubnetActivated::<T>::insert(path, vote_subnet_data.clone());

		// Self::activate_subnet(
		// 	activator, 
		// 	proposer,
		// 	vote_subnet_data.clone().data,
		// )
		Ok(())
	}
	fn vote_deactivated(deactivator: T::AccountId, path: Vec<u8>, proposer: T::AccountId, vote_subnet_data: SubnetDemocracySubnetData) -> DispatchResult {
		SubnetActivated::<T>::insert(path, vote_subnet_data.clone());

		Self::deactivate_subnet(
			vote_subnet_data.clone().data.path,
			SubnetRemovalReason::SubnetDemocracy
		)
	}
	fn vote_add_subnet_node(
		origin: OriginFor<T>, 
		subnet_id: u32, 
		peer_id: PeerId, 
		stake_to_be_added: u128,
	) -> DispatchResult {
		Ok(())
	}
	fn get_total_subnets() -> u32 {
		TotalSubnets::<T>::get()
	}
	fn get_subnet_initialization_cost() -> u128 {
		let block: u64 = Self::get_current_block_as_u64();
		Self::get_subnet_initialization_cost(block)
	}
	fn get_subnet_path_exist(path: Vec<u8>) -> bool {
		if SubnetPaths::<T>::contains_key(path) {
			true
		} else {
			false
		}
	}
	fn get_subnet_id_by_path(path: Vec<u8>) -> u32 {
		if !SubnetPaths::<T>::contains_key(path.clone()) {
			return 0
		} else {
			return SubnetPaths::<T>::get(path.clone()).unwrap()
		}
	}
	fn get_subnet_id_exist(id: u32) -> bool {
		if SubnetsData::<T>::contains_key(id) {
			true
		} else {
			false
		}
	}
	// Should never be called unless contains_key is confirmed
	fn get_subnet_data(id: u32) -> SubnetData {
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
	fn is_submittable_subnet_node_account(account_id: T::AccountId) -> bool {
		true
	}
	fn is_subnet_initialized(id: u32) -> bool {
		let subnet_data = SubnetsData::<T>::get(id).unwrap();
		let subnet_initialized = subnet_data.initialized;

		let epoch_length: u64 = T::EpochLength::get();
		let min_required_subnet_consensus_submit_epochs = MinRequiredSubnetConsensusSubmitEpochs::<T>::get();
		let block: u64 = Self::get_current_block_as_u64();

		block >= Self::get_eligible_epoch_block(
			epoch_length, 
			subnet_initialized, 
			min_required_subnet_consensus_submit_epochs
		)
	}
	fn get_total_subnet_errors(id: u32) -> u32 {
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
	fn get_stake_balance(account_id: T::AccountId) -> u128 {
		Self::get_account_total_stake_balance(account_id)
	}
	fn get_delegate_stake_balance(account_id: T::AccountId) -> u128 {
		let mut total_delegate_stake_balance = 0;
		for (subnet_id, _) in SubnetsData::<T>::iter() {
			total_delegate_stake_balance += Self::convert_account_shares_to_balance(
				&account_id,
				subnet_id
			);
		}
		total_delegate_stake_balance
	}
	fn get_voting_power() -> u128 {
		Self::get_total_voting_power()
	}
}

pub trait SubnetVote<OriginFor, AccountId> {
	fn vote_subnet_in(vote_subnet_data: SubnetDemocracySubnetData) -> DispatchResult;
	fn vote_subnet_out(vote_subnet_data: SubnetDemocracySubnetData) -> DispatchResult;
	fn vote_activated(activator: AccountId, path: Vec<u8>, proposer: AccountId, vote_subnet_data: SubnetDemocracySubnetData) -> DispatchResult;
	fn vote_deactivated(deactivator: AccountId, path: Vec<u8>, proposer: AccountId, vote_subnet_data: SubnetDemocracySubnetData) -> DispatchResult;
	fn vote_add_subnet_node(
		origin: OriginFor, 
		subnet_id: u32, 
		peer_id: PeerId, 
		stake_to_be_added: u128,
	) -> DispatchResult;
	fn get_total_subnets() -> u32;
	fn get_subnet_initialization_cost() -> u128;
	fn get_subnet_path_exist(path: Vec<u8>) -> bool;
	fn get_subnet_id_by_path(path: Vec<u8>) -> u32;
	fn get_subnet_id_exist(id: u32) -> bool;
	fn get_subnet_data(id: u32) -> SubnetData;
	fn get_max_subnet_nodes() -> u32;
	fn get_min_stake_balance() -> u128;
	fn is_submittable_subnet_node_account(account_id: AccountId) -> bool;
	fn is_subnet_initialized(id: u32) -> bool;
	fn get_total_subnet_errors(id: u32) -> u32;
	fn get_min_subnet_nodes(memory_mb: u128) -> u32;
	fn get_target_subnet_nodes(min_subnet_nodes: u32) -> u32;
	fn get_stake_balance(account_id: AccountId) -> u128;
	fn get_delegate_stake_balance(account_id: AccountId) -> u128;
	fn get_voting_power() -> u128;
}

// impl<T: Config> Tester<OriginFor<T>, T::AccountId> for Pallet<T> {
// 	// pub trait OriginFor<T> = <T as crate::Config>::RuntimeOrigin;
// 	fn test_function(account_id: T::AccountId) -> u128 {
// 		1
// 	}
// 	fn test_function2(origin: OriginFor<T>) -> u128 {
// 		1
// 	}
// }

// pub trait Tester<OriginFor, AccountId> {
// 	fn test_function(account_id: AccountId) -> u128;
// 	fn test_function2(origin: OriginFor) -> u128;
// }

// Admin logic
impl<T: Config> AdminInterface<T::AccountId> for Pallet<T> {
	fn set_vote_subnet_in(path: Vec<u8>, memory_mb: u128) -> DispatchResult {
		Self::set_vote_subnet_in(path, memory_mb)
	}
	fn set_vote_subnet_out(path: Vec<u8>) -> DispatchResult {
		Self::set_vote_subnet_out(path)
	}
	fn set_max_subnets(value: u32) -> DispatchResult {
		Self::set_max_subnets(value)
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
	fn set_min_required_subnet_consensus_submit_epochs(value: u64) -> DispatchResult {
		Self::set_min_required_subnet_consensus_submit_epochs(value)
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
	fn set_max_subnet_rewards_weight(value: u128) -> DispatchResult {
		Self::set_max_subnet_rewards_weight(value)
	}
	fn set_stake_reward_weight(value: u128) -> DispatchResult {
		Self::set_stake_reward_weight(value)
	}
	fn set_subnet_per_peer_init_cost(value: u128) -> DispatchResult {
		Self::set_subnet_per_peer_init_cost(value)
	}
	fn set_subnet_consensus_unconfirmed_threshold(value: u128) -> DispatchResult {
		Self::set_subnet_consensus_unconfirmed_threshold(value)
	}
	fn set_remove_subnet_node_epoch_percentage(value: u128) -> DispatchResult {
		Self::set_remove_subnet_node_epoch_percentage(value)
	}
	fn council_remove_subnet(path: Vec<u8>) -> DispatchResult {
		Self::deactivate_subnet(
			path,
			SubnetRemovalReason::Council
		)
	}
	fn council_remove_subnet_node(account_id: T::AccountId, subnet_id: u32) -> DispatchResult {
		Self::do_remove_subnet_node(
			account_id,
			subnet_id
		)
	}
	fn set_min_nodes_slope_parameters(params: MinNodesCurveParametersSet) -> DispatchResult {
		Self::set_min_nodes_slope_parameters(params)
	}
}

pub trait AdminInterface<AccountId> {
	fn set_vote_subnet_in(path: Vec<u8>, memory_mb: u128) -> DispatchResult;
	fn set_vote_subnet_out(path: Vec<u8>) -> DispatchResult;
	fn set_max_subnets(value: u32) -> DispatchResult;
	fn set_min_subnet_nodes(value: u32) -> DispatchResult;
	fn set_max_subnet_nodes(value: u32) -> DispatchResult;
	fn set_min_stake_balance(value: u128) -> DispatchResult;
	fn set_tx_rate_limit(value: u64) -> DispatchResult;
	fn set_max_consensus_epochs_errors(value: u32) -> DispatchResult;
	fn set_min_required_subnet_consensus_submit_epochs(value: u64) -> DispatchResult;
	fn set_min_required_peer_consensus_submit_epochs(value: u64) -> DispatchResult;
	fn set_min_required_peer_consensus_inclusion_epochs(value: u64) -> DispatchResult;
	fn set_min_required_peer_consensus_dishonesty_epochs(value: u64) -> DispatchResult;	
	fn set_max_outlier_delta_percent(value: u8) -> DispatchResult;
	fn set_subnet_node_consensus_submit_percent_requirement(value: u128) -> DispatchResult;
	fn set_consensus_blocks_interval(value: u64) -> DispatchResult;
	fn set_peer_removal_threshold(value: u128) -> DispatchResult;
	fn set_max_subnet_rewards_weight(value: u128) -> DispatchResult;
	fn set_stake_reward_weight(value: u128) -> DispatchResult;
	fn set_subnet_per_peer_init_cost(value: u128) -> DispatchResult;
	fn set_subnet_consensus_unconfirmed_threshold(value: u128) -> DispatchResult;
	fn set_remove_subnet_node_epoch_percentage(value: u128) -> DispatchResult;
	fn council_remove_subnet(path: Vec<u8>) -> DispatchResult;
	fn council_remove_subnet_node(account_id: AccountId, subnet_id: u32) -> DispatchResult;
	fn set_min_nodes_slope_parameters(params: MinNodesCurveParametersSet) -> DispatchResult;
}