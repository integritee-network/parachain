use crate::{xcm_config::AccountIdOf, AccountId, Balance};
use frame_support::pallet_prelude::{OriginTrait, Weight};
use pallet_porteer::XcmFeeParams;
use parity_scale_codec::Encode;
use sp_runtime::{traits::TryConvert, DispatchError};
use sp_std::{boxed::Box, vec};
use xcm::{
	latest::{
		Asset, AssetFilter,
		AssetTransferFilter::{ReserveDeposit, Teleport},
		Location, OriginKind, Parent, WeightLimit, WildAsset, Xcm,
	},
	prelude::{
		AllCounted, BurnAsset, BuyExecution, ClearOrigin, DepositAsset, ExecuteXcm, Fungible,
		GlobalConsensus, Here, InitiateTransfer, Kusama, Parachain, PayFees, Polkadot,
		ReceiveTeleportedAsset, RefundSurplus, SendXcm, SetAppendix, Transact, Wild, WithdrawAsset,
	},
};
use xcm::prelude::XcmError;
use xcm_executor::XcmExecutor;

pub const IK_FEE: u128 = 1000000000000;
pub const AHK_FEE: u128 = 33849094374679;
pub const AHP_FEE: u128 = 3000000000000;
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
		Box::new(vec![Asset { id: Here.into(), fun: Fungible(amount) }].into()),
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
pub const ASSET_HUB_KUSAMA_PARA_ID: u32 = 1000;
pub const ASSET_HUB_POLKADOT_PARA_ID: u32 = 1000;

pub const PALLET_PORTEER_INDEX: u32 = 56;
pub const PALLET_PORTEER_MINT_PORTED_TOKENS: u32 = 56;

pub fn ik_sibling_v5() -> Location {
	Location::new(1, [Parachain(INTEGRITEE_KUSAMA_PARA_ID)])
}

pub fn ik_cousin_v5() -> Location {
	Location::new(2, [GlobalConsensus(Kusama), Parachain(INTEGRITEE_KUSAMA_PARA_ID)])
}

pub fn ip_sibling_v5() -> Location {
	Location::new(1, [Parachain(INTEGRITEE_POLKADOT_PARA_ID)])
}

pub fn ip_cousin_v5() -> Location {
	Location::new(2, [GlobalConsensus(Polkadot), Parachain(INTEGRITEE_POLKADOT_PARA_ID)])
}

pub fn asset_hub_kusama_location() -> Location {
	Location::new(2, [GlobalConsensus(Kusama), Parachain(ASSET_HUB_KUSAMA_PARA_ID)])
}

pub fn asset_hub_polkadot_location() -> Location {
	Location::new(2, [GlobalConsensus(Polkadot), Parachain(ASSET_HUB_POLKADOT_PARA_ID)])
}

/// XCM as it is being sent from the local Integritee chain to its cousin.
pub fn local_integritee_xcm<Call, IntegriteePolkadotCall: Encode>(
	call: IntegriteePolkadotCall,
	local_fee: Balance,
	local_as_sibling: Location,
	local_as_cousin: Location,
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
			remote_xcm: ah_sibling_xcm(
				call,
				local_as_sibling,
				local_as_cousin,
				ah_cousin,
				integritee_cousin_as_sibling,
			),
		},
	])
}

/// Returns an Xcm that is meant to be executed locally to burn the native asset.
pub fn burn_native_xcm<Call>(who: Location, amount: Balance, local_fees: Balance) -> Xcm<Call> {
	let asset: Asset = (Here, Fungible(amount)).into();
	burn_asset_xcm(who, asset, local_fees)
}

/// Returns an Xcm that is meant to be executed locally to burn the `Asset`.
pub fn burn_asset_xcm<Call>(who: Location, asset: Asset, local_fees: Balance) -> Xcm<Call> {
	Xcm(vec![
		WithdrawAsset(vec![asset.clone(), (Here, Fungible(local_fees)).into()].into()),
		PayFees { asset: (Here, Fungible(local_fees)).into() },
		SetAppendix(Xcm(vec![
			RefundSurplus,
			DepositAsset { assets: AssetFilter::Wild(WildAsset::All), beneficiary: who },
		])),
		BurnAsset(asset.into()),
	])
}

pub fn receive_teleported_asset<Call>(asset: Asset, beneficiary: Location) -> Xcm<Call> {
	Xcm(vec![
		ReceiveTeleportedAsset(asset.clone().into()),
		ClearOrigin,
		BuyExecution { fees: asset, weight_limit: WeightLimit::Unlimited },
		DepositAsset { assets: Wild(AllCounted(1)), beneficiary },
	])
}

pub fn teleport_asset<XcmConfig: xcm_executor::Config>(
	who: Location,
	beneficiary: Location,
	asset: Asset,
	local_fee: Balance,
	destination: Location,
) -> Result<(), XcmError> {
	let xcm = burn_asset_xcm(who.clone(), asset.clone(), local_fee);

	let mut hash = xcm.using_encoded(sp_io::hashing::blake2_256);
	let outcome = XcmExecutor::<XcmConfig>::prepare_and_execute(
		who,
		xcm,
		&mut hash,
		Weight::MAX,
		Weight::zero(),
	);

	outcome.ensure_complete().map_err(|error| {
		log::error!("Local execution is incomplete: {:?}", error);
		error
	})?;

	let remote_xcm = receive_teleported_asset(asset.into(), beneficiary);

	let (ticket, _delivery_fees) = <XcmConfig as xcm_executor::Config>::XcmSender::validate(
		&mut Some(destination),
		&mut Some(remote_xcm),
	)?;

	<XcmConfig as xcm_executor::Config>::XcmSender::deliver(ticket)?;
	Ok(())
}

/// Nested XCM to be executed as `remote_xcm` from within `local_integritee_xcm` on the
/// first hop, namely the Asset Hub sibling.
fn ah_sibling_xcm<Call, IntegriteePolkadotCall: Encode>(
	call: IntegriteePolkadotCall,
	local_as_sibling: Location,
	local_as_cousin: Location,
	ah_cousin: (Location, Balance),
	integritee_cousin_as_sibling: (Location, Balance),
) -> Xcm<Call> {
	Xcm(vec![
		SetAppendix(Xcm(vec![
			RefundSurplus,
			DepositAsset {
				assets: AssetFilter::Wild(WildAsset::All),
				beneficiary: local_as_sibling,
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
			remote_xcm: ah_cousin_xcm(call, local_as_cousin, integritee_cousin_as_sibling),
		},
	])
}

/// Nested XCM to be executed as `remote_xcm` from within `ah_sibling_xcm` on the
/// second hop, namely the Asset Hub cousin.
fn ah_cousin_xcm<Call, IntegriteePolkadotCall: Encode>(
	call: IntegriteePolkadotCall,
	local_as_cousin: Location,
	integritee_cousin_as_sibling: (Location, Balance),
) -> Xcm<Call> {
	Xcm(vec![
		SetAppendix(Xcm(vec![
			RefundSurplus,
			DepositAsset {
				assets: AssetFilter::Wild(WildAsset::All),
				beneficiary: local_as_cousin,
			},
		])),
		WithdrawAsset((Parent, Fungible(integritee_cousin_as_sibling.1)).into()),
		InitiateTransfer {
			destination: integritee_cousin_as_sibling.0,
			remote_fees: Some(ReserveDeposit(AssetFilter::Definite(
				Asset { id: Parent.into(), fun: Fungible(integritee_cousin_as_sibling.1) }.into(),
			))),
			preserve_origin: true,
			assets: Default::default(),
			remote_xcm: integritee_cousin_xcm(call),
		},
	])
}

fn integritee_cousin_xcm<Call, IntegriteePolkadotCall: Encode>(
	call: IntegriteePolkadotCall,
) -> Xcm<Call> {
	Xcm(vec![Transact {
		origin_kind: OriginKind::SovereignAccount,
		fallback_max_weight: None,
		call: call.encode().into(),
	}])
}
