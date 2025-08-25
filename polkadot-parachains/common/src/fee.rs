//! Fee model used in encointer's runtimes.
//!
//! Copied from statemine/src/constants but minimally adjusted where documented.

use frame_support::weights::{
	constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
	WeightToFeePolynomial,
};
use polkadot_core_primitives::Balance;
use smallvec::smallvec;

// not existing upstream
pub use polkadot_runtime_common::SlowAdjustingFeeUpdate;

use crate::currency;
use frame_support::{pallet_prelude::Weight, weights::FeePolynomial};
pub use sp_runtime::Perbill;

/// The block saturation level. Fees will be updates based on this value.
pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25); // <- unused upstream (v9020)

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - [0, MAXIMUM_BLOCK_WEIGHT]
///   - [Balance::min, Balance::max]
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl frame_support::weights::WeightToFee for WeightToFee {
	type Balance = Balance;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		let time_poly: FeePolynomial<Balance> = RefTimeToFee::polynomial().into();
		let proof_poly: FeePolynomial<Balance> = ProofSizeToFee::polynomial().into();

		// Take the maximum instead of the sum to charge by the more scarce resource.
		time_poly.eval(weight.ref_time()).max(proof_poly.eval(weight.proof_size()))
	}
}

/// Maps the reference time component of `Weight` to a fee.
pub struct RefTimeToFee;
impl WeightToFeePolynomial for RefTimeToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// In Kusama, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
		// The standard system parachain configuration is 1/10 of that, as in 1/100 CENT.
		let p = currency::CENTS;
		let q = 100 * Balance::from(ExtrinsicBaseWeight::get().ref_time());

		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

/// Maps the proof size component of `Weight` to a fee.
pub struct ProofSizeToFee;
impl WeightToFeePolynomial for ProofSizeToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// Map 10kb proof to 1 CENT.
		let p = currency::CENTS;
		let q = 10_000;

		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

pub fn calculate_weight_to_fee(weight: &Weight) -> Balance {
	<WeightToFee as frame_support::weights::WeightToFee>::weight_to_fee(weight)
}
