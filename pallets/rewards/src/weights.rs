
//! Autogenerated weights for `crate`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-07, STEPS: `50`, REPEAT: `30`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `github-runner-5203088300-attempt-1`, CPU: `AMD EPYC 7452 32-Core Processor`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/creditcoin-node
// benchmark
// pallet
// --chain
// dev
// --steps=50
// --repeat=30
// --pallet
// crate
// --extrinsic=*
// --execution
// wasm
// --wasm-execution=compiled
// --heap-pages=10000
// --output
// ./pallets/rewards/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `crate`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> crate::WeightInfo for WeightInfo<T> {
	/// Storage: PosSwitch SwitchBlockNumber (r:1 w:0)
	/// Proof: PosSwitch SwitchBlockNumber (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	fn on_initialize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `165`
		//  Estimated: `1517`
		// Minimum execution time: 7_100_000 picoseconds.
		Weight::from_parts(7_300_000, 0)
			.saturating_add(Weight::from_parts(0, 1517))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: PosSwitch SwitchBlockNumber (r:1 w:0)
	/// Proof: PosSwitch SwitchBlockNumber (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	fn on_finalize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `165`
		//  Estimated: `1517`
		// Minimum execution time: 7_000_000 picoseconds.
		Weight::from_parts(7_400_000, 0)
			.saturating_add(Weight::from_parts(0, 1517))
			.saturating_add(T::DbWeight::get().reads(1))
	}
}
