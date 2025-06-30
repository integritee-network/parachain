use crate::{
	xcm_config::XcmConfig, Balances, IntegriteeKusamaLocation, IntegriteeKusamaSovereignAccount,
	Porteer, RuntimeCall, TEER,
};
use frame_support::{__private::sp_tracing, dispatch::RawOrigin, traits::Currency};
use pallet_porteer::PorteerConfig;
use parity_scale_codec::Encode;
use staging_xcm::{
	latest::{Asset, AssetFilter, ExecuteXcm, Junctions, OriginKind, Weight, WildAsset, Xcm},
	prelude::{DepositAsset, PayFees, RefundSurplus, SetAppendix, Transact, WithdrawAsset},
	v5::Outcome,
};
use staging_xcm_executor::{traits::ConvertLocation, XcmExecutor};

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
fn porteer_mint_from_ik_works() {
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
				})
				.encode()
				.into(),
			},
		]);

		let message =
			Xcm::<<XcmConfig as staging_xcm_executor::Config>::RuntimeCall>::from(message.clone());
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
