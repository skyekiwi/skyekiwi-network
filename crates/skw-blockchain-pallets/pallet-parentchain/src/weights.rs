// This file is part of SkyeKiwi Network.

// Copyright (C) 2021 - 2022 SkyeKiwi.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Autogenerated weights for pallet_parentchain
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
// pallet_parentchain
// --extrinsic
// *
// --execution
// wasm
// --wasm-execution
// compiled
// --heap-pages
// 4096
// --output
// /Users/songzhou/Desktop/skyekiwi-network/crates/skw-blockchain-pallets/pallet-parentchain/src/weights.rs
// --template
// /Users/songzhou/Desktop/skyekiwi-network/misc/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_parentchain.
pub trait WeightInfo {
	fn set_shard_confirmation_threshold() -> Weight;
	fn submit_outcome(s: u32, ) -> Weight;
}

/// Weights for pallet_parentchain using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Parentchain ShardConfirmationThreshold (r:1 w:1)
	fn set_shard_confirmation_threshold() -> Weight {
		(2_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Registry Expiration (r:1 w:0)
	// Storage: Registry PublicKey (r:1 w:0)
	// Storage: Parentchain ShardConfirmationThreshold (r:1 w:0)
	// Storage: Registry BeaconIndex (r:1 w:0)
	// Storage: Registry BeaconCount (r:1 w:0)
	// Storage: Parentchain StateRoot (r:1 w:1)
	// Storage: Parentchain Confirmation (r:1 w:1)
	// Storage: Parentchain Outcome (r:0 w:1)
	fn submit_outcome(s: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 44_000
			.saturating_add((26_624_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Parentchain ShardConfirmationThreshold (r:1 w:1)
	fn set_shard_confirmation_threshold() -> Weight {
		(2_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: Registry Expiration (r:1 w:0)
	// Storage: Registry PublicKey (r:1 w:0)
	// Storage: Parentchain ShardConfirmationThreshold (r:1 w:0)
	// Storage: Registry BeaconIndex (r:1 w:0)
	// Storage: Registry BeaconCount (r:1 w:0)
	// Storage: Parentchain StateRoot (r:1 w:1)
	// Storage: Parentchain Confirmation (r:1 w:1)
	// Storage: Parentchain Outcome (r:0 w:1)
	fn submit_outcome(s: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 44_000
			.saturating_add((26_624_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(RocksDbWeight::get().reads(7 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
	}
}
