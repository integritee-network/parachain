
//! Autogenerated weights for `pallet_child_bounties`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 47.2.0
//! DATE: 2025-06-12, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `caribe`, CPU: `12th Gen Intel(R) Core(TM) i7-1260P`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("integritee-rococo-local-dev")`, DB CACHE: 1024

// Executed Command:
// ./target/release/integritee-collator
// benchmark
// pallet
// --chain=integritee-rococo-local-dev
// --steps=50
// --repeat=20
// --pallet=pallet_child_bounties
// --extrinsic=*
// --heap-pages=4096
// --output=./polkadot-parachains/integritee-kusama/src/weights/pallet_child_bounties.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_child_bounties`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_child_bounties::WeightInfo for WeightInfo<T> {
	/// Storage: `ChildBounties::ParentChildBounties` (r:1 w:1)
	/// Proof: `ChildBounties::ParentChildBounties` (`max_values`: None, `max_size`: Some(16), added: 2491, mode: `MaxEncodedLen`)
	/// Storage: `Bounties::Bounties` (r:1 w:0)
	/// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ParentTotalChildBounties` (r:1 w:1)
	/// Proof: `ChildBounties::ParentTotalChildBounties` (`max_values`: None, `max_size`: Some(16), added: 2491, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ChildBountyDescriptionsV1` (r:0 w:1)
	/// Proof: `ChildBounties::ChildBountyDescriptionsV1` (`max_values`: None, `max_size`: Some(16412), added: 18887, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ChildBounties` (r:0 w:1)
	/// Proof: `ChildBounties::ChildBounties` (`max_values`: None, `max_size`: Some(145), added: 2620, mode: `MaxEncodedLen`)
	/// The range of component `d` is `[0, 16384]`.
	fn add_child_bounty(_d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `511`
		//  Estimated: `6196`
		// Minimum execution time: 86_799_000 picoseconds.
		Weight::from_parts(109_352_194, 0)
			.saturating_add(Weight::from_parts(0, 6196))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	/// Storage: `Bounties::Bounties` (r:1 w:0)
	/// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ChildBounties` (r:1 w:1)
	/// Proof: `ChildBounties::ChildBounties` (`max_values`: None, `max_size`: Some(145), added: 2620, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ChildrenCuratorFees` (r:1 w:1)
	/// Proof: `ChildBounties::ChildrenCuratorFees` (`max_values`: None, `max_size`: Some(28), added: 2503, mode: `MaxEncodedLen`)
	fn propose_curator() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `575`
		//  Estimated: `3642`
		// Minimum execution time: 22_718_000 picoseconds.
		Weight::from_parts(23_914_000, 0)
			.saturating_add(Weight::from_parts(0, 3642))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Bounties::Bounties` (r:1 w:0)
	/// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ChildBounties` (r:1 w:1)
	/// Proof: `ChildBounties::ChildBounties` (`max_values`: None, `max_size`: Some(145), added: 2620, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn accept_curator() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `744`
		//  Estimated: `3642`
		// Minimum execution time: 39_787_000 picoseconds.
		Weight::from_parts(41_066_000, 0)
			.saturating_add(Weight::from_parts(0, 3642))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `ChildBounties::ChildBounties` (r:1 w:1)
	/// Proof: `ChildBounties::ChildBounties` (`max_values`: None, `max_size`: Some(145), added: 2620, mode: `MaxEncodedLen`)
	/// Storage: `Bounties::Bounties` (r:1 w:0)
	/// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn unassign_curator() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `744`
		//  Estimated: `3642`
		// Minimum execution time: 43_134_000 picoseconds.
		Weight::from_parts(44_513_000, 0)
			.saturating_add(Weight::from_parts(0, 3642))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Bounties::Bounties` (r:1 w:0)
	/// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ChildBounties` (r:1 w:1)
	/// Proof: `ChildBounties::ChildBounties` (`max_values`: None, `max_size`: Some(145), added: 2620, mode: `MaxEncodedLen`)
	fn award_child_bounty() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `641`
		//  Estimated: `3642`
		// Minimum execution time: 24_652_000 picoseconds.
		Weight::from_parts(25_460_000, 0)
			.saturating_add(Weight::from_parts(0, 3642))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `ChildBounties::ChildBounties` (r:1 w:1)
	/// Proof: `ChildBounties::ChildBounties` (`max_values`: None, `max_size`: Some(145), added: 2620, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:3 w:3)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ParentChildBounties` (r:1 w:1)
	/// Proof: `ChildBounties::ParentChildBounties` (`max_values`: None, `max_size`: Some(16), added: 2491, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ChildBountyDescriptionsV1` (r:0 w:1)
	/// Proof: `ChildBounties::ChildBountyDescriptionsV1` (`max_values`: None, `max_size`: Some(16412), added: 18887, mode: `MaxEncodedLen`)
	fn claim_child_bounty() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `595`
		//  Estimated: `8799`
		// Minimum execution time: 127_540_000 picoseconds.
		Weight::from_parts(130_806_000, 0)
			.saturating_add(Weight::from_parts(0, 8799))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	/// Storage: `Bounties::Bounties` (r:1 w:0)
	/// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ChildBounties` (r:1 w:1)
	/// Proof: `ChildBounties::ChildBounties` (`max_values`: None, `max_size`: Some(145), added: 2620, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ChildrenCuratorFees` (r:1 w:1)
	/// Proof: `ChildBounties::ChildrenCuratorFees` (`max_values`: None, `max_size`: Some(28), added: 2503, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ParentChildBounties` (r:1 w:1)
	/// Proof: `ChildBounties::ParentChildBounties` (`max_values`: None, `max_size`: Some(16), added: 2491, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ChildBountyDescriptionsV1` (r:0 w:1)
	/// Proof: `ChildBounties::ChildBountyDescriptionsV1` (`max_values`: None, `max_size`: Some(16412), added: 18887, mode: `MaxEncodedLen`)
	fn close_child_bounty_added() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `821`
		//  Estimated: `6196`
		// Minimum execution time: 90_044_000 picoseconds.
		Weight::from_parts(92_637_000, 0)
			.saturating_add(Weight::from_parts(0, 6196))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	/// Storage: `Bounties::Bounties` (r:1 w:0)
	/// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ChildBounties` (r:1 w:1)
	/// Proof: `ChildBounties::ChildBounties` (`max_values`: None, `max_size`: Some(145), added: 2620, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:3 w:3)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ChildrenCuratorFees` (r:1 w:1)
	/// Proof: `ChildBounties::ChildrenCuratorFees` (`max_values`: None, `max_size`: Some(28), added: 2503, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ParentChildBounties` (r:1 w:1)
	/// Proof: `ChildBounties::ParentChildBounties` (`max_values`: None, `max_size`: Some(16), added: 2491, mode: `MaxEncodedLen`)
	/// Storage: `ChildBounties::ChildBountyDescriptionsV1` (r:0 w:1)
	/// Proof: `ChildBounties::ChildBountyDescriptionsV1` (`max_values`: None, `max_size`: Some(16412), added: 18887, mode: `MaxEncodedLen`)
	fn close_child_bounty_active() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1005`
		//  Estimated: `8799`
		// Minimum execution time: 107_479_000 picoseconds.
		Weight::from_parts(110_223_000, 0)
			.saturating_add(Weight::from_parts(0, 8799))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(7))
	}
}
