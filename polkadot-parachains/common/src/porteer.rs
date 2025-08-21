// Copyright 2021 Integritee AG and Supercomputing Systems AG
// This file is part of the "Integritee parachain" and is
// based on Cumulus from Parity Technologies (UK) Ltd.

// Integritee parachain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Integritee parachain.  If not, see <http://www.gnu.org/licenses/>.

//! Common code that the Porteer requires to send tokens back and forth between IK<>IP.

use crate::{AccountId, Balance};
use pallet_porteer::XcmFeeParams;
use parity_scale_codec::Encode;
use sp_std::vec;
use xcm::{
	latest::{
		Asset, AssetFilter, AssetTransferFilter::ReserveDeposit, Location, OriginKind, Parent,
		WildAsset, Xcm,
	},
	prelude::{
		DepositAsset, Fungible, GlobalConsensus, InitiateTransfer, Kusama, Parachain, PayFees,
		Polkadot, ReceiveTeleportedAsset, RefundSurplus, SetAppendix, Transact, WithdrawAsset,
	},
};

pub const IK_FEE: u128 = 1000000000000;
pub const AHK_FEE: u128 = 33849094374679;
pub const AHP_FEE: u128 = 3000000000000;
pub const IP_FEE: u128 = 1000000000000;

pub const DEFAULT_XCM_FEES_IK_PERSPECTIVE: XcmFeeParams<Balance> =
	XcmFeeParams { hop1: AHK_FEE, hop2: AHP_FEE, hop3: IP_FEE };

pub const DEFAULT_XCM_FEES_IP_PERSPECTIVE: XcmFeeParams<Balance> =
	XcmFeeParams { hop1: AHP_FEE, hop2: AHK_FEE, hop3: IK_FEE };

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

pub fn ahk_cousin_location() -> Location {
	Location::new(2, [GlobalConsensus(Kusama), Parachain(ASSET_HUB_KUSAMA_PARA_ID)])
}

pub fn ahp_cousin_location() -> Location {
	Location::new(2, [GlobalConsensus(Polkadot), Parachain(ASSET_HUB_POLKADOT_PARA_ID)])
}

/// XCM to be executed on the first hop, namely the Asset Hub sibling.
pub fn ah_sibling_xcm<Call, IntegriteePolkadotCall: Encode>(
	call: IntegriteePolkadotCall,
	teleported_asset: Asset,
	local_as_sibling: Location,
	local_as_cousin: Location,
	ah_cousin: (Location, Balance),
	integritee_cousin_as_sibling: (Location, Balance),
) -> Xcm<Call> {
	Xcm(vec![
		ReceiveTeleportedAsset(teleported_asset.clone().into()),
		PayFees { asset: teleported_asset },
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
