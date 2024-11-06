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

// https://blog.oak.tech/tutorial-benchmarking-for-parity-substrate-pallet-development-9cb68bf87ea2
// https://github.com/paritytech/substrate/blob/master/.maintain/frame-weight-template.hbs
// Executed Command:
// ./target/release/node-template benchmark pallet --chain=dev --wasm-execution=compiled --pallet=pallet_admin --extrinsic=* --steps=1 --repeat=1 --output="pallets/admin/src/weights.rs" --template ./.maintain/frame-weight-template.hbs

// cargo build --release --features runtime-benchmarks
// cargo test --release --features runtime-benchmarks
// Build only admin pallet
// cargo build --package pallet-admin --features runtime-benchmarks
use super::*;
// use crate::mock::*;
use frame_benchmarking::{account, benchmarks, whitelist_account, BenchmarkError};
use frame_support::{
	assert_noop, assert_ok,
	traits::Currency,
};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use crate::Pallet as Admin;
use scale_info::prelude::vec;

const SEED: u32 = 0;

benchmarks! {
  set_vote_subnet_in {
		let model_path: Vec<u8> = "petals-team-3/StableBeluga2".into();
	}: set_vote_subnet_in(RawOrigin::Root, model_path.clone(), 50000)
	verify {
		assert_eq!(Some(true), Some(true));
	}

	impl_benchmark_test_suite!(
		Admin,
		crate::mock::new_test_ext(),
		crate::mock::Test
	);
}