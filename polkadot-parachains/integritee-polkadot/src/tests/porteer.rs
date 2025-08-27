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
	porteer::IntegriteeKusamaLocation, xcm_config::XcmConfig, Balances,
	IntegriteeKusamaSovereignAccount, Porteer, RuntimeCall, TEER,
};
use frame_support::{__private::sp_tracing, dispatch::RawOrigin, traits::Currency};
use integritee_parachains_common::porteer::integritee_runtime_porteer_mint;
use pallet_porteer::PorteerConfig;
use parity_scale_codec::{Decode, Encode};
use xcm::{
	latest::{Asset, AssetFilter, ExecuteXcm, Junctions, OriginKind, Weight, WildAsset, Xcm},
	prelude::{DepositAsset, PayFees, RefundSurplus, SetAppendix, Transact, WithdrawAsset},
	v5::Outcome,
};
use xcm_executor::{traits::ConvertLocation, XcmExecutor};

#[test]
fn ik_porteer_sovereign_account_matches() {
	sp_io::TestExternalities::default().execute_with(|| {
		let account = crate::xcm_config::LocationToAccountId::convert_location(
			&IntegriteeKusamaLocation::get(),
		)
		.unwrap();

		assert_eq!(account, IntegriteeKusamaSovereignAccount::get());
	});
}

#[test]
fn integritee_polkadot_porteer_mint_is_correct() {
	let beneficiary = IntegriteeKusamaSovereignAccount::get();
	let amount = 10;
	let call = integritee_runtime_porteer_mint(beneficiary.clone(), amount, None, 0);

	let decoded = RuntimeCall::decode(&mut call.encode().as_slice()).unwrap();

	assert_eq!(
		decoded,
		RuntimeCall::Porteer(pallet_porteer::Call::mint_ported_tokens {
			beneficiary,
			amount,
			forward_tokens_to_location: None,
			source_nonce: 0,
		})
	)
}
