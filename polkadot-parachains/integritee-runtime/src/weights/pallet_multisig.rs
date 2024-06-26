
//! Autogenerated weights for `pallet_multisig`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 36.0.0
//! DATE: 2024-05-31, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=pallet_multisig
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./polkadot-parachains/integritee-runtime/src/weights/pallet_multisig.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_multisig`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_multisig::WeightInfo for WeightInfo<T> {
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_threshold_1(z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_462_000 picoseconds.
		Weight::from_parts(8_083_670, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 8
			.saturating_add(Weight::from_parts(71, 0).saturating_mul(z.into()))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(465), added: 2940, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_create(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `159 + s * (11 ±0)`
		//  Estimated: `3930`
		// Minimum execution time: 26_810_000 picoseconds.
		Weight::from_parts(24_185_896, 0)
			.saturating_add(Weight::from_parts(0, 3930))
			// Standard Error: 9_927
			.saturating_add(Weight::from_parts(436_678, 0).saturating_mul(s.into()))
			// Standard Error: 8
			.saturating_add(Weight::from_parts(1_321, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(465), added: 2940, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[3, 10]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_approve(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `282`
		//  Estimated: `3930`
		// Minimum execution time: 13_432_000 picoseconds.
		Weight::from_parts(13_486_148, 0)
			.saturating_add(Weight::from_parts(0, 3930))
			// Standard Error: 11_053
			.saturating_add(Weight::from_parts(150_758, 0).saturating_mul(s.into()))
			// Standard Error: 8
			.saturating_add(Weight::from_parts(1_305, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(465), added: 2940, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_complete(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `301 + s * (43 ±0)`
		//  Estimated: `3930`
		// Minimum execution time: 27_667_000 picoseconds.
		Weight::from_parts(28_803_935, 0)
			.saturating_add(Weight::from_parts(0, 3930))
			// Standard Error: 34_707
			.saturating_add(Weight::from_parts(17_329, 0).saturating_mul(s.into()))
			// Standard Error: 29
			.saturating_add(Weight::from_parts(1_218, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(465), added: 2940, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	fn approve_as_multi_create(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `159 + s * (11 ±0)`
		//  Estimated: `3930`
		// Minimum execution time: 19_788_000 picoseconds.
		Weight::from_parts(24_720_333, 0)
			.saturating_add(Weight::from_parts(0, 3930))
			// Standard Error: 15_090
			.saturating_add(Weight::from_parts(18_838, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(465), added: 2940, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	fn approve_as_multi_approve(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `282`
		//  Estimated: `3930`
		// Minimum execution time: 12_090_000 picoseconds.
		Weight::from_parts(12_248_944, 0)
			.saturating_add(Weight::from_parts(0, 3930))
			// Standard Error: 12_358
			.saturating_add(Weight::from_parts(312_946, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(465), added: 2940, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	fn cancel_as_multi(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `365 + s * (11 ±0)`
		//  Estimated: `3930`
		// Minimum execution time: 21_578_000 picoseconds.
		Weight::from_parts(23_432_827, 0)
			.saturating_add(Weight::from_parts(0, 3930))
			// Standard Error: 8_169
			.saturating_add(Weight::from_parts(330_245, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
