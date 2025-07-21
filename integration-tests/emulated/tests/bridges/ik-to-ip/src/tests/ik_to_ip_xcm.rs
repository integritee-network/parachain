use crate::{
	tests::{
		assert_bridge_hub_kusama_message_accepted, assert_bridge_hub_polkadot_message_received,
		asset_hub_polkadot_location, bridge_hub_polkadot_location, bridged_ksm_at_ah_polkadot,
		create_foreign_on_ah_kusama, create_foreign_on_ah_polkadot, create_reserve_asset_on_ip,
		ik_on_ahk, ik_on_ahk_v5, ik_on_ahp_v5, ip_on_ahp_v5, set_up_pool_with_dot_on_ah_polkadot,
		set_up_pool_with_ksm_on_ah_kusama, teer_on_self,
	},
	*,
};
use emulated_integration_tests_common::{
	impls::Parachain,
	xcm_emulator::{log, ConvertLocation},
};
use frame_support::{assert_ok, traits::fungible::Mutate as M};
use kusama_polkadot_system_emulated_network::{
	integritee_kusama_emulated_chain::integritee_kusama_runtime::{Alice, TEER},
	integritee_polkadot_emulated_chain::integritee_polkadot_runtime,
};

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

	<IntegriteeKusama as TestExt>::execute_with(|| {
		type RuntimeEvent = <IntegriteeKusama as Chain>::RuntimeEvent;
		type Balances = <IntegriteeKusama as IntegriteeKusamaPallet>::Balances;
		type Porteer = <IntegriteeKusama as IntegriteeKusamaPallet>::Porteer;

		assert_ok!(<Balances as M<_>>::mint_into(&root_on_local, INITIAL_TEER_BALANCE));

		Porteer::port_tokens(
			<IntegriteeKusama as Chain>::RuntimeOrigin::signed(Alice::get()),
			100 * TEER,
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

	// We can see the following logs, but these are expected, as the first 2 traders fail until
	// we get the right one:
	// 2025-07-19T18:42:17.124871Z ERROR xcm::weight: FixedRateOfFungible::buy_weight Failed to substract from payment amount=3275251420 error=AssetsInHolding { fungible: {AssetId(Location { parents: 1, interior: Here }): 20000000000}, non_fungible: {} }
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
