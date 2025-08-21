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

//! Some generic XCM helpers for executing and sending XCMs.

use frame_support::pallet_prelude::Weight;
use parity_scale_codec::Encode;
use xcm::latest::{Asset, AssetFilter, ExecuteXcm, Location, SendXcm, WeightLimit, WildAsset, Xcm};
use xcm::prelude::{AllCounted, BurnAsset, BuyExecution, ClearOrigin, DepositAsset, Fungible, Here, PayFees, ReceiveTeleportedAsset, RefundSurplus, SetAppendix, Wild, WithdrawAsset, XcmError};
use xcm_executor::XcmExecutor;
use crate::Balance;

/// Returns an XCM that is meant to be executed locally to burn the native asset.
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
    let remote_xcm = receive_teleported_asset(asset, beneficiary);

    execute_local_and_remote_xcm::<XcmConfig, <XcmConfig as xcm_executor::Config>::RuntimeCall>(
        who,
        xcm,
        destination,
        remote_xcm,
    )?;
    Ok(())
}

pub fn execute_local_and_remote_xcm<XcmConfig: xcm_executor::Config<RuntimeCall = Call>, Call>(
    who: Location,
    local_xcm: Xcm<Call>,
    destination: Location,
    remote_xcm: Xcm<()>,
) -> Result<(), XcmError> {
    let mut hash = local_xcm.using_encoded(sp_io::hashing::blake2_256);
    let outcome = XcmExecutor::<XcmConfig>::prepare_and_execute(
        who,
        local_xcm,
        &mut hash,
        Weight::MAX,
        Weight::zero(),
    );

    outcome.ensure_complete().inspect_err(|&error| {
        log::error!("Local execution is incomplete: {:?}", error);
    })?;

    let (ticket, _delivery_fees) = <XcmConfig as xcm_executor::Config>::XcmSender::validate(
        &mut Some(destination),
        &mut Some(remote_xcm),
    )?;

    <XcmConfig as xcm_executor::Config>::XcmSender::deliver(ticket)?;
    Ok(())
}