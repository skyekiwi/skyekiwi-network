// This file is part of SkyeKiwi Network.

// Copyright (C) 2021 - 2022 SkyeKiwi.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Autogenerated weights for pallet_secrets
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-30, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// pallet_secrets
// --extrinsic
// *
// --execution
// wasm
// --wasm-execution
// compiled
// --heap-pages
// 4096
// --output
// /Users/songzhou/Desktop/skyekiwi-network/crates/skw-blockchain-pallets/pallet-secrets/src/weights.rs
// --template
// /Users/songzhou/Desktop/skyekiwi-network/misc/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_secrets.
pub trait WeightInfo {
	fn register_secret() -> Weight;
	fn nominate_member() -> Weight;
	fn remove_member() -> Weight;
	fn force_nominate_member() -> Weight;
	fn force_remove_member() -> Weight;
	fn force_change_owner() -> Weight;
	fn update_metadata() -> Weight;
	fn burn_secret() -> Weight;
}

/// Weights for pallet_secrets using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Secrets CurrentSecretId (r:1 w:1)
	// Storage: Secrets Metadata (r:0 w:1)
	// Storage: Secrets Owner (r:0 w:1)
	fn register_secret() -> Weight {
		(10_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: Secrets Owner (r:1 w:0)
	// Storage: Secrets Operator (r:0 w:1)
	fn nominate_member() -> Weight {
		(11_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Secrets Owner (r:1 w:0)
	// Storage: Secrets Operator (r:1 w:1)
	fn remove_member() -> Weight {
		(13_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Secrets Operator (r:0 w:1)
	fn force_nominate_member() -> Weight {
		(8_000_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Secrets Operator (r:1 w:1)
	fn force_remove_member() -> Weight {
		(10_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Secrets Owner (r:1 w:1)
	fn force_change_owner() -> Weight {
		(3_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Secrets Operator (r:1 w:0)
	// Storage: Secrets Owner (r:1 w:0)
	// Storage: Secrets Metadata (r:1 w:1)
	fn update_metadata() -> Weight {
		(14_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Secrets Owner (r:1 w:1)
	// Storage: Secrets Metadata (r:1 w:1)
	fn burn_secret() -> Weight {
		(15_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Secrets CurrentSecretId (r:1 w:1)
	// Storage: Secrets Metadata (r:0 w:1)
	// Storage: Secrets Owner (r:0 w:1)
	fn register_secret() -> Weight {
		(10_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(3 as Weight))
	}
	// Storage: Secrets Owner (r:1 w:0)
	// Storage: Secrets Operator (r:0 w:1)
	fn nominate_member() -> Weight {
		(11_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: Secrets Owner (r:1 w:0)
	// Storage: Secrets Operator (r:1 w:1)
	fn remove_member() -> Weight {
		(13_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: Secrets Operator (r:0 w:1)
	fn force_nominate_member() -> Weight {
		(8_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: Secrets Operator (r:1 w:1)
	fn force_remove_member() -> Weight {
		(10_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: Secrets Owner (r:1 w:1)
	fn force_change_owner() -> Weight {
		(3_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: Secrets Operator (r:1 w:0)
	// Storage: Secrets Owner (r:1 w:0)
	// Storage: Secrets Metadata (r:1 w:1)
	fn update_metadata() -> Weight {
		(14_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: Secrets Owner (r:1 w:1)
	// Storage: Secrets Metadata (r:1 w:1)
	fn burn_secret() -> Weight {
		(15_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
}
