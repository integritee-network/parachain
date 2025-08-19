use crate::{xcm_config::AccountIdOf, AccountId, Balance};
use frame_support::pallet_prelude::OriginTrait;
use pallet_porteer::XcmFeeParams;
use parity_scale_codec::Encode;
use sp_runtime::{traits::TryConvert, DispatchError};
use sp_std::{boxed::Box, vec};
use xcm::{
	latest::{
		Asset, AssetFilter,
		AssetTransferFilter::{ReserveDeposit, Teleport},
		Location, NetworkId, OriginKind, Parent, WeightLimit, WildAsset, Xcm,
	},
	prelude::{
		DepositAsset, Fungible, GlobalConsensus, Here, InitiateTransfer, Parachain, Polkadot,
		RefundSurplus, SetAppendix, Transact, WithdrawAsset,
	},
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

pub const INTEGRITEE_KUSAMA_PARA_ID: u32 = 2015;
pub const INTEGRITEE_POLKADOT_PARA_ID: u32 = 2039;
pub const ASSET_HUB_POLKADOT_PARA_ID: u32 = 1000;

pub const PALLET_PORTEER_INDEX: u32 = 56;
pub const PALLET_PORTEER_MINT_PORTED_TOKENS: u32 = 56;

pub fn ip_on_ahp_v5() -> Location {
	Location::new(1, [Parachain(INTEGRITEE_POLKADOT_PARA_ID)])
}

pub fn asset_hub_polkadot_location() -> Location {
	Location::new(2, [GlobalConsensus(Polkadot), Parachain(ASSET_HUB_POLKADOT_PARA_ID)])
}

pub fn ik_on_ahp_v5() -> Location {
	Location::new(2, [GlobalConsensus(NetworkId::Kusama), Parachain(INTEGRITEE_KUSAMA_PARA_ID)])
}

/// XCM as it is being sent from IK all the way to the IP.
pub fn ik_xcm<Call, IntegriteePolkadotCall: Encode>(
	call: IntegriteePolkadotCall,
	local_fee: Balance,
	ah_sibling: (Location, Balance),
	ah_cousin: (Location, Balance),
	integritee_cousin_as_sibling: (Location, Balance),
) -> Xcm<Call> {
	Xcm(vec![
		// Assume that we always pay in native for now
		WithdrawAsset((Here, Fungible(local_fee + ah_sibling.1)).into()),
		SetAppendix(Xcm(vec![
			RefundSurplus,
			DepositAsset { assets: AssetFilter::Wild(WildAsset::All), beneficiary: Here.into() },
		])),
		InitiateTransfer {
			destination: ah_sibling.0,
			remote_fees: Some(Teleport(AssetFilter::Definite(
				Asset { id: Here.into(), fun: Fungible(ah_sibling.1) }.into(),
			))),
			preserve_origin: true,
			assets: Default::default(),
			remote_xcm: ah_sibling_xcm(call, ah_cousin, integritee_cousin_as_sibling),
		},
	])
}

/// Nested XCM to be executed as `remote_xcm` from within `ik_xcm` on AHK.
fn ah_sibling_xcm<Call, IntegriteePolkadotCall: Encode>(
	call: IntegriteePolkadotCall,
	ah_cousin: (Location, Balance),
	integritee_cousin_as_sibling: (Location, Balance),
) -> Xcm<Call> {
	Xcm(vec![
		SetAppendix(Xcm(vec![
			RefundSurplus,
			DepositAsset {
				assets: AssetFilter::Wild(WildAsset::All),
				beneficiary: (Parent, Parachain(2015)).into(),
			},
		])),
		WithdrawAsset((Parent, Fungible(ah_cousin.1)).into()),
		InitiateTransfer {
			destination: ah_cousin.0,
			remote_fees: Some(ReserveDeposit(AssetFilter::Definite(
				Asset { id: Parent.into(), fun: Fungible(ah_cousin.1) }.into(),
			))),
			preserve_origin: true,
			assets: Default::default(),
			remote_xcm: ah_cousin_xcm(call, integritee_cousin_as_sibling),
		},
	])
}

/// Nested XCM to be executed as `remote_xcm` from within `ahk_xcm` on AHP.
fn ah_cousin_xcm<Call, IntegriteePolkadotCall: Encode>(
	call: IntegriteePolkadotCall,
	integritee_cousin_as_sibling: (Location, Balance),
) -> Xcm<Call> {
	Xcm(vec![
		SetAppendix(Xcm(vec![
			RefundSurplus,
			// Fixme: Our XCM Config seems broken currently. It fails to deposit the asset and traps it.
			// I guess that it fails to convert the Location to our local AssetId.
			// Log observed:
			//  PolkadotXcm(Event::AssetsTrapped { hash: 0xb225b0f34edb281841f89c7237884f1e41746c8d1874770fca38a95845ca41ae, origin: Location { parents: 2, interior: X2([GlobalConsensus(Kusama), Parachain(2015)]) }, assets: V5(Assets([Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(16724748580) }])) })
			DepositAsset { assets: AssetFilter::Wild(WildAsset::All), beneficiary: ik_on_ahp_v5() },
		])),
		WithdrawAsset((Parent, Fungible(integritee_cousin_as_sibling.1)).into()),
		InitiateTransfer {
			destination: integritee_cousin_as_sibling.0,
			remote_fees: Some(ReserveDeposit(AssetFilter::Definite(
				Asset { id: Parent.into(), fun: Fungible(integritee_cousin_as_sibling.1) }.into(),
			))),
			preserve_origin: true,
			assets: Default::default(),
			remote_xcm: ip_xcm(call),
		},
	])
}

fn ip_xcm<Call, IntegriteePolkadotCall: Encode>(call: IntegriteePolkadotCall) -> Xcm<Call> {
	Xcm(vec![Transact {
		origin_kind: OriginKind::SovereignAccount,
		fallback_max_weight: None,
		call: call.encode().into(),
	}])
}
