use frame_support::ord_parameter_types;
use pallet_porteer::{ForwardPortedTokens, PortTokens};
use sp_core::hex2array;
use sp_runtime::traits::Convert;
use xcm::latest::{AssetId, Location, Xcm};
use xcm::prelude::{GlobalConsensus, NetworkId, Parachain, XcmError};
use xcm::{VersionedAssetId, VersionedXcm};
use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApi;
use integritee_parachains_common::{AccountId, Balance};
use integritee_parachains_common::porteer::{ah_sibling_xcm, ahk_cousin_location, ik_sibling_v5, integritee_runtime_porteer_mint, ip_cousin_v5, ip_sibling_v5};
use integritee_parachains_common::xcm_helpers::{burn_asset_xcm, burn_native_xcm, execute_local_and_remote_xcm, teleport_asset};
use crate::{Balances, ExistentialDeposit, ParachainInfo, Porteer, Runtime};
use crate::xcm_config::{AccountIdToLocation, AssetHubLocation, XcmConfig};

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
