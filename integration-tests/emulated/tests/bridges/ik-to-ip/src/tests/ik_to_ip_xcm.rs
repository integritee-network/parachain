use crate::{
	tests::{
		assert_bridge_hub_kusama_message_accepted, assert_bridge_hub_polkadot_message_received,
		asset_hub_polkadot_location, bridge_hub_polkadot_location, bridged_ksm_at_ah_polkadot,
		create_foreign_on_ah_kusama, create_foreign_on_ah_polkadot, create_reserve_asset_on_ip,
		ik_on_ahk, ik_on_ahk_v5, ik_on_ahp_v5, ip_on_ahp, ip_on_ahp_v5,
		set_up_pool_with_dot_on_ah_polkadot, set_up_pool_with_ksm_on_ah_kusama, teer_on_self,
	},
	*,
};
use emulated_integration_tests_common::{
	impls::Parachain,
	xcm_emulator::{log, ConvertLocation},
};
use frame_support::{assert_ok, dispatch::RawOrigin, traits::fungible::Mutate as M};
use kusama_polkadot_system_emulated_network::{
	integritee_kusama_emulated_chain::integritee_kusama_runtime::TEER,
	integritee_polkadot_emulated_chain::integritee_polkadot_runtime,
};
use xcm::{latest::AssetTransferFilter::ReserveDeposit, v5::AssetTransferFilter::Teleport};
use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApi;

fn ik_on_ahk_account() -> AccountId {
	AssetHubKusama::sovereign_account_id_of(ik_on_ahk_v5())
}

fn ik_on_ahp_account() -> AccountId {
	AssetHubPolkadot::sovereign_account_of_parachain_on_other_global_consensus(
		KusamaId,
		IntegriteeKusama::para_id(),
	)
}

#[test]
fn ik_to_ip_xcm_works() {
	const INITIAL_TEER_BALANCE: u128 = 100 * TEER;
	const ONE_KSM: u128 = 1_000_000_000_000;
	const ONE_DOT: u128 = 10_000_000_000;
	const INITIAL_KSM_BALANCE: u128 = 100 * ONE_KSM;

	// // set XCM versions
	AssetHubKusama::force_xcm_version(asset_hub_polkadot_location(), XCM_VERSION);
	AssetHubPolkadot::force_xcm_version(ip_on_ahp_v5(), XCM_VERSION);
	AssetHubPolkadot::force_xcm_version(ik_on_ahp_v5(), XCM_VERSION);
	BridgeHubKusama::force_xcm_version(bridge_hub_polkadot_location(), XCM_VERSION);

	let root_on_local =
		<IntegriteeKusama as Parachain>::LocationToAccountId::convert_location(&teer_on_self())
			.unwrap();
	let ik_on_ahk_acc = ik_on_ahk_account();
	let ik_on_ahp_acc = ik_on_ahp_account();

	// fund the KAH's SA on KBH for paying bridge transport fees
	BridgeHubKusama::fund_para_sovereign(AssetHubKusama::para_id(), 10 * ONE_KSM);

	// Fund accounts
	let ip_treasury = integritee_polkadot_runtime::TreasuryAccount::get();
	IntegriteePolkadot::fund_accounts(vec![
		(ip_treasury.clone(), 100 * TEER),
		(ik_on_ahp_acc.clone(), 100 * TEER),
	]);

	AssetHubKusama::fund_accounts(vec![(ik_on_ahk_acc, INITIAL_KSM_BALANCE)]);
	AssetHubPolkadot::fund_accounts(vec![(ik_on_ahp_acc.clone(), 100 * ONE_DOT)]);

	let ik_on_ahk = ik_on_ahk();
	create_foreign_on_ah_kusama(ik_on_ahk.clone(), false, vec![(ik_on_ahk_account(), 100 * TEER)]);
	set_up_pool_with_ksm_on_ah_kusama(ik_on_ahk, true);

	let bridged_ksm_at_ah_polkadot = bridged_ksm_at_ah_polkadot();
	create_foreign_on_ah_polkadot(
		bridged_ksm_at_ah_polkadot.clone(),
		true,
		vec![(ik_on_ahp_acc.clone(), 100 * ONE_KSM)],
	);
	set_up_pool_with_dot_on_ah_polkadot(bridged_ksm_at_ah_polkadot.clone(), true);

	create_reserve_asset_on_ip(
		0,
		Parent.into(),
		true,
		vec![(ik_on_ahp_acc.clone(), 100 * ONE_DOT), (ip_treasury, 100 * ONE_DOT)],
	);

	log::info!("Setup Done! Sending XCM.");

	// need to declare the XCMs twice as the generic parameter is coerced to `()` when the
	// weight is queried
	let xcm1 = ik_xcm();
	let xcm2 = ik_xcm();
	<IntegriteeKusama as TestExt>::execute_with(|| {
		type Runtime = <IntegriteeKusama as Chain>::Runtime;
		type RuntimeEvent = <IntegriteeKusama as Chain>::RuntimeEvent;
		type Balances = <IntegriteeKusama as IntegriteeKusamaPallet>::Balances;
		type PolkadotXcm = <IntegriteeKusama as IntegriteeKusamaPallet>::PolkadotXcm;

		assert_ok!(<Balances as M<_>>::mint_into(&root_on_local, INITIAL_TEER_BALANCE));

		let weight = Runtime::query_xcm_weight(VersionedXcm::from(xcm1)).unwrap();
		PolkadotXcm::execute(RawOrigin::Root.into(), bx!(VersionedXcm::from(xcm2)), weight)
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
	Xcm(vec![
		SetAppendix(Xcm(vec![
			RefundSurplus,
			DepositAsset { assets: AssetFilter::Wild(WildAsset::All), beneficiary: ik_on_ahp_v5() },
		])),
		WithdrawAsset((Parent, Fungible(30000000000)).into()),
		InitiateTransfer {
			destination: ip_on_ahp_v5(),
			remote_fees: Some(ReserveDeposit(AssetFilter::Definite(
				Asset { id: Parent.into(), fun: Fungible(20000000000) }.into(),
			))),
			preserve_origin: true,
			assets: Default::default(),
			remote_xcm: ip_xcm(),
		},
	])
}

fn ip_xcm<Call>() -> Xcm<Call> {
	type RuntimeCall = <IntegriteePolkadot as Chain>::RuntimeCall;

	Xcm(vec![Transact {
		origin_kind: OriginKind::SovereignAccount,
		fallback_max_weight: None,
		call: RuntimeCall::System(frame_system::Call::remark { remark: "Hello".encode() })
			.encode()
			.into(),
	}])
}
