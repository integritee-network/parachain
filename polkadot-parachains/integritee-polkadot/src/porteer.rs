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

use crate::{
	xcm_config::{AccountIdToLocation, AssetHubLocation, XcmConfig},
	Balances, ExistentialDeposit, ParachainInfo, Porteer, Runtime,
};
use alloc::vec;
use frame_support::ord_parameter_types;
use integritee_parachains_common::{
	porteer::{
		ah_sibling_xcm, ahk_cousin_location, ik_sibling_v5, integritee_runtime_porteer_mint,
		ip_cousin_v5, ip_sibling_v5,
	},
	xcm_helpers::{burn_asset_xcm, burn_native_xcm, execute_local_and_remote_xcm, teleport_asset},
	AccountId, Balance,
};
use pallet_porteer::{ForwardPortedTokens, PortTokens};
use sp_core::hex2array;
use sp_runtime::traits::Convert;
use xcm::{
	latest::{AssetId, Location, Xcm},
	prelude::{GlobalConsensus, NetworkId, Parachain, XcmError},
	VersionedAssetId, VersionedXcm,
};
use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApi;

ord_parameter_types! {
	pub const IntegriteeKusamaLocation: Location = Location {
			parents: 2,
			interior: (GlobalConsensus(NetworkId::Kusama), Parachain(2015)).into(),
		};
	pub const IntegriteeKusamaSovereignAccount: AccountId = AccountId::new(hex2array!("8e420f365988e859988375baa048faf25a88de7ddd979425ccaad93fea6b244d"));
}

pub struct PortTokensToKusama;

impl PortTokens for PortTokensToKusama {
	type AccountId = AccountId;
	type Balance = Balance;
	type Location = Location;
	type Error = XcmError;

	fn port_tokens(
		who: Self::AccountId,
		amount: Self::Balance,
		location: Option<Self::Location>,
	) -> Result<(), Self::Error> {
		let who_location = AccountIdToLocation::convert(who.clone());
		let fees = Porteer::xcm_fee_config();

		let tentative_xcm = burn_native_xcm(who_location.clone(), amount, 0);
		let local_fee = Self::query_native_fee(tentative_xcm)?;

		let ah_sibling_fee =
			(Location::new(1, Parachain(ParachainInfo::parachain_id().into())), fees.hop1);

		let local_xcm =
			burn_asset_xcm(who_location.clone(), ah_sibling_fee.clone().into(), local_fee);
		let remote_xcm = ah_sibling_xcm(
			integritee_runtime_porteer_mint(who.clone(), amount, location.clone()),
			ah_sibling_fee.into(),
			ip_sibling_v5(),
			ip_cousin_v5(),
			(ahk_cousin_location(), fees.hop2),
			(ik_sibling_v5(), fees.hop3),
		);
		execute_local_and_remote_xcm::<XcmConfig, <XcmConfig as xcm_executor::Config>::RuntimeCall>(
			who_location,
			local_xcm,
			AssetHubLocation::get(),
			remote_xcm,
		)?;
		Ok(())
	}
}

impl PortTokensToKusama {
	fn query_native_fee(xcm: Xcm<()>) -> Result<Balance, XcmError> {
		let local_weight = Runtime::query_xcm_weight(VersionedXcm::V5(xcm)).map_err(|e| {
			log::error!("Could not query weight: {:?}", e);
			XcmError::WeightNotComputable
		})?;

		let local_fee = Runtime::query_weight_to_asset_fee(
			local_weight,
			VersionedAssetId::from(AssetId(Location::here())),
		)
		.map_err(|e| {
			log::error!("Could not convert weight to asset: {:?}", e);
			XcmError::FeesNotMet
		})?;

		Ok(local_fee)
	}
}

impl ForwardPortedTokens for PortTokensToKusama {
	type AccountId = AccountId;
	type Balance = Balance;
	type Location = Location;
	type Error = XcmError;

	fn forward_ported_tokens(
		who: Self::AccountId,
		amount: Self::Balance,
		destination: Self::Location,
	) -> Result<(), Self::Error> {
		let who_location = AccountIdToLocation::convert(who.clone());
		let tentative_xcm = burn_native_xcm(who_location.clone(), amount, 0);
		let local_fee = Self::query_native_fee(tentative_xcm)?;

		let forward_amount = sp_std::cmp::min(
			amount,
			Balances::free_balance(&who)
				.saturating_sub(ExistentialDeposit::get())
				.saturating_sub(local_fee),
		);

		let who_location = AccountIdToLocation::convert(who.clone());
		let asset =
			(Location::new(1, Parachain(ParachainInfo::parachain_id().into())), forward_amount);
		let beneficiary_location = AccountIdToLocation::convert(who.clone());

		teleport_asset::<XcmConfig>(
			who_location,
			beneficiary_location,
			asset.into(),
			local_fee,
			destination,
		)?;
		Ok(())
	}
}
