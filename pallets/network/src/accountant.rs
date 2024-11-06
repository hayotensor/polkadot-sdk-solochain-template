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

impl<T: Config> Pallet<T> {
  pub fn do_submit_accountant_data(
    accountant: T::AccountId,
    subnet_id: u32,
    epoch: u32,
    data: Vec<AccountantDataNodeParams>,
  ) -> DispatchResult {
    // --- Ensure is epochs accountant

    // New accountants are chosen at the beginning of each epoch, if the previous accountant doesn't submit 
    // data by the end of the epoch, then they will get errors when the new accountants are chosen. New accountants
    // cannot be the last accountants

    // --- Ensure is epochs accountant
    let mut current_accountants = match CurrentAccountants::<T>::try_get(subnet_id, epoch) {
      Ok(accountants) => accountants,
      Err(()) =>
        return Err(Error::<T>::InvalidSubnetRewardsSubmission.into()),
    };

    ensure!(
      current_accountants.contains_key(&accountant.clone()),
      Error::<T>::NotAccountant
    );

    // Check if removed all stake yet
    let has_submitted: bool = match current_accountants.get(&accountant.clone()) {
      Some(submitted) => *submitted,
      None => false,
    };
    ensure!(
      !has_submitted,
      Error::<T>::NotAccountant
    );

    let data_len = data.len();
    let total_subnet_nodes: u32 = TotalSubnetNodes::<T>::get(subnet_id);

    // --- Ensure length of data does not exceed total subnet peers of subnet ID
    ensure!(
      data_len as u32 <= total_subnet_nodes && data_len as u32 > 0,
      Error::<T>::InvalidAccountantData
    );

    // --- Update to data submitted
    current_accountants.insert(accountant.clone(), true);
    CurrentAccountants::<T>::insert(subnet_id, epoch, current_accountants);
    
    let accountant_data_index: u32 = AccountantDataCount::<T>::get(subnet_id);

    let block: u64 = Self::get_current_block_as_u64();

    AccountantData::<T>::insert(
      subnet_id,
      accountant_data_index.clone(),
      AccountantDataParams {
        accountant,
        block,
        epoch,
        data,
      }
    );

    Ok(())
  }

  pub fn choose_accountants(
    block: u64,
    epoch: u32,
    subnet_id: u32,
    min_subnet_nodes: u32,
    target_accountants_len: u32,
  ) {
    let node_sets: BTreeMap<T::AccountId, u64> = SubnetNodesClasses::<T>::get(subnet_id, SubnetNodeClass::Accountant);
    let node_sets_len: u32 = node_sets.len() as u32;
    // --- Ensure min subnet peers that are submittable are at least the minimum required
    // --- Consensus cannot begin until this minimum is reached
    // --- If not min subnet peers count then accountant isn't needed
    if node_sets_len < min_subnet_nodes {
      return
    }

    let account_ids: Vec<T::AccountId> = node_sets.iter()
      .map(|x| x.0.clone())
      .collect();

    // --- Ensure we don't attempt to choose more accountants than are available
    let mut max_accountants: u32 = target_accountants_len;
    if node_sets_len < max_accountants {
      max_accountants = node_sets_len;
    }

    // `-1` for overflow
    let account_ids_len = account_ids.len() - 1;

    // --- Ensure no duplicates
    // let mut unique_accountants: Vec<T::AccountId> = Vec::new();
    let mut chosen_accountants_complete: bool = false;

    let mut current_accountants: BTreeMap<T::AccountId, bool> = BTreeMap::new();

    // --- Get random number 0 - MAX
    // Because true randomization isn't as important here, we only get one random number
    // and choose the other accountants as `n+1 % MAX` to limit computation
    // We use block + 1 in order to differentiate between validators to prevent the chosen
    // validator being one of the accountants. 
    let rand_index = Self::get_random_number(account_ids_len as u32, (block + 1) as u32);

    for n in 0..max_accountants {
      let rand = rand_index + n % account_ids_len as u32;
      let random_accountant: &T::AccountId = &account_ids[rand as usize];

      current_accountants.insert(random_accountant.clone(), false);
    }

    CurrentAccountants::<T>::insert(subnet_id, epoch, current_accountants);
  }
}