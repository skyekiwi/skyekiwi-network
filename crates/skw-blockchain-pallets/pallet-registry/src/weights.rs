// This file is part of SkyeKiwi Network.

// Copyright (C) 2021 - 2022 SkyeKiwi.
// SPDX-License-Identifier: GPL-3.0-or-later


//! Autogenerated weights for pallet_registry
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-24, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: ``, CPU: ``
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// /Users/songzhou/Desktop/skyekiwi-network/target/release/skyekiwi-node
// benchmark
// pallet
// --steps
// 50
// --repeat
// 20
// --pallet
// pallet_registry
// --extrinsic
// *
// --execution
// wasm
// --wasm-execution
// compiled
// --heap-pages
// 4096
// --output
// /Users/songzhou/Desktop/skyekiwi-network/crates/skw-blockchain-pallets/pallet-registry/src/weights.rs
// --template
// /Users/songzhou/Desktop/skyekiwi-network/misc/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_registry.
pub trait WeightInfo {
	fn register_secret_keeper() -> Weight;
	fn renew_registration() -> Weight;
	fn remove_registration() -> Weight;
	fn register_running_shard() -> Weight;
	fn register_user_public_key() -> Weight;
}

/// Weights for pallet_registry using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Registry Expiration (r:1 w:1)
	// Storage: Registry SecretKeepers (r:1 w:1)
	// Storage: Registry PublicKey (r:0 w:1)
	fn register_secret_keeper() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(18_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Registry Expiration (r:1 w:1)
	// Storage: Registry PublicKey (r:1 w:1)
	fn renew_registration() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(19_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Registry Expiration (r:1 w:1)
	// Storage: Registry PublicKey (r:1 w:1)
	// Storage: Registry SecretKeepers (r:1 w:1)
	fn remove_registration() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(23_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Registry Expiration (r:1 w:0)
	// Storage: Registry PublicKey (r:1 w:0)
	// Storage: Registry ShardMembers (r:1 w:1)
	// Storage: Registry BeaconCount (r:1 w:1)
	// Storage: Registry BeaconIndex (r:0 w:1)
	fn register_running_shard() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(17_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Registry UserPublicKey (r:0 w:1)
	fn register_user_public_key() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(2_000_000 as u64)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Registry Expiration (r:1 w:1)
	// Storage: Registry SecretKeepers (r:1 w:1)
	// Storage: Registry PublicKey (r:0 w:1)
	fn register_secret_keeper() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(18_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	// Storage: Registry Expiration (r:1 w:1)
	// Storage: Registry PublicKey (r:1 w:1)
	fn renew_registration() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(19_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
	// Storage: Registry Expiration (r:1 w:1)
	// Storage: Registry PublicKey (r:1 w:1)
	// Storage: Registry SecretKeepers (r:1 w:1)
	fn remove_registration() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(23_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	// Storage: Registry Expiration (r:1 w:0)
	// Storage: Registry PublicKey (r:1 w:0)
	// Storage: Registry ShardMembers (r:1 w:1)
	// Storage: Registry BeaconCount (r:1 w:1)
	// Storage: Registry BeaconIndex (r:0 w:1)
	fn register_running_shard() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(17_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	// Storage: Registry UserPublicKey (r:0 w:1)
	fn register_user_public_key() -> Weight {
		// Minimum execution time:  nanoseconds.
		Weight::from_ref_time(2_000_000 as u64)
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
}