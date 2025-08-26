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

#[test]
fn porteer_mint_from_ik_works_without_forwarding() {
	// Need to run with `RUST_LOG=DEBUG` to see the logs.
	sp_tracing::init_for_tests();
	sp_io::TestExternalities::default().execute_with(|| {
		let fee_asset: Asset = (Junctions::Here, TEER).into();

		let bob = sp_keyring::Sr25519Keyring::Bob.to_account_id();
		let mint_amount = TEER;
		let bob_balance_before = Balances::free_balance(&bob);

		Balances::make_free_balance_be(&IntegriteeKusamaSovereignAccount::get(), 4 * TEER);
		Porteer::set_porteer_config(
			RawOrigin::Root.into(),
			PorteerConfig { send_enabled: true, receive_enabled: true },
		)
		.unwrap();

		let message = Xcm(vec![
			// Assume that the IntegriteeKusamaSovereign account has some TEER
			WithdrawAsset(fee_asset.clone().into()),
			PayFees { asset: fee_asset },
			SetAppendix(Xcm::<()>(vec![
				// Not sure if we can use this across the bridge...
				// ReportError(QueryResponseInfo {
				// 	destination: (Parent, Parachain(42)).into(),
				// 	query_id: 1,
				// 	max_weight: Weight::zero(),
				// }),
				RefundSurplus,
				DepositAsset {
					assets: AssetFilter::Wild(WildAsset::All),
					beneficiary: IntegriteeKusamaLocation::get(),
				},
			])),
			Transact {
				origin_kind: OriginKind::SovereignAccount,
				fallback_max_weight: None,
				call: RuntimeCall::Porteer(pallet_porteer::Call::mint_ported_tokens {
					beneficiary: bob.clone(),
					amount: mint_amount,
					forward_tokens_to_location: None,
					source_nonce: 0,
				})
				.encode()
				.into(),
			},
		]);

		let message =
			Xcm::<<XcmConfig as xcm_executor::Config>::RuntimeCall>::from(message.clone());
		let mut hash = Default::default();

		// Execute message in this parachain with IntegriteeKusamaOrigin
		let result = XcmExecutor::<XcmConfig>::prepare_and_execute(
			IntegriteeKusamaLocation::get(),
			message,
			&mut hash,
			Weight::MAX,
			Weight::zero(),
		);

		// This does not catch errors from within the Porteer pallet.
		assert!(matches!(result, Outcome::Complete { .. }));

		assert_eq!(Balances::free_balance(&bob), bob_balance_before + mint_amount);
	});
}
