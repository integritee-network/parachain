use crate::{
	tests::{
		assert_bridge_hub_kusama_message_accepted, assert_bridge_hub_polkadot_message_received,
		asset_hub_polkadot_location, bridge_hub_polkadot_location, bridged_ksm_at_ah_polkadot,
		create_foreign_on_ah_kusama, create_foreign_on_ah_polkadot, ik_on_ahk, ik_on_ahk_v5,
		ik_on_ahp_v5, ip_on_ahp, ip_on_ahp_v5, set_up_pool_with_dot_on_ah_polkadot,
		set_up_pool_with_ksm_on_ah_kusama, teer_on_self,
	},
	*,
};
use emulated_integration_tests_common::{xcm_emulator::ConvertLocation, USDT_ID};
use frame_support::{
	assert_ok,
	dispatch::RawOrigin,
	traits::{fungible::Mutate as M, fungibles::Mutate},
};
use integration_tests_helpers::asset_test_utils::GovernanceOrigin::Origin;
use kusama_polkadot_system_emulated_network::integritee_kusama_emulated_chain::{
	integritee_kusama_runtime, integritee_kusama_runtime::TEER,
};
use kusama_polkadot_system_emulated_network::integritee_polkadot_emulated_chain::{
	integritee_polkadot_runtime,
};
use sp_runtime::traits::Bounded;
use xcm::{
	latest::AssetTransferFilter::ReserveDeposit, v3::Error::WeightLimitReached,
	v5::AssetTransferFilter::Teleport,
};
use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApi;

fn ik_on_ahk_account() -> AccountId {
	// Todo: replace with asset_hub_kusama_runtime, but the emulated network doesn't expose it.
	integritee_kusama_runtime::xcm_config::LocationToAccountId::convert_location(&ik_on_ahk_v5())
		.expect("can convert ik_on_ahk_v5 to AccountId")
}

fn ik_on_ahp_account() -> AccountId {
	// Todo: replace with polkadot_hub_kusama_runtime, but the emulated network doesn't expose it.
	integritee_polkadot_runtime::xcm_config::LocationToAccountId::convert_location(&ik_on_ahp_v5())
		.expect("can convert ik_on_ahp_v5 to AccountId")
}


#[test]
fn ik_to_ip_xcm_works() {
	const INITIAL_TEER_BALANCE: u128 = 100 * TEER;
	const ONE_KSM: u128 = 1_000_000_000_000;
	const ONE_DOT: u128 = 10_000_000_000;
	const INITIAL_KSM_BALANCE: u128 = 100 * ONE_KSM;
	let recipient = AccountId::new([5u8; 32]);

	let ahk = (Parent, Parachain(1000));

	let root_on_local =
		integritee_kusama_runtime::xcm_config::LocationToAccountId::convert_location(
			&teer_on_self(),
		)
		.unwrap();

	let ik_on_ahk_acc = ik_on_ahk_account();

	// fund the KAH's SA on KBH for paying bridge transport fees
	BridgeHubKusama::fund_para_sovereign(AssetHubKusama::para_id(), 10_000_000_000_000u128);

	// set XCM versions
	AssetHubKusama::force_xcm_version(asset_hub_polkadot_location(), XCM_VERSION);
	BridgeHubKusama::force_xcm_version(bridge_hub_polkadot_location(), XCM_VERSION);

	<AssetHubKusama as TestExt>::execute_with(|| {
		type Assets = <AssetHubKusama as AssetHubKusamaPallet>::Assets;
		type Balances = <AssetHubKusama as AssetHubKusamaPallet>::Balances;

		assert_ok!(<Balances as M<_>>::mint_into(&ik_on_ahk_acc, INITIAL_KSM_BALANCE));
	});

	let ik_on_ahk = ik_on_ahk();
	create_foreign_on_ah_kusama(ik_on_ahk.clone(), false, vec![(ik_on_ahk_account(), 100 * TEER)]);
	set_up_pool_with_ksm_on_ah_kusama(ik_on_ahk, true);

	let bridged_ksm_at_ah_polkadot = bridged_ksm_at_ah_polkadot();

	<AssetHubPolkadot as TestExt>::execute_with(|| {
		type Balances = <AssetHubPolkadot as AssetHubPolkadotPallet>::Balances;

		assert_ok!(<Balances as M<_>>::mint_into(&ik_on_ahp_account(), 100 * ONE_DOT));
	});

	create_foreign_on_ah_polkadot(bridged_ksm_at_ah_polkadot.clone(), true, vec![(ik_on_ahp_account(), 100 * ONE_KSM)]);
	set_up_pool_with_dot_on_ah_polkadot(bridged_ksm_at_ah_polkadot.clone(), true);

	// need to declare the XCMs twice as the generic parameter is coerced to `()` when the
	// weight is queried
	let xcm1 = ik_xcm();
	let xcm2 = ik_xcm();
	<IntegriteeKusama as TestExt>::execute_with(|| {
		type Runtime = <IntegriteeKusama as Chain>::Runtime;
		type RuntimeEvent = <IntegriteeKusama as Chain>::RuntimeEvent;
		type Balances = <IntegriteeKusama as IntegriteeKusamaPallet>::Balances;
		assert_ok!(<Balances as M<_>>::mint_into(&root_on_local, INITIAL_TEER_BALANCE));

		let weight = Runtime::query_xcm_weight(VersionedXcm::from(xcm1)).unwrap();
		<IntegriteeKusama as IntegriteeKusamaPallet>::PolkadotXcm::execute(
			RawOrigin::Root.into(),
			bx!(VersionedXcm::from(xcm2)),
			weight,
		)
		.unwrap();

		assert_expected_events!(
			IntegriteeKusama,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	<AssetHubKusama as TestExt>::execute_with(|| {
		type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

		type Assets = <AssetHubKusama as AssetHubKusamaPallet>::Assets;
		type Balances = <AssetHubKusama as AssetHubKusamaPallet>::Balances;

		assert_expected_events!(
			AssetHubKusama,
			vec![
				// message processed successfully
				RuntimeEvent::MessageQueue(
						pallet_message_queue::Event::Processed { success: true, .. }
				) => {},
			]
		);
	});

	assert_bridge_hub_kusama_message_accepted(true);
	assert_bridge_hub_polkadot_message_received();

	AssetHubPolkadot::execute_with(|| {
		type RuntimeEvent = <AssetHubPolkadot as Chain>::RuntimeEvent;
		assert_expected_events!(
			AssetHubPolkadot,
			vec![
				// Todo! verify other events
				// message processed successfully
				RuntimeEvent::MessageQueue(
					pallet_message_queue::Event::Processed { success: true, .. }
				) => {},
			]
		);
	});

	<IntegriteePolkadot as TestExt>::execute_with(|| {
		type RuntimeEvent = <IntegriteePolkadot as Chain>::RuntimeEvent;
		assert_expected_events!(
			IntegriteePolkadot,
			vec![
				RuntimeEvent::MessageQueue(
					pallet_message_queue::Event::Processed { success: true, .. }
				) => {},			]
		);
	});
}

/// XCM as it is being sent from IK all the way to the IP.
fn ik_xcm<Call>() -> Xcm<Call> {
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
			remote_xcm: ahk_xcm(),
		},
	])
}

/// Nested XCM to be executed as `remote_xcm` from within `ik_xcm` on AHK.
fn ahk_xcm<Call>() -> Xcm<Call> {
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
			remote_xcm: ahp_xcm(),
		},
	])
}

/// Nested XCM to be executed as `remote_xcm` from within `ahk_xcm` on AHP.
fn ahp_xcm<Call>() -> Xcm<Call> {
	type RuntimeCall = <IntegriteeKusama as Chain>::RuntimeCall;

	Xcm(vec![
		SetAppendix(Xcm(vec![
			RefundSurplus,
			DepositAsset { assets: AssetFilter::Wild(WildAsset::All), beneficiary: ik_on_ahp_v5() },
		])),
		WithdrawAsset((Parent, Fungible(300000000000)).into()),
		InitiateTransfer {
			destination: ip_on_ahp_v5(),
			remote_fees: Some(ReserveDeposit(AssetFilter::Definite(
				Asset { id: Parent.into(), fun: Fungible(200000000000) }.into(),
			))),
			preserve_origin: true,
			assets: Default::default(),
			remote_xcm: Default::default(),
		},
	])
}
