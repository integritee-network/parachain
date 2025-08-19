use crate::{xcm_config::AccountIdOf, AccountId, Balance};
use frame_support::pallet_prelude::OriginTrait;
use pallet_porteer::XcmFeeParams;
use sp_runtime::{traits::TryConvert, DispatchError};
use sp_std::{boxed::Box, vec};
use xcm::{
	latest::{Asset, Location, WeightLimit},
	prelude::{Fungible, Here},
};

pub const IK_FEE: u128 = 1000000000000;
pub const AHK_FEE: u128 = 33849094374679;
pub const AHP_FEE: u128 = 300000000000;
pub const IP_FEE: u128 = 1000000000000;

pub const DEFAULT_XCM_FEES_IK_PERSPECTIVE: XcmFeeParams<Balance> =
	XcmFeeParams { hop1: AHK_FEE, hop2: AHP_FEE, hop3: IP_FEE };

pub const DEFAULT_XCM_FEES_IP_PERSPECTIVE: XcmFeeParams<Balance> =
	XcmFeeParams { hop1: AHP_FEE, hop2: AHK_FEE, hop3: IK_FEE };

pub fn forward_teer<
	T: pallet_xcm::Config + frame_system::Config,
	AccountIdToLocation: for<'a> TryConvert<&'a AccountIdOf<T>, Location>,
>(
	who: AccountIdOf<T>,
	destination: Location,
	amount: Balance,
) -> Result<(), DispatchError> {
	let beneficiary_location = AccountIdToLocation::try_convert(&who).unwrap();
	pallet_xcm::Pallet::<T>::transfer_assets(
		<T as frame_system::Config>::RuntimeOrigin::signed(who.clone()),
		Box::new(destination.into_versioned()),
		Box::new(beneficiary_location.into()),
		Box::new(vec![Asset { id: Here.into(), fun: Fungible(amount) }.into()].into()),
		0,
		WeightLimit::Unlimited,
	)
}

/// The porteer::mint call used by both runtimes.
///
/// We have tests in the runtimes that ensure that the
/// format and the indexes are correct.
pub fn integritee_runtime_porteer_mint(
	beneficiary: AccountId,
	amount: Balance,
	location: Option<Location>,
) -> ([u8; 2], AccountId, Balance, Option<Location>) {
	// ([pallet_index, call_index], ...)
	([56, 7], beneficiary, amount, location)
}
