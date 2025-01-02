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

//! Pallet for subnet voting.
//! Sets all parameters required for initializing subnets
// Votes are calculated from the accounts stake balance across all staking options.
// Each account can use its stake balance across all proposals
// e.g. If an account has 1000 tokens staked in total, they can vote on multiple proposals using that 1000
//      in each proposal. (proposal 0: vote(1000), proposal 1: vote(1000), ...) 
//
// [1] Propose
// [2.1] Activate:
//        100% of bootstrap subnet nodes must verify and bond minimum stake balance
// [2.2] Deactivate: Automatically verified
// [3] Activate proposal
// [3.1] Activate:
//        If 100% of bootstrap subnet nodes verified, activating proposal is enabled and must be done manually
// [3.2] Deactivate: Automatically verified (not required to activate proposal)
// [4] Cast Votes
//      Each account can vote based on their stake balance
//        - Blockchain node stake balance
//        - Blockchain delegate stake balance
//        - Subnet node stake balance
//        - Subnet node delegate stake balance
// [5] Execute: If quorum and consensus reached
//      Activate: Add new subnet
//      Deactivate: Remove existing subnet
//



#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_core::{
  OpaquePeerId as PeerId,
  crypto::KeyTypeId,
  Get
};
use frame_system::{
  pallet_prelude::{OriginFor, BlockNumberFor},
  ensure_signed, ensure_root,
  offchain::{
    AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction,
    SignedPayload, Signer, SigningTypes, SubmitTransaction,
  },
};
use frame_support::{
  pallet_prelude::DispatchResult,
  ensure,
  traits::{Currency, LockableCurrency, ReservableCurrency, WithdrawReasons, LockIdentifier},
};
use sp_runtime::Vec;
use sp_runtime::{
  traits::Zero,
  Saturating, Perbill, Percent
};
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

use pallet_network::{SubnetVote, RegistrationSubnetData, SubnetDemocracySubnetData};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

mod types;
mod admin;
mod utils;

pub use types::PropIndex;

const SUBNET_DEMOCRACY_ID: LockIdentifier = *b"sndemocr";

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use frame_support::pallet_prelude::*;
  use sp_runtime::traits::TrailingZeroInput;

  #[pallet::config]
  pub trait Config: frame_system::Config {
  // pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
    /// `rewards` events
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    /// The maximum number of public proposals that can exist at any time.
		#[pallet::constant]
		type MaxActivateProposals: Get<u32>;

    /// The maximum number of public proposals that can exist at any time.
		#[pallet::constant]
		type MaxDeactivateProposals: Get<u32>;

    #[pallet::constant]
		type MaxProposals: Get<u32>;

    /// Minimum amount to stake to create a proposal
    #[pallet::constant]
		type MinProposerStake: Get<u128>;

    #[pallet::constant]
		type Quorum: Get<u128>;

    #[pallet::constant]
		type QuorumVotingPowerPercentage: Get<u8>;

    /// The length of the voting period
    // Must be less than the unstaking period
    #[pallet::constant]
		type VotingPeriod: Get<BlockNumberFor<Self>>;

    /// The period after proposals completion to enact its goal
    #[pallet::constant]
		type EnactmentPeriod: Get<BlockNumberFor<Self>>;

    /// Verification period for to-be subnet nodes to verify their commitment for subnet activation proposals only
    #[pallet::constant]
		type VerifyPeriod: Get<BlockNumberFor<Self>>;

    /// slash percentage for proposer if they cancel before voting begins
    #[pallet::constant]
		type CancelSlashPercent: Get<u8>;

    // type SubnetVote: SubnetVote<Self::AccountId>; 
    type SubnetVote: SubnetVote<OriginFor<Self>, Self::AccountId>; 

    // type Currency: Currency<Self::AccountId> + LockableCurrency<Self::AccountId, Moment = BlockNumberFor<Self>> + Send + Sync;
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + Send + Sync;

    type WeightInfo: WeightInfo;
  }

  	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
    /// Subnet path already exists
		SubnetPathExists,
    /// Subnet proposal invalid - Can't be (Active)
		ProposalInvalid,
    /// Proposal active still
    ProposalActive,
    /// Proposal doesn't exist 
    ProposalNotExist,
    /// Maximum proposals allowed
    MaxActiveProposals,
    /// Maximum activate proposals allowed
    MaxActivateProposals,
    /// Maximum deactivate proposals allowed
    MaxDeactivateProposals,
    /// Proposal voting period closed
    EnactmentPeriodInvalid,
    /// Proposal voting period closed
    VotingPeriodInvalid,
    /// Subnet ID doens't exist
    SubnetIdNotExists,
		/// Minimum required subnet peers not met
		SubnetNodesLengthInvalid,
    /// Minimum required subnet peers not met
		NotEnoughSubnetInitializationBalance,
    /// Minimum required subnet peers stake balance not in wallet
		NotEnoughMinStakeBalance,
    /// Not enough balance to vote
    NotEnoughBalanceToVote,
    /// Could not convert to balance
    CouldNotConvertToBalance,
    /// Proposal type invalid and None
    PropsTypeInvalid,
    /// Vote still active
    VoteActive,
    /// Quorum not reached
    QuorumNotReached,
    /// Executor must be proposer
    NotProposer,
    /// Vote completed alreaady
    VoteComplete,
    /// Enactment period passed
    EnactmentPeriodPassed,
    /// Votes are still open
    VotingOpen,
    /// Vote are not longer open either voting period has passed or proposal no longer active
    VotingNotOpen,
    /// Proposal is not active anymore
    ProposalNotActive,
    /// Proposal has concluded
    Concluded,
    /// Vote balance doesn't exist
    VotesBalanceInvalid,
    /// Vote balance is zero
    VoteBalanceZero,
    InvalidQuorum,
    InvalidNodeVotePremium,
    InvalidPeerId,
    SubnetNodeAlreadyVerified,
    SubnetNodeAlreadyBonded,
    VerifyPeriodPassed,
    ProposalVerified,
    ProposalNotVerified,
    ProposalIsDeactvation,
    /// Not a subnet node for activation proposal
    NotSubnetNode,
    SubnetMemoryIsZero,
  }

  /// `pallet-rewards` events
  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    SubnetVoteInInitialized(Vec<u8>, u64),
    SubnetVoteOutInitialized(u32, u64),
    SubnetVoteInSuccess(Vec<u8>, u64),
    SubnetVoteOutSuccess(u32, u64),
    SetNodeVotePremium(u128),
    SetQuorum(u128),
    SetMajority(u128),
  }

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetNode<AccountId> {
    pub account_id: AccountId,
		pub peer_id: PeerId,
	}

  #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct ActivatePropsParams<AccountId> {
    pub path: Vec<u8>,
		pub subnet_nodes: Vec<SubnetNode<AccountId>>,
    pub end_vote_block: u64,
	}

  #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct PropsParams<AccountId> {
    pub proposer: AccountId,
    pub proposer_stake: u128, // Activate: Non refundable, Deactivate: Refundable
    pub proposal_status: PropsStatus,
    pub proposal_type: PropsType, // Activation or Deactivation
    pub path: Vec<u8>, // path for downloading subnet used in subnet, can be anything (HuggingFace, IPFS, etc.)
    pub subnet_data: RegistrationSubnetData,
		pub subnet_nodes: Vec<SubnetNode<AccountId>>,
    pub subnet_nodes_verified: BTreeSet<AccountId>,
    pub subnet_nodes_bonded: BTreeMap<AccountId, u128>,
    pub start_block: u64, // used for data only, not in logic
    pub start_vote_block: u64, // block start voting, and end verify period
    pub end_vote_block: u64, // block ending voting
	}

  // #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	// pub struct DeactivatePropsParams<AccountId> {
  //   pub path: Vec<u8>,
	// 	pub subnet_nodes: Vec<SubnetNode<AccountId>>,
	// }

  #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct VotesParams {
    pub yay: u128,
		pub nay: u128,
    pub abstain: u128,
	}

  #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct ActivateVotesParams {
    pub yay: u128,
		pub nay: u128,
	}

  #[pallet::type_value]
	pub fn DefaultAccountId<T: Config>() -> T::AccountId {
		T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap()
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNode<T: Config>() -> SubnetNode<T::AccountId> {
		return SubnetNode {
			account_id: T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap(),
			peer_id: PeerId(Vec::new()),
    };
	}
	#[pallet::type_value]
	pub fn DefaultActivatePropsParams<T: Config>() -> ActivatePropsParams<T::AccountId> {
		return ActivatePropsParams {
			path: Vec::new(),
			subnet_nodes: Vec::new(),
      end_vote_block: 0,
    };
	}
  #[pallet::type_value]
	pub fn DefaultPropsParams<T: Config>() -> PropsParams<T::AccountId> {
		return PropsParams {
      proposer: T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap(),
      proposer_stake: 0,
      proposal_status: PropsStatus::None,
      proposal_type: PropsType::None,
			path: Vec::new(),
      subnet_data: RegistrationSubnetData::default(),
      subnet_nodes_verified: BTreeSet::new(),
      subnet_nodes_bonded: BTreeMap::new(),
			subnet_nodes: Vec::new(),
      start_block: 0,
      start_vote_block: 0,
      end_vote_block: 0,
    };
	}
  #[pallet::type_value]
	pub fn DefaultVotes<T: Config>() -> VotesParams {
		return VotesParams {
      yay: 0,
      nay: 0,
      abstain: 0,
    }
	}
  // #[pallet::type_value]
	// pub fn DefaultActivateVotes<T: Config>() -> ActivateVotesParams {
	// 	return ActivateVotesParams {
  //     yay: 0,
  //     nay: 0,
  //   }
	// }
  // #[pallet::type_value]
	// pub fn DefaultDeactivatePropsParams<T: Config>() -> DeactivatePropsParams<T::AccountId> {
	// 	return DeactivatePropsParams {
	// 		path: Vec::new(),
	// 		subnet_nodes: Vec::new(),
  //   };
	// }
  #[pallet::type_value]
	pub fn DefaultPropsStatus() -> PropsStatus {
		PropsStatus::None
	}
  #[pallet::type_value]
	pub fn DefaultQuorum() -> u128 {
    // 10,000 * 1e18
		// 10000000000000000000000

    // 100 * 1e18 for testing
    100000000000000000000
	}
  #[pallet::type_value]
	pub fn DefaultMajority() -> u128 {
		66
	}

  #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  enum VoteOutReason {
    // If subnet peers are performing manipulation for rewards
    SubnetEmissionsManipulation,
    // If the subnet is down
    SubnetDown,
    // If the subnet isn't open-sourced
    SubnetCloseSourced,
    // If subnet is broken
    SubnetBroken,
    // If the subnet doesn't have minimum required peers
    SubnetMinimumNodes,
    // If the subnet is outputting illicit or illegal data
    SubnetIllicit,
    // Other
    Other,
  }

  #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub enum VoteType {
    Yay,
    Nay,
    Abstain,
  }

  #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub enum PropsType {
    None,
    Activate,
    Deactivate,
  }

  impl Default for PropsType {
    fn default() -> Self {
      PropsType::None
    }
  }

  #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub enum PropsStatus {
    // Default status
    None,
    /// Voting in progress or not yet executed
    Active,
    /// Voting succeeded and executed
    Succeeded,
    /// Not enough votes within voting period accomplished
    Defeated,
    /// Proposer cancelled proposal
    Cancelled,
    /// Voting period passed, thus expiring proposal
    Expired,
  }

  impl Default for PropsStatus {
    fn default() -> Self {
      PropsStatus::None
    }
  }

  // #[pallet::storage]
  // pub type ModelTypes<T: Config> =
  //   StorageMap<_, Blake2_128Concat, PropIndex, PropsParams<T::AccountId>, ValueQuery, DefaultPropsParams<T>>;

	// #[pallet::storage]
	// #[pallet::getter(fn activate_props)]
	// pub type ActivateProps<T: Config> =
	// 	StorageMap<_, Blake2_128Concat, PropIndex, ActivatePropsParams<T::AccountId>, ValueQuery, DefaultActivatePropsParams<T>>;

  #[pallet::storage]
  #[pallet::getter(fn props)]
  pub type Proposals<T: Config> =
    StorageMap<_, Blake2_128Concat, PropIndex, PropsParams<T::AccountId>, ValueQuery, DefaultPropsParams<T>>;
  
  // Activation proposals that are active for voting and verified by commited subnet nodes
  // We only require that ``activate`` proposals are verified and stored into a BTreeSet
  // Deactivate proposals are only checked by its current count of deactivate proposals in ``DeactivateProposalsCount``
  #[pallet::storage]
  #[pallet::getter(fn active_props)]
  pub type ActiveActivateProposals<T: Config> = StorageValue<_, BTreeSet<PropIndex>, ValueQuery>;
  
  // Track active proposals to ensure that we don't increase past the max proposals
  // This includes the sum of all activate and deactivate proposals
  #[pallet::storage]
  #[pallet::getter(fn active_activate_proposals)]
	pub type ActiveProposalsCount<T> = StorageValue<_, u32, ValueQuery>;
  
  #[pallet::storage]
	pub type ActivateProposalsCount<T> = StorageValue<_, u32, ValueQuery>;

  #[pallet::storage]
  pub type DeactivateProposalsCount<T> = StorageValue<_, u32, ValueQuery>;

  #[pallet::storage]
  #[pallet::getter(fn votes)]
  pub type Votes<T: Config> =
    StorageMap<_, Blake2_128Concat, PropIndex, VotesParams, ValueQuery, DefaultVotes<T>>;
  
  #[pallet::storage]
  #[pallet::getter(fn votes_balance)]
  pub type VotesBalance<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    PropIndex,
    Identity,
    T::AccountId,
    BalanceOf<T>,
    ValueQuery,
  >;

  // #[pallet::storage]
  // pub type ActivateVotes<T: Config> =
  //   StorageMap<_, Blake2_128Concat, PropIndex, ActivateVotesParams, ValueQuery, DefaultActivateVotes<T>>;
  
  // #[pallet::storage]
	// #[pallet::getter(fn activate_prop_count)]
	// pub type ActivatePropCount<T> = StorageValue<_, PropIndex, ValueQuery>;

  #[pallet::storage]
	#[pallet::getter(fn prop_count)]
	pub type PropCount<T> = StorageValue<_, PropIndex, ValueQuery>;

  // #[pallet::storage]
	// #[pallet::getter(fn deactivate_props)]
	// pub type DeactivateProps<T: Config> =
	// 	StorageMap<_, Blake2_128Concat, PropIndex, ActivatePropsParams<T::AccountId>, ValueQuery, DefaultActivatePropsParams<T>>;

  // #[pallet::storage]
	// #[pallet::getter(fn deactivate_prop_count)]
	// pub type DeactivatePropCount<T> = StorageValue<_, PropIndex, ValueQuery>;

  #[pallet::storage]
	pub type PropsPathStatus<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, PropsStatus, ValueQuery, DefaultPropsStatus>;

  // #[pallet::storage]
  // #[pallet::getter(fn quorum)]
  // pub type Quorum<T> = StorageValue<_, u128, ValueQuery, DefaultQuorum>;
  
  #[pallet::storage]
  pub type Majority<T> = StorageValue<_, u128, ValueQuery>;

  #[pallet::storage]
  pub type NodeVotePremium<T> = StorageValue<_, u128, ValueQuery>;

  #[pallet::pallet]
  #[pallet::without_storage_info]
  pub struct Pallet<T>(_);

  #[pallet::call]
  impl<T: Config> Pallet<T> {

    /// Propose a new subnet to be initiated.
		///
		/// May only be call to activate a subnet if 
    ///  - The subnet doesn't already exist within the network pallet
    ///  - The subnet isn't already proposed to be activated via PropsStatus::Active
    ///  - The proposer doesn't have the funds to initiate the subnet
    ///  - The subnet_nodes entered are below or above the min and max requirements
    ///  - The subnet_nodes don't have the minimum required stake balance available
    ///
		/// May only be call to deactivate a subnet if 
    ///  - The subnet already does exist within the network pallet
    ///  - The subnet isn't already proposed to be deactivated via PropsStatus::Active
    ///
    /// The RegistrationSubnetData is used to dictate the subnets rewards and node requirements.
    /// Memory must be accurate to usage of the subnet for servers/
    /// Higher memory requirements will garner higher rewards but also require increased node requirements.
    /// Lower memory requirements will garner lower rewards but require lesser noes to operate the subnet.
    #[pallet::call_index(0)]
    #[pallet::weight(T::WeightInfo::propose())]
    // #[pallet::weight(0)]
    pub fn propose(
      origin: OriginFor<T>, 
      subnet_data: RegistrationSubnetData,
      mut subnet_nodes: Vec<SubnetNode<T::AccountId>>,
      proposal_type: PropsType
    ) -> DispatchResult {
      let account_id: T::AccountId = ensure_signed(origin)?;

      // --- Ensure proper proposal type
      ensure!(
				proposal_type != PropsType::None,
				Error::<T>::PropsTypeInvalid
			);

      // --- Ensure active proposals count don't exceed max proposals count
      ensure!(
				ActiveProposalsCount::<T>::get() <= T::MaxProposals::get(),
				Error::<T>::MaxActiveProposals
			);

      let proposal_index = PropCount::<T>::get();

      let mut proposer_stake: u128 = 0;
      let proposer_balance = T::Currency::free_balance(&account_id);

      if proposal_type == PropsType::Activate {
        // --- Stake the value of initializing a new subnet
        let subnet_initialization_cost = T::SubnetVote::get_subnet_initialization_cost();
        proposer_stake = subnet_initialization_cost;
        let subnet_initialization_cost_as_balance = Self::u128_to_balance(subnet_initialization_cost);
    
        ensure!(
          subnet_initialization_cost_as_balance.is_some(),
          Error::<T>::CouldNotConvertToBalance
        );
    
        ensure!(
          proposer_balance >= subnet_initialization_cost_as_balance.unwrap(),
          Error::<T>::NotEnoughSubnetInitializationBalance
        );
    
        // --- Reserve balance to be used once succeeded, otherwise it is freed on defeat
        // The final initialization fee may be more or less than the current initialization cost results
        T::Currency::reserve(
          &account_id,
          subnet_initialization_cost_as_balance.unwrap(),
        );

        // --- Proposal prelims
        Self::try_propose_activate(
          account_id.clone(), 
          subnet_data.clone(), 
          subnet_nodes.clone()
        ).map_err(|e| e)?;
      } else if proposal_type == PropsType::Deactivate {
        // --- Ensure zero subnet peers are submitted on deactivation proposals
        ensure!(
          subnet_nodes.clone().len() == 0,
          Error::<T>::SubnetNodesLengthInvalid
        );

        proposer_stake = T::MinProposerStake::get();
        let proposer_stake_as_balance = Self::u128_to_balance(proposer_stake);

        ensure!(
          proposer_balance >= proposer_stake_as_balance.unwrap(),
          Error::<T>::NotEnoughSubnetInitializationBalance
        );
    
        // --- Reserve balance to be used once succeeded, otherwise it is freed on defeat
        // This ultimately goes to the network pallet as the initialization fee or is returned to the proposer
        // and can be slashed if canceled.
        // The final initialization fee may be more or less than the current initialization cost results
        T::Currency::reserve(
          &account_id,
          proposer_stake_as_balance.unwrap(),
        );

        // --- Proposal prelims
        Self::try_propose_deactivate(account_id.clone(), subnet_data.clone().path)
          .map_err(|e| e)?;
      }

      // --- Save proposal
      Proposals::<T>::insert(
        proposal_index,
        PropsParams {
          proposer: account_id.clone(),
          proposer_stake: proposer_stake,
          proposal_status: PropsStatus::Active,
          proposal_type: proposal_type,
          path: subnet_data.clone().path,
          subnet_data: subnet_data.clone(),
          subnet_nodes: subnet_nodes.clone(),
          subnet_nodes_verified: BTreeSet::new(),
          subnet_nodes_bonded: BTreeMap::new(),
          start_block: Self::convert_block_as_u64(<frame_system::Pallet<T>>::block_number()),
          start_vote_block: Self::convert_block_as_u64(<frame_system::Pallet<T>>::block_number() + T::VerifyPeriod::get()),
          end_vote_block: Self::convert_block_as_u64(<frame_system::Pallet<T>>::block_number() + T::VerifyPeriod::get() + T::VotingPeriod::get()),
        },
      );
  
      // --- Set path to current proposal status to active
      PropsPathStatus::<T>::insert(subnet_data.path, PropsStatus::Active);

      // --- Increase proposals count
      PropCount::<T>::put(proposal_index + 1);

      // --- Increase active proposals count
      ActiveProposalsCount::<T>::mutate(|n: &mut u32| *n += 1);

      Ok(())
    }

    /// Anyone can activate a proposal and open it up for votes once:
    /// Activate: If all validator nodes for the subnet have bonded to validate their commitment
    /// Deactivate: Automatically activated and this doesn't need to be called, see ``is_proposal_active``
    #[pallet::call_index(1)]
    #[pallet::weight(0)]
    pub fn activate_proposal(
      origin: OriginFor<T>, 
      proposal_index: PropIndex,
    ) -> DispatchResult {
      let account_id: T::AccountId = ensure_signed(origin)?;
      ensure!(
        Proposals::<T>::contains_key(proposal_index),
        Error::<T>::ProposalInvalid
      );

      let proposal = Proposals::<T>::get(proposal_index);

      Self::try_activate_proposal(proposal_index, proposal);
      Ok(())
    }


    /// Vote on a proposal.
		///
		/// May only vote if
    ///  - Voter has enough balance
    ///
    /// Vote is based on balance and balance is staked until execution or defeat.
    #[pallet::call_index(2)]
    // #[pallet::weight(0)]
    #[pallet::weight(T::WeightInfo::cast_vote())]
    pub fn cast_vote(
      origin: OriginFor<T>, 
      proposal_index: PropIndex,
      vote_amount: BalanceOf<T>,
      vote: VoteType,
    ) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

      ensure!(
        Proposals::<T>::contains_key(proposal_index),
        Error::<T>::ProposalInvalid
      );

      let proposal = Proposals::<T>::get(proposal_index);
  
      Self::try_cast_vote(account_id, proposal_index, proposal, vote_amount, vote)
    }

    /// Execute completion of proposal 
    ///
    /// Voting must have completed
    ///
    /// Enactment period must not have completed
    ///
    /// Vote is based on balance and balance is staked until execution or defeat.
    ///
    /// Anyone can call this
    //
    // This cannot fail as long as the proposal ID exists, isn't concluded, and the voting period has completed
    #[pallet::call_index(3)]
    // #[pallet::weight(0)]
    #[pallet::weight(T::WeightInfo::execute())]
    pub fn execute(
      origin: OriginFor<T>, 
      proposal_index: PropIndex,
    ) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

      // --- Ensure proposal exists
      ensure!(
        Proposals::<T>::contains_key(proposal_index),
        Error::<T>::ProposalInvalid
      );
  
      let proposal = Proposals::<T>::get(proposal_index);

      // --- Ensure proposal is active and has not concluded
      ensure!(
        proposal.proposal_status == PropsStatus::Active,
        Error::<T>::Concluded
      );

      ensure!(
        !Self::is_voting_open(proposal.clone()),
        Error::<T>::VotingOpen
      );
  
      // --- Ensure voting has ended
      let end_vote_block = proposal.end_vote_block;
      let block = Self::get_current_block_as_u64();

      ensure!(
        block > end_vote_block,
        Error::<T>::VoteActive
      );

      // --- We made it past the voting period, we cannot fail from here

      if proposal.proposal_type == PropsType::Activate {
        ActivateProposalsCount::<T>::mutate(|n: &mut u32| n.saturating_dec());
      } else {
        DeactivateProposalsCount::<T>::mutate(|n: &mut u32| n.saturating_dec());
      }
      
      // --- If enactment period has passed, expire the proposal
      // Don't revert here to allow expired paths to be reproposed
      if block > end_vote_block + Self::convert_block_as_u64(T::EnactmentPeriod::get())  {
        Self::try_expire(proposal_index, proposal.path.clone())
          .map_err(|e| e)?;
        return Ok(())
      }

      // --- Get status of proposal
      let votes = Votes::<T>::get(proposal_index);

      let quorum_reached = Self::quorum_reached(votes.clone());
      let vote_succeeded = Self::vote_succeeded(votes.clone());

      let proposer_stake_as_balance = Self::u128_to_balance(proposal.proposer_stake);
      // --- Unreserve here to pay for initialization fee or give back to proposer
      T::Currency::unreserve(
        &proposal.proposer,
        proposer_stake_as_balance.unwrap(),
      );

      // --- Remove proposal from active proposals
      Self::try_deactivate_proposal(
        proposal_index,
        proposal.clone(),
      );
    
      // --- If quorum and vote YAYS aren greater than vote NAYS, then pass, else, defeat
      if quorum_reached && vote_succeeded {
        Self::try_succeed(account_id, proposal_index, proposal.clone())
          .map_err(|e| e)?;
      } else if quorum_reached && !vote_succeeded {
        Self::try_defeat(proposal_index, proposal.path.clone())
          .map_err(|e| e)?;
      } else {
        Self::try_expire(proposal_index, proposal.path.clone())
          .map_err(|e| e)?;
      }

      Ok(())
    }

    /// Cancel a proposal
    ///
    /// Can only be called by the proposer
    #[pallet::call_index(4)]
    #[pallet::weight(T::WeightInfo::cancel_proposal())]
    pub fn cancel_proposal(
      origin: OriginFor<T>, 
      proposal_index: PropIndex,
    ) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

      // --- Ensure proposal exists
      ensure!(
        Proposals::<T>::contains_key(proposal_index),
        Error::<T>::ProposalInvalid
      );
  
      let proposal = Proposals::<T>::get(proposal_index);

      ensure!(
        proposal.proposal_status == PropsStatus::Active,
        Error::<T>::Concluded
      );

      let block = Self::get_current_block_as_u64();

      // --- Ensure voting hasn't ended if not executed already
      // Can't cancel once the proposals voting period has surpassed
      let end_vote_block = proposal.end_vote_block;

      ensure!(
        block <= end_vote_block,
        Error::<T>::VoteComplete
      );

      let mut proposer_stake_unreserve_as_balance = Self::u128_to_balance(proposal.proposer_stake);

      if block > proposal.start_vote_block {
        // if verify period has ended
        // anyone can cancel if 100% verification has not been achieved
        //
        // If bootstrap subnet nodes did not bond, then we don't slash the proposer

        // --- Check if verified by subnet nodes
        // Deactivation proposals will always be verified, therefor deactivation proposals
        // can only be cancelled by the proposer if voting hasn't completed yet
        ensure!(
          !Self::is_verified(proposal.clone()),
          Error::<T>::ProposalVerified
        );

        // ensure!(
        //   !Self::is_bonded(proposal),
        //   Error::<T>::ProposalVerified
        // );
      } else {
        // if verify period has not ended yet
        // only the proposer can cancel, even if 100% verified
        ensure!(
          proposal.proposer == account_id,
          Error::<T>::NotProposer
        );

        // --- If proposer is cancelling, slash to disincentivize brute forcing proposals
        let proposer_stake_unreserve = proposal.proposer_stake - Percent::from_percent(T::CancelSlashPercent::get()) * proposal.proposer_stake;
        proposer_stake_unreserve_as_balance = Self::u128_to_balance(proposer_stake_unreserve);
      }

      // // --- If proposer is cancelling, slash to disincentivize brute forcing proposals
      // let proposer_stake_unreserve = proposal.proposer_stake - Percent::from_percent(T::CancelSlashPercent::get()) * proposal.proposer_stake;
      // let proposer_stake_unreserve_as_balance = Self::u128_to_balance(proposer_stake_unreserve);
      
      // --- Unreserve here to give back to proposer
      T::Currency::unreserve(
        &proposal.proposer,
        proposer_stake_unreserve_as_balance.unwrap(),
      );

      // --- Remove proposal from active proposals
      Self::try_deactivate_proposal(
        proposal_index,
        proposal.clone(),
      );
      
      Self::try_cancel(proposal_index, proposal.path)
    }

    /// NOTE: This function IS TO BE REMOVED
    /// Unreserve vote stake
    ///
    /// Proposal must be not Active
    #[pallet::call_index(5)]
    // #[pallet::weight(0)]
    #[pallet::weight(T::WeightInfo::unreserve())]
    pub fn unreserve(
      origin: OriginFor<T>, 
      proposal_index: PropIndex,
    ) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

      // --- Ensure proposal exists
      ensure!(
        Proposals::<T>::contains_key(proposal_index),
        Error::<T>::ProposalInvalid
      );

      // --- Ensure proposal not active
      let proposal = Proposals::<T>::get(proposal_index);

      // The only way a proposal can not be active or none is if it was executed already
      // Therefor, we do not check the block numbers
      // --- Ensure proposal is no longer active
      ensure!(
        proposal.proposal_status != PropsStatus::None && proposal.proposal_status != PropsStatus::Active,
        Error::<T>::ProposalInvalid
      );

      // --- Ensure account has vote balance on proposal ID
      ensure!(
        VotesBalance::<T>::contains_key(proposal_index, &account_id),
        Error::<T>::VotesBalanceInvalid
      );

      // --- Get balance and remove from storage
      let balance = VotesBalance::<T>::take(proposal_index, &account_id);

      // ensure!(
      //   Self::balance_to_u128(balance) > 0,
      //   Error::<T>::VoteBalanceZero
      // );

      // let reserved = T::Currency::reserved_balance(
      //   &account_id,
      // );

      // T::Currency::unreserve(
      //   &account_id,
      //   balance,
      // );
  
      Ok(())
    }
   
    /// Verify activation proposal as a subnet node
    // Each subnet node entered into an activation proposal must verify their inclusion in the proposals data
    #[pallet::call_index(6)]
    #[pallet::weight({0})]
    pub fn verify_proposal(
      origin: OriginFor<T>, 
      proposal_index: PropIndex,
    ) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

      // --- Ensure proposal exists
      ensure!(
        Proposals::<T>::contains_key(proposal_index),
        Error::<T>::ProposalInvalid
      );
      
      let proposal = Proposals::<T>::get(proposal_index);

      // --- Proposals only need verification on activation proposals
      ensure!(
        proposal.proposal_type == PropsType::Activate,
        Error::<T>::ProposalIsDeactvation
      );
      
      let block = Self::get_current_block_as_u64();

      // --- Ensure within verify period
      ensure!(
        block <= proposal.start_vote_block,
        Error::<T>::VerifyPeriodPassed
      );

      // Get account match
      let mut is_subnet_node = false;
      for subnet_node in proposal.subnet_nodes {
        if subnet_node.account_id == account_id {
          is_subnet_node = true;
          break;
        }
      }

      ensure!(
        is_subnet_node,
        Error::<T>::NotSubnetNode
      );

      // --- Ensure peers have the minimum required stake balance
      let min_stake: u128 = T::SubnetVote::get_min_stake_balance();
      let min_stake_as_balance = Self::u128_to_balance(min_stake);

      ensure!(
        min_stake_as_balance.is_some(),
        Error::<T>::CouldNotConvertToBalance
      );

      let peer_balance = T::Currency::free_balance(&account_id);

      ensure!(
        peer_balance >= min_stake_as_balance.unwrap(),
        Error::<T>::NotEnoughMinStakeBalance
      );

      let mut subnet_nodes_verified = proposal.subnet_nodes_verified;

      // Ensure not already verified
      ensure!(
        subnet_nodes_verified.insert(account_id.clone()),
        Error::<T>::SubnetNodeAlreadyVerified
      );

      Proposals::<T>::mutate(
        proposal_index,
        |params: &mut PropsParams<T::AccountId>| {
          params.subnet_nodes_verified = subnet_nodes_verified;
        },
      );
  
      Ok(())
    }

    #[pallet::call_index(7)]
    #[pallet::weight({0})]
    pub fn bond_proposal(
      origin: OriginFor<T>, 
      proposal_index: PropIndex,
    ) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

      // --- Ensure proposal exists
      ensure!(
        Proposals::<T>::contains_key(proposal_index),
        Error::<T>::ProposalInvalid
      );
      
      let proposal = Proposals::<T>::get(proposal_index);

      // --- Proposals only need verification on activation proposals
      ensure!(
        proposal.proposal_type == PropsType::Activate,
        Error::<T>::ProposalIsDeactvation
      );
      
      let block = Self::get_current_block_as_u64();

      // --- Ensure within verify period
      ensure!(
        block <= proposal.start_vote_block,
        Error::<T>::VerifyPeriodPassed
      );

      // Get account match
      let mut is_subnet_node = false;
      for subnet_node in proposal.subnet_nodes {
        if subnet_node.account_id == account_id {
          is_subnet_node = true;
          break;
        }
      }

      ensure!(
        is_subnet_node,
        Error::<T>::NotSubnetNode
      );

      // --- Ensure peers have the minimum required stake balance
      let min_stake: u128 = T::SubnetVote::get_min_stake_balance();
      let min_stake_as_balance = Self::u128_to_balance(min_stake);

      ensure!(
        min_stake_as_balance.is_some(),
        Error::<T>::CouldNotConvertToBalance
      );

      let peer_balance = T::Currency::free_balance(&account_id);

      ensure!(
        peer_balance >= min_stake_as_balance.unwrap(),
        Error::<T>::NotEnoughMinStakeBalance
      );

      // --- Reserve stake for the subnet node as bond
      T::Currency::reserve(
        &account_id,
        min_stake_as_balance.unwrap(),
      );

      let mut subnet_nodes_bonded = proposal.subnet_nodes_bonded;

      // Ensure not already bonded
      ensure!(
        subnet_nodes_bonded.insert(account_id.clone(), min_stake) == None,
        Error::<T>::SubnetNodeAlreadyBonded
      );

      Proposals::<T>::mutate(
        proposal_index,
        |params: &mut PropsParams<T::AccountId>| {
          params.subnet_nodes_bonded = subnet_nodes_bonded;
        },
      );
  
      Ok(())
    }

    #[pallet::call_index(8)]
    #[pallet::weight({0})]
    pub fn activate_subnet_node(
      origin: OriginFor<T>, 
      proposal_index: PropIndex,
    ) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

      // --- Ensure proposal exists
      ensure!(
        Proposals::<T>::contains_key(proposal_index),
        Error::<T>::ProposalInvalid
      );
      
      let proposal = Proposals::<T>::get(proposal_index);

      // --- Proposals only need verification on activation proposals
      ensure!(
        proposal.proposal_type == PropsType::Activate,
        Error::<T>::ProposalIsDeactvation
      );
      
      let block = Self::get_current_block_as_u64();

      // --- Ensure within verify period
      ensure!(
        block <= proposal.start_vote_block,
        Error::<T>::VerifyPeriodPassed
      );

      // Get account match
      let mut is_subnet_node = false;
      for subnet_node in proposal.subnet_nodes {
        if subnet_node.account_id == account_id {
          is_subnet_node = true;
          break;
        }
      }

      ensure!(
        is_subnet_node,
        Error::<T>::NotSubnetNode
      );

      // --- Ensure peers have the minimum required stake balance
      let min_stake: u128 = T::SubnetVote::get_min_stake_balance();
      let min_stake_as_balance = Self::u128_to_balance(min_stake);

      ensure!(
        min_stake_as_balance.is_some(),
        Error::<T>::CouldNotConvertToBalance
      );

      let peer_balance = T::Currency::free_balance(&account_id);

      ensure!(
        peer_balance >= min_stake_as_balance.unwrap(),
        Error::<T>::NotEnoughMinStakeBalance
      );

      // --- Reserve stake for the subnet node as bond
      T::Currency::reserve(
        &account_id,
        min_stake_as_balance.unwrap(),
      );

      let mut subnet_nodes_bonded = proposal.subnet_nodes_bonded;

      // Ensure not already bonded
      ensure!(
        subnet_nodes_bonded.insert(account_id.clone(), min_stake) == None,
        Error::<T>::SubnetNodeAlreadyBonded
      );

      Proposals::<T>::mutate(
        proposal_index,
        |params: &mut PropsParams<T::AccountId>| {
          params.subnet_nodes_bonded = subnet_nodes_bonded;
        },
      );
  
      Ok(())
    }
  }

  #[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn offchain_worker(block_number: BlockNumberFor<T>) {
    }
  }
}

// impl<T: Config + pallet::Config> Pallet<T> {
impl<T: Config> Pallet<T> {
  fn try_propose_activate(account_id: T::AccountId, subnet_data: RegistrationSubnetData, mut subnet_nodes: Vec<SubnetNode<T::AccountId>>) -> DispatchResult {
    // --- Ensure path doesn't already exist in Network or SubnetVoting
    // If it doesn't already exist, then it has either been not proposed or deactivated
    ensure!(
      !T::SubnetVote::get_subnet_path_exist(subnet_data.clone().path),
      Error::<T>::SubnetPathExists
    );

    // --- Ensure proposal on subnet path not already in progress
    let proposal_status = PropsPathStatus::<T>::get(subnet_data.clone().path);

    // --- Ensure not active
    // A proposal can only be active if the subnet is not already initialized into the blockchain
    ensure!(
      proposal_status != PropsStatus::Active,
      Error::<T>::ProposalInvalid
    );

    ensure!(
      ActivateProposalsCount::<T>::get() < T::MaxActivateProposals::get(),
      Error::<T>::MaxActivateProposals
    );

    ensure!(
      subnet_data.clone().memory_mb > 0,
      Error::<T>::SubnetMemoryIsZero
    );

    // Remove duplicates based on peer_id and account_id
    subnet_nodes.dedup_by(|a, b| a.peer_id == b.peer_id && a.account_id == b.account_id);

    // --- Ensure PeerId's are validated
    for node in subnet_nodes.iter() {
      let valid = Self::validate_peer_id(node.clone().peer_id);
      ensure!(valid, Error::<T>::InvalidPeerId);
    }

    // --- Ensure account has enough balance to pay cost of subnet initialization

    // --- Ensure minimum peers required are already met before going forward
    let subnet_nodes_len: u32 = subnet_nodes.len() as u32;

    // @to-do: Get minimum subnet peers from network pallet
    ensure!(
      subnet_nodes_len >= T::SubnetVote::get_min_subnet_nodes(subnet_data.clone().memory_mb) && 
      subnet_nodes_len <= T::SubnetVote::get_max_subnet_nodes(),
      Error::<T>::SubnetNodesLengthInvalid
    );

    // --- Ensure peers have the minimum required stake balance
    let min_stake: u128 = T::SubnetVote::get_min_stake_balance();
    let min_stake_as_balance = Self::u128_to_balance(min_stake);

    ensure!(
      min_stake_as_balance.is_some(),
      Error::<T>::CouldNotConvertToBalance
    );

    for peer in subnet_nodes.clone() {
      let peer_balance = T::Currency::free_balance(&peer.account_id);

      ensure!(
        peer_balance >= min_stake_as_balance.unwrap(),
        Error::<T>::NotEnoughMinStakeBalance
      );
    }

    ActivateProposalsCount::<T>::mutate(|n: &mut u32| *n += 1);

    // --- Begin to begin voting

    Ok(())
  }

  fn try_propose_deactivate(account_id: T::AccountId, path: Vec<u8>) -> DispatchResult {
    // --- Ensure subnet ID exists to be removed
    let subnet_id = T::SubnetVote::get_subnet_id_by_path(path.clone());
    ensure!(
      subnet_id != 0,
      Error::<T>::SubnetIdNotExists
    );

    // --- Ensure subnet has had enough time to initialize
    ensure!(
      T::SubnetVote::is_subnet_initialized(subnet_id),
      Error::<T>::ProposalInvalid
    );
    
    ensure!(
      DeactivateProposalsCount::<T>::get() < T::MaxDeactivateProposals::get(),
      Error::<T>::MaxDeactivateProposals
    );

    // --- Ensure proposal on subnet path not already in progress
    let proposal_status = PropsPathStatus::<T>::get(path.clone());

    ensure!(
      proposal_status != PropsStatus::Active,
      Error::<T>::ProposalInvalid
    );

    DeactivateProposalsCount::<T>::mutate(|n: &mut u32| *n += 1);
    
    Ok(())
  }

  fn try_activate_proposal(
    proposal_index: PropIndex,
    proposal: PropsParams<T::AccountId>,
  ) -> DispatchResult {
    ensure!(
      proposal.proposal_status == PropsStatus::Active,
      Error::<T>::Concluded
    );

    let mut active_activate_proposals: BTreeSet<PropIndex> = ActiveActivateProposals::<T>::get();
    ensure!(
      active_activate_proposals.len() < T::MaxProposals::get() as usize,
      Error::<T>::ProposalInvalid
    );

    ensure!(
      Self::is_verified(proposal),
      Error::<T>::ProposalNotVerified
    );

    // ensure!(
    //   Self::is_bonded(proposal),
    //   Error::<T>::ProposalNotVerified
    // );

    active_activate_proposals.insert(proposal_index);
    ActiveActivateProposals::<T>::put(active_activate_proposals);
    Ok(())
  }

  fn try_deactivate_proposal(
    proposal_index: PropIndex,
    proposal: PropsParams<T::AccountId>,
  ) -> DispatchResult {
    let mut active_activate_proposals: BTreeSet<PropIndex> = ActiveActivateProposals::<T>::get();
    active_activate_proposals.remove(&proposal_index);
    ActiveActivateProposals::<T>::put(active_activate_proposals);
    Ok(())
  }

  /// Cast vote on a proposal
  // Each account must be a staker
  // The ``vote_amount`` is not reserved
  fn try_cast_vote(
    account_id: T::AccountId, 
    proposal_index: PropIndex, 
    proposal: PropsParams<T::AccountId>,
    vote_amount: BalanceOf<T>,
    vote: VoteType,
  ) -> DispatchResult {
    ensure!(
      Self::is_proposal_active(proposal_index, proposal.clone()),
      Error::<T>::ProposalNotActive
    );

    // --- Ensure voting hasn't closed yet
    ensure!(
      Self::is_voting_open(proposal.clone()),
      Error::<T>::VotingNotOpen
    );

    // --- Get staked balance of voter
    let available_balance = Self::get_available_vote_balance(proposal_index, account_id.clone());

    // --- Ensure balance is some
    ensure!(
      available_balance > 0,
      Error::<T>::NotEnoughBalanceToVote
    );

    let vote_amount_as_u128: u128 = Self::balance_to_u128(vote_amount);

    ensure!(
      available_balance >= vote_amount_as_u128,
      Error::<T>::NotEnoughBalanceToVote
    );

    // --- Increase accounts reserved voting balance in relation to proposal index
    VotesBalance::<T>::mutate(proposal_index.clone(), account_id.clone(), |n| *n += vote_amount);

    // --- Save vote
    if vote == VoteType::Yay {
      Votes::<T>::mutate(
        proposal_index.clone(),
        |params: &mut VotesParams| {
          params.yay += vote_amount_as_u128;
        }
      );
    } else if vote == VoteType::Nay {
      Votes::<T>::mutate(
        proposal_index.clone(),
        |params: &mut VotesParams| {
          params.nay += vote_amount_as_u128;
        }
      );  
    } else {
      Votes::<T>::mutate(
        proposal_index.clone(),
        |params: &mut VotesParams| {
          params.abstain += vote_amount_as_u128;
        }
      );  
    }

    Ok(())
  }

  fn validate_peers(subnet_nodes: Vec<SubnetNode<T::AccountId>>) -> DispatchResult {
    Ok(())
  }

  fn get_voting_power(account_id: T::AccountId, balance: BalanceOf<T>) -> u128 {
    let is_submittable_subnet_node_account: bool = T::SubnetVote::is_submittable_subnet_node_account(account_id);

    if is_submittable_subnet_node_account {
      let peer_vote_premium = Perbill::from_rational(NodeVotePremium::<T>::get(), 100 as u128);
      let voting_power = balance.saturating_add(peer_vote_premium * balance);
   
      return Self::balance_to_u128(voting_power)
    }

    Self::balance_to_u128(balance)
  }

  fn try_succeed(
    activator: T::AccountId,
    proposal_index: PropIndex, 
    proposal: PropsParams<T::AccountId>, 
  ) -> DispatchResult {
    Proposals::<T>::mutate(
      proposal_index,
      |params: &mut PropsParams<T::AccountId>| {
        params.proposal_status = PropsStatus::Succeeded;
      },
    );

    PropsPathStatus::<T>::insert(proposal.clone().subnet_data.path, PropsStatus::Succeeded);

    ActiveProposalsCount::<T>::mutate(|n: &mut u32| n.saturating_dec());

    if proposal.proposal_type == PropsType::Activate {
      Self::try_activate_subnet(activator, proposal.clone().proposer, proposal.clone().subnet_data);
    } else {
      Self::try_deactivate_subnet(activator, proposal.clone().proposer, proposal.clone().subnet_data);
    }

    // --- Proposal stake unservered in the `execute`, update to reflect no reserves 
    Proposals::<T>::mutate(
      proposal_index,
      |params: &mut PropsParams<T::AccountId>| {
        params.proposer_stake = 0;
      },
    );
    Ok(())
  }

  fn try_defeat(proposal_index: PropIndex, path: Vec<u8>) -> DispatchResult {
    Proposals::<T>::mutate(
      proposal_index,
      |params: &mut PropsParams<T::AccountId>| {
        params.proposal_status = PropsStatus::Defeated;
      },
    );
  
    PropsPathStatus::<T>::insert(path.clone(), PropsStatus::Defeated);

    ActiveProposalsCount::<T>::mutate(|n: &mut u32| n.saturating_dec());

    Ok(())
  }
  
  fn try_cancel(proposal_index: PropIndex, path: Vec<u8>) -> DispatchResult {
    Proposals::<T>::mutate(
      proposal_index,
      |params: &mut PropsParams<T::AccountId>| {
        params.proposal_status = PropsStatus::Cancelled;
        params.proposer_stake = 0;
      },
    );

    PropsPathStatus::<T>::insert(path.clone(), PropsStatus::Cancelled);

    ActiveProposalsCount::<T>::mutate(|n: &mut u32| n.saturating_dec());

    Ok(())
  }

  fn try_expire(proposal_index: PropIndex, path: Vec<u8>) -> DispatchResult {
    Proposals::<T>::mutate(
      proposal_index,
      |params: &mut PropsParams<T::AccountId>| {
        params.proposal_status = PropsStatus::Expired;
      },
    );
  
    PropsPathStatus::<T>::insert(path.clone(), PropsStatus::Expired);

    ActiveProposalsCount::<T>::mutate(|n: &mut u32| n.saturating_dec());

    Ok(())
  }

  /// Is the proposal verified by subnet nodes entered in during activate proposals
  // If deactivate proposal, it will always return true since 0 == 0
  pub fn is_verified(proposal: PropsParams<T::AccountId>) -> bool {
    proposal.subnet_nodes.len() == proposal.subnet_nodes_verified.len()
  }

  pub fn is_bonded(proposal: PropsParams<T::AccountId>) -> bool {
    proposal.subnet_nodes.len() == proposal.subnet_nodes_bonded.len()
  }

  /// Is voting active and within voting period
  fn is_voting_open(proposal: PropsParams<T::AccountId>) -> bool {
    let block = Self::get_current_block_as_u64();
    let end_vote_block = proposal.end_vote_block;
    
    block <= end_vote_block && proposal.proposal_status == PropsStatus::Active
  }

  fn vote_succeeded(votes: VotesParams) -> bool {
    votes.yay > votes.nay
  }

  fn quorum_reached(votes: VotesParams) -> bool {
    // let quorum = Quorum::<T>::get();
    let total_quorum_votes = votes.yay + votes.abstain;
    total_quorum_votes >= T::Quorum::get()
  }

  /// Activate subnet - Someone must add_subnet once activated
  fn try_activate_subnet(activator: T::AccountId, proposer: T::AccountId, subnet_data: RegistrationSubnetData) -> DispatchResult {
    let vote_subnet_data = SubnetDemocracySubnetData {
      data: subnet_data.clone(),
      active: true,
    };

    T::SubnetVote::vote_activated(
      activator.clone(),
      subnet_data.clone().path, 
      proposer.clone(),
      vote_subnet_data.clone()
    )
  }

  fn try_deactivate_subnet(activator: T::AccountId, proposer: T::AccountId, subnet_data: RegistrationSubnetData) -> DispatchResult {
    let vote_subnet_data = SubnetDemocracySubnetData {
      data: subnet_data.clone(),
      active: false,
    };

    T::SubnetVote::vote_deactivated(
      activator.clone(),
      subnet_data.clone().path, 
      proposer.clone(),
      vote_subnet_data.clone()
    )

    // T::SubnetVote::vote_activated(subnet_data.clone().path, vote_subnet_data)
    // Ok(())
  }

  /// Get total voting power at the time of the proposal
  fn get_quorum() -> u128 {
    // T::Quorum::get()
    let network_voting_power: u128 = T::SubnetVote::get_voting_power();
    let blockchain_voting_power: u128 = 0;
    let total_voting_power = network_voting_power.saturating_add(blockchain_voting_power);
    total_voting_power - Percent::from_percent(T::QuorumVotingPowerPercentage::get()) * total_voting_power
  }

  /// Get the accounts overall stake across:
  /// - Blockchain stake
  /// - Blockchain delegate stake
  /// - Subnet stake
  /// - Subnet delegate stake
  fn get_stake_balance(account_id: T::AccountId) -> u128 {
    /// TODO: Add voting power based on validator level
    /// i.e. subnet nodes get 100%, blockchain nodes get 100%, subnet delegates get 50%, blockchain delegates get 50%
    let subnet_stake_balance: u128 = T::SubnetVote::get_stake_balance(account_id.clone());
    let subnet_delegate_stake_balance: u128 = T::SubnetVote::get_delegate_stake_balance(account_id.clone());
    let blockchain_stake_balance: u128 = 0;
    let blockchain_delegate_stake_balance: u128 = 0;
    subnet_stake_balance
      .saturating_add(subnet_delegate_stake_balance)
      .saturating_add(blockchain_stake_balance)
      .saturating_add(blockchain_delegate_stake_balance)
  }

  /// Get current vote balance for proposal ID
  fn get_vote_balance(proposal_index: PropIndex, account_id: T::AccountId) -> u128 {
    let votes_balance = VotesBalance::<T>::get(proposal_index, account_id);
    Self::balance_to_u128(votes_balance)
  }

  /// Get total available vote balance for a given proposal
  fn get_available_vote_balance(proposal_index: PropIndex, account_id: T::AccountId) -> u128 {
    let vote_balance = Self::get_vote_balance(proposal_index, account_id.clone());
    let stake_balance = Self::get_stake_balance(account_id.clone());
    // Redundant but check regardless
    if vote_balance >= stake_balance {
      return 0;
    }
    stake_balance - vote_balance
  }

  fn is_proposal_active(proposal_index: PropIndex, proposal: PropsParams<T::AccountId>) -> bool {
    // Deactivate proposals don't require validation from nodes so they are always active unless completed or cancelled
    if proposal.proposal_type == PropsType::Deactivate {
      if proposal.proposal_status == PropsStatus::Active {
        return true
      } else {
        return false
      }
    }
    let active_activate_proposals = ActiveActivateProposals::<T>::get();

    active_activate_proposals.get(&proposal_index) != None
  }
}

impl<T: Config> Pallet<T> {
  fn u128_to_balance(
    input: u128,
  ) -> Option<
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
  > {
    input.try_into().ok()
  }

  fn balance_to_u128(
    input: BalanceOf<T>,
  ) -> u128 {
    return match input.try_into() {
      Ok(_result) => _result,
      Err(_error) => 0,
    }
  }

  fn get_current_block_as_u64() -> u64 {
    TryInto::try_into(<frame_system::Pallet<T>>::block_number())
      .ok()
      .expect("blockchain will not exceed 2^64 blocks; QED.")
  }

  fn convert_block_as_u64(block: BlockNumberFor<T>) -> u64 {
    TryInto::try_into(block)
      .ok()
      .expect("blockchain will not exceed 2^64 blocks; QED.")
  }
}

// Admin logic
impl<T: Config> AdminInterface for Pallet<T> {
	fn set_peer_vote_premium(value: u128) -> DispatchResult {
		Self::set_peer_vote_premium(value)
	}
	fn set_quorum(value: u128) -> DispatchResult {
		Self::set_quorum(value)
	}
  fn set_majority(value: u128) -> DispatchResult {
		Self::set_majority(value)
	}
}

pub trait AdminInterface {
	fn set_peer_vote_premium(value: u128) -> DispatchResult;
  fn set_quorum(value: u128) -> DispatchResult;
  fn set_majority(value: u128) -> DispatchResult;
}