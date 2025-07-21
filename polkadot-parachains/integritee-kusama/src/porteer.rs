use crate::*;

use parity_scale_codec::Encode;
use xcm::{
	latest::{
		Asset, AssetFilter,
		AssetTransferFilter::{ReserveDeposit, Teleport},
		Location, NetworkId, OriginKind, Parent, WildAsset, Xcm,
	},
	prelude::{
		DepositAsset, Fungible, GlobalConsensus, Here, InitiateTransfer, Parachain, Polkadot,
		RefundSurplus, SetAppendix, Transact, WithdrawAsset,
	},
};

pub const INTEGRITEE_KUSAMA_PARA_ID: u32 = 2015;
pub const INTEGRITEE_POLKADOT_PARA_ID: u32 = 2039;
pub const ASSET_HUB_POLKADOT_PARA_ID: u32 = 1000;

// For testing
pub fn integritee_polkadot_system_remark(remark: Vec<u8>) -> ([u8; 2], Vec<u8>) {
	// ([pallet_index, call_index], remark)
	([0, 0], remark)
}

pub fn integritee_polkadot_porteer_mint(beneficiary: AccountId, amount: Balance) -> ([u8; 2], AccountId, Balance) {
	// ([pallet_index, call_index], ...)
	([56, 2], beneficiary, amount)
}

pub(crate) fn ip_on_ahp_v5() -> Location {
	Location::new(1, [Parachain(INTEGRITEE_POLKADOT_PARA_ID)])
}

pub(crate) fn asset_hub_polkadot_location() -> Location {
	Location::new(2, [GlobalConsensus(Polkadot), Parachain(ASSET_HUB_POLKADOT_PARA_ID)])
}

pub(crate) fn ik_on_ahp_v5() -> Location {
	Location::new(2, [GlobalConsensus(NetworkId::Kusama), Parachain(INTEGRITEE_KUSAMA_PARA_ID)])
}

/// XCM as it is being sent from IK all the way to the IP.
pub fn ik_xcm<Call, IntegriteePolkadotCall: Encode>(call: IntegriteePolkadotCall) -> Xcm<Call> {
	const ALAIN_WITHDRAW: u128 = 34849094374679;
	const ALAIN_REMOTE_FEE: u128 = 33849094374679;

	Xcm(vec![
		// Assume that we always pay in native for now
		WithdrawAsset((Here, Fungible(ALAIN_WITHDRAW * 2)).into()),
		SetAppendix(Xcm(vec![
			RefundSurplus,
			DepositAsset { assets: AssetFilter::Wild(WildAsset::All), beneficiary: Here.into() },
		])),
		InitiateTransfer {
			destination: (Parent, Parachain(1000)).into(),
			remote_fees: Some(Teleport(AssetFilter::Definite(
				Asset { id: Here.into(), fun: Fungible(ALAIN_REMOTE_FEE * 2) }.into(),
			))),
			preserve_origin: true,
			assets: Default::default(),
			remote_xcm: ahk_xcm(call),
		},
	])
}

/// Nested XCM to be executed as `remote_xcm` from within `ik_xcm` on AHK.
fn ahk_xcm<Call, IntegriteePolkadotCall: Encode>(call: IntegriteePolkadotCall) -> Xcm<Call> {
	Xcm(vec![
		SetAppendix(Xcm(vec![
			RefundSurplus,
			DepositAsset {
				assets: AssetFilter::Wild(WildAsset::All),
				beneficiary: (Parent, Parachain(2015)).into(),
			},
		])),
		WithdrawAsset((Parent, Fungible(300000000000)).into()),
		InitiateTransfer {
			destination: asset_hub_polkadot_location(),
			remote_fees: Some(ReserveDeposit(AssetFilter::Definite(
				Asset { id: Parent.into(), fun: Fungible(200000000000) }.into(),
			))),
			preserve_origin: true,
			assets: Default::default(),
			remote_xcm: ahp_xcm(call),
		},
	])
}

/// Nested XCM to be executed as `remote_xcm` from within `ahk_xcm` on AHP.
fn ahp_xcm<Call, IntegriteePolkadotCall: Encode>(call: IntegriteePolkadotCall) -> Xcm<Call> {
	Xcm(vec![
		SetAppendix(Xcm(vec![
			RefundSurplus,
			// Fixme: Our XCM Config seems broken currently. It fails to deposit the asset and traps it.
			// I guess that it fails to convert the Location to our local AssetId.
			// Log observed:
			//  PolkadotXcm(Event::AssetsTrapped { hash: 0xb225b0f34edb281841f89c7237884f1e41746c8d1874770fca38a95845ca41ae, origin: Location { parents: 2, interior: X2([GlobalConsensus(Kusama), Parachain(2015)]) }, assets: V5(Assets([Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(16724748580) }])) })
			DepositAsset { assets: AssetFilter::Wild(WildAsset::All), beneficiary: ik_on_ahp_v5() },
		])),
		WithdrawAsset((Parent, Fungible(1000000000000)).into()),
		InitiateTransfer {
			destination: ip_on_ahp_v5(),
			remote_fees: Some(ReserveDeposit(AssetFilter::Definite(
				Asset { id: Parent.into(), fun: Fungible(1000000000000) }.into(),
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
