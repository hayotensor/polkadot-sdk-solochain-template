use super::*;

impl<T: Config> Pallet<T> {
  pub fn get_subnet_nodes(
    subnet_id: u32,
  ) -> Vec<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let block: u64 = Self::get_current_block_as_u64();
    let epoch_length: u64 = T::EpochLength::get();
    let epoch: u64 = block / epoch_length;
    Self::get_classified_subnet_nodes(subnet_id, &ClassTest::Idle, epoch)
  }

  pub fn get_subnet_nodes_included(
    subnet_id: u32,
  ) -> Vec<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let block: u64 = Self::get_current_block_as_u64();
    let epoch_length: u64 = T::EpochLength::get();
    let epoch: u64 = block / epoch_length;
    Self::get_classified_subnet_nodes(subnet_id, &ClassTest::Included, epoch)
  }

  // pub fn get_subnet_nodes_submittable(
  //   subnet_id: u32,
  // ) -> Vec<SubnetNode<T::AccountId>> {
  //   if !SubnetsData::<T>::contains_key(subnet_id) {
  //     return Vec::new();
  //   }

  //   // let node_sets: BTreeMap<T::AccountId, u64> = SubnetNodesClasses::<T>::get(subnet_id, SubnetNodeClass::Submittable);

  //   let subnet_nodes: Vec<T::AccountId> = SubnetNodesClasses::<T>::get(subnet_id, SubnetNodeClass::Submittable).iter()
  //     .map(|x| { 
  //       *x.0
  //      } )
  //     .collect();

  //   subnet_nodes
  // }

  pub fn get_subnet_nodes_submittable(
    subnet_id: u32,
  ) -> Vec<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let block: u64 = Self::get_current_block_as_u64();
    let epoch_length: u64 = T::EpochLength::get();
    let epoch: u64 = block / epoch_length;
    Self::get_classified_subnet_nodes(subnet_id, &ClassTest::Submittable, epoch)
  }

  pub fn get_subnet_nodes_subnet_unconfirmed_count(
    subnet_id: u32,
  ) -> u32 {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return 0;
    }

    0
  }

  // id is consensus ID
  pub fn get_consensus_data(
    subnet_id: u32,
    epoch: u32
  ) -> Option<RewardsData<T::AccountId>> {
    let data = SubnetRewardsSubmission::<T>::get(subnet_id, epoch);
    Some(data?)
  }

  // id is proposal ID
  pub fn get_accountant_data(
    subnet_id: u32,
    id: u32
  ) -> Option<AccountantDataParams<T::AccountId>> {
    let data = AccountantData::<T>::get(subnet_id, id);
    Some(data)
  }

  pub fn get_minimum_subnet_nodes(subnet_id: u32, memory_mb: u128) -> u32 {
    Self::get_min_subnet_nodes(BaseSubnetNodeMemoryMB::<T>::get(), memory_mb)
  }

  pub fn get_subnet_node_stake_by_peer_id(subnet_id: u32, peer_id: PeerId) -> u128 {
    match SubnetNodeAccount::<T>::try_get(subnet_id, peer_id.clone()) {
      Ok(account_id) => {
        AccountSubnetStake::<T>::get(account_id, subnet_id)
      },
      Err(()) => 0,
    }
  }

  pub fn is_subnet_node_by_peer_id(subnet_id: u32, peer_id: PeerId) -> bool {
    match SubnetNodeAccount::<T>::try_get(subnet_id, peer_id.clone()) {
      Ok(account_id) => true,
      Err(()) => false,
    }
  }
}