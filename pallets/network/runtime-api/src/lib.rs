// This file is part of Hypertensor.

// Copyright (C) 2023 Parity Technologies (UK) Ltd.
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

//! Runtime API definition for the network pallet.

#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
  pub trait NetworkRuntimeApi {
    fn get_subnet_nodes(subnet_id: u32) -> Vec<u8>;
    fn get_subnet_nodes_included(subnet_id: u32) -> Vec<u8>;
    fn get_subnet_nodes_submittable(subnet_id: u32) -> Vec<u8>;
    fn get_subnet_nodes_subnet_unconfirmed_count(subnet_id: u32) -> u32;
    fn get_consensus_data(subnet_id: u32, epoch: u32) -> Vec<u8>;
    fn get_accountant_data(subnet_id: u32, id: u32) -> Vec<u8>;
    fn get_minimum_subnet_nodes(memory_mb: u128) -> u32;
    fn get_minimum_delegate_stake(memory_mb: u128) -> u128;
  }
}