
//! Autogenerated weights for `super`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-09-22, STEPS: `8`, REPEAT: 8, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/creditcoin-node
// benchmark
// --chain
// dev
// --steps=8
// --repeat=8
// --pallet
// super
// --extrinsic=*
// --execution
// wasm
// --wasm-execution=compiled
// --heap-pages=10000
// --output
// ./pallets/creditcoin/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `super`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> super::WeightInfo for WeightInfo<T> {
	// Storage: Creditcoin DealOrders (r:511 w:510)
	// Storage: Creditcoin BidOrders (r:0 w:255)
	// Storage: Creditcoin Offers (r:0 w:255)
	// Storage: Creditcoin PendingTasks (r:0 w:510)
	// Storage: Creditcoin AskOrders (r:0 w:31)
	fn on_initialize(a: u32, b: u32, o: u32, d: u32, f: u32, u: u32, c: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 8_017_000
			.saturating_add((16_337_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 8_017_000
			.saturating_add((16_341_000 as Weight).saturating_mul(b as Weight))
			// Standard Error: 8_017_000
			.saturating_add((15_617_000 as Weight).saturating_mul(o as Weight))
			// Standard Error: 8_017_000
			.saturating_add((28_213_000 as Weight).saturating_mul(d as Weight))
			// Standard Error: 8_017_000
			.saturating_add((31_968_000 as Weight).saturating_mul(f as Weight))
			// Standard Error: 8_017_000
			.saturating_add((15_264_000 as Weight).saturating_mul(u as Weight))
			// Standard Error: 8_017_000
			.saturating_add((11_061_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(d as Weight)))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(f as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(o as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(d as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(f as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(u as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
	}
	// Storage: Creditcoin Addresses (r:1 w:1)
	fn register_address() -> Weight {
		(274_501_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Creditcoin LegacyWallets (r:1 w:1)
	// Storage: Creditcoin LegacyBalanceKeeper (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	fn claim_legacy_wallet() -> Weight {
		(72_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Creditcoin AskOrders (r:1 w:1)
	// Storage: Creditcoin Addresses (r:1 w:0)
	// Storage: Creditcoin UsedGuids (r:1 w:1)
	fn add_ask_order() -> Weight {
		(41_900_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Creditcoin BidOrders (r:1 w:1)
	// Storage: Creditcoin Addresses (r:1 w:0)
	// Storage: Creditcoin UsedGuids (r:1 w:1)
	fn add_bid_order() -> Weight {
		(42_800_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Creditcoin AskOrders (r:1 w:0)
	// Storage: Creditcoin BidOrders (r:1 w:0)
	// Storage: Creditcoin Offers (r:1 w:1)
	fn add_offer() -> Weight {
		(44_700_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Creditcoin DealOrders (r:1 w:1)
	// Storage: Creditcoin Offers (r:1 w:0)
	// Storage: Creditcoin AskOrders (r:1 w:0)
	// Storage: Creditcoin BidOrders (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	fn add_deal_order() -> Weight {
		(54_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Creditcoin Authorities (r:1 w:1)
	fn add_authority() -> Weight {
		(7_200_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Creditcoin Authorities (r:1 w:0)
	// Storage: Creditcoin Transfers (r:1 w:1)
	// Storage: Creditcoin PendingTasks (r:0 w:1)
	fn persist_transfer() -> Weight {
		(41_901_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Creditcoin Authorities (r:1 w:0)
	// Storage: Creditcoin Transfers (r:1 w:0)
	// Storage: Creditcoin PendingTasks (r:0 w:1)
	fn fail_transfer() -> Weight {
		(32_501_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Creditcoin DealOrders (r:1 w:1)
	// Storage: Creditcoin Addresses (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Creditcoin Transfers (r:1 w:1)
	fn fund_deal_order() -> Weight {
		(58_500_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Creditcoin DealOrders (r:1 w:1)
	fn lock_deal_order() -> Weight {
		(31_600_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Creditcoin DealOrders (r:1 w:0)
	// Storage: Creditcoin Addresses (r:2 w:0)
	// Storage: Creditcoin Transfers (r:1 w:0)
	// Storage: Creditcoin PendingTasks (r:0 w:1)
	fn register_funding_transfer() -> Weight {
		(53_301_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Creditcoin DealOrders (r:1 w:0)
	// Storage: Creditcoin Addresses (r:2 w:0)
	// Storage: Creditcoin Transfers (r:1 w:0)
	// Storage: Creditcoin PendingTasks (r:0 w:1)
	fn register_repayment_transfer() -> Weight {
		(53_101_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Creditcoin DealOrders (r:1 w:1)
	// Storage: Creditcoin Addresses (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Creditcoin Transfers (r:1 w:1)
	fn close_deal_order() -> Weight {
		(65_900_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Creditcoin DealOrders (r:1 w:1)
	// Storage: Creditcoin Addresses (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	fn exempt() -> Weight {
		(41_600_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Creditcoin Addresses (r:2 w:0)
	// Storage: Creditcoin AskOrders (r:1 w:1)
	// Storage: Creditcoin BidOrders (r:1 w:1)
	// Storage: Creditcoin Offers (r:1 w:1)
	// Storage: Creditcoin DealOrders (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	fn register_deal_order() -> Weight {
		(329_201_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: Creditcoin CollectCoinsContract (r:1 w:0)
	// Storage: Creditcoin CollectedCoins (r:1 w:0)
	// Storage: Creditcoin PendingTasks (r:1 w:1)
	// Storage: Creditcoin Addresses (r:1 w:0)
	fn request_collect_coins() -> Weight {
		(42_300_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Creditcoin Authorities (r:1 w:0)
	// Storage: Creditcoin CollectedCoins (r:1 w:0)
	// Storage: Creditcoin PendingTasks (r:0 w:1)
	fn fail_collect_coins() -> Weight {
		(23_700_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Creditcoin Authorities (r:1 w:0)
	// Storage: Creditcoin CollectedCoins (r:1 w:1)
	// Storage: Creditcoin Addresses (r:1 w:0)
	// Storage: Creditcoin PendingTasks (r:0 w:1)
	fn persist_collect_coins() -> Weight {
		(83_701_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Creditcoin Authorities (r:1 w:1)
	fn remove_authority() -> Weight {
		(8_200_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Creditcoin Currencies (r:1 w:1)
	fn register_currency() -> Weight {
		(7_600_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Creditcoin CollectCoinsContract (r:0 w:1)
	fn set_collect_coins_contract() -> Weight {
		(4_000_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
