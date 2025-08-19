use crate::{
	tests::{
		assert_bridge_hub_kusama_message_accepted, assert_bridge_hub_polkadot_message_received,
		asset_hub_polkadot_location, bridge_hub_polkadot_location, bridged_ksm_at_ah_polkadot,
		create_foreign_on_ah_kusama, create_foreign_on_ah_polkadot, create_reserve_asset_on_ip,
		ik_sibling, ik_sibling_v5, ik_cousin_v5, ip_sibling_v5, set_up_pool_with_dot_on_ah_polkadot,
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
};
use kusama_polkadot_system_emulated_network::integritee_kusama_emulated_chain::genesis::AssetHubLocation;
use crate::tests::ip_sibling;

fn ik_sibling_account() -> AccountId {
	AssetHubKusama::sovereign_account_id_of(ik_sibling_v5())
}

fn ik_cousin_account() -> AccountId {
	AssetHubPolkadot::sovereign_account_of_parachain_on_other_global_consensus(
		KusamaId,
		IntegriteeKusama::para_id(),
	)
}

fn root_on_ik() -> AccountId {
	<IntegriteeKusama as Parachain>::LocationToAccountId::convert_location(&teer_on_self()).unwrap()
}

#[test]
fn ik_to_ip_xcm_works() {
	const KSM: u128 = 1_000_000_000_000;
	const DOT: u128 = 10_000_000_000;

	// Set XCM versions
	AssetHubKusama::force_xcm_version(asset_hub_polkadot_location(), XCM_VERSION);
	AssetHubPolkadot::force_xcm_version(ip_sibling_v5(), XCM_VERSION);
	AssetHubPolkadot::force_xcm_version(ik_cousin_v5(), XCM_VERSION);
	BridgeHubKusama::force_xcm_version(bridge_hub_polkadot_location(), XCM_VERSION);

	let root_on_ik = root_on_ik();
	let ik_sibling_acc = ik_sibling_account();
	let ik_cousin_acc = ik_cousin_account();

	// Fund accounts

	// Note: First we thought that these accounts need to exist on IP, but no.
	// let ip_treasury = integritee_polkadot_runtime::TreasuryAccount::get();
	// IntegriteePolkadot::fund_accounts(vec![
	// 	(ip_treasury.clone(), 100 * TEER),
	// 	(ik_cousin_acc.clone(), 100 * TEER),
	// ]);

	// fund the KAH's SA on KBH for paying bridge transport fees
	BridgeHubKusama::fund_para_sovereign(AssetHubKusama::para_id(), 10 * KSM);

	AssetHubKusama::fund_accounts(vec![(ik_sibling_acc, 100 * KSM)]);
	AssetHubPolkadot::fund_accounts(vec![(ik_cousin_acc.clone(), 100 * DOT)]);

	let ik_sibling = ik_sibling();
	create_foreign_on_ah_kusama(ik_sibling.clone(), false, vec![]);
	set_up_pool_with_ksm_on_ah_kusama(ik_sibling, true);

	let bridged_ksm_at_ah_polkadot = bridged_ksm_at_ah_polkadot();
	create_foreign_on_ah_polkadot(bridged_ksm_at_ah_polkadot.clone(), true, vec![]);
	set_up_pool_with_dot_on_ah_polkadot(bridged_ksm_at_ah_polkadot.clone(), true);

	create_foreign_on_ah_polkadot(ip_sibling(), false, vec![]);
	set_up_pool_with_dot_on_ah_polkadot(ip_sibling(), true);

	create_reserve_asset_on_ip(0, Parent.into(), true, vec![]);

	log::info!("Setup Done! Sending XCM.");

	let token_owner = Alice::get();
	let port_tokens_amount = 100 * TEER;

	let token_owner_balance_before_on_ik =
		IntegriteeKusama::account_data_of(token_owner.clone()).free;
	let token_owner_balance_before_on_ip =
		IntegriteePolkadot::account_data_of(token_owner.clone()).free;

	<IntegriteeKusama as TestExt>::execute_with(|| {
		type RuntimeEvent = <IntegriteeKusama as Chain>::RuntimeEvent;
		type Balances = <IntegriteeKusama as IntegriteeKusamaPallet>::Balances;
		type Porteer = <IntegriteeKusama as IntegriteeKusamaPallet>::Porteer;

		assert_ok!(<Balances as M<_>>::mint_into(&root_on_ik, 100 * TEER));

		Porteer::port_tokens(
			<IntegriteeKusama as Chain>::RuntimeOrigin::signed(token_owner.clone()),
			port_tokens_amount,
			Some(AssetHubLocation::get()),
		)
		.unwrap();

		assert_expected_events!(
			IntegriteeKusama,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
				RuntimeEvent::Porteer(pallet_porteer::Event::PortedTokens { .. }) => {},
			]
		);
	});

	// Assert Events on all hops until the IP

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
				) => {},
				RuntimeEvent::Porteer(pallet_porteer::Event::MintedPortedTokens {
					who, amount,
				}) => { who: *who == token_owner, amount: *amount == port_tokens_amount, },
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	AssetHubPolkadot::execute_with(|| {
		type RuntimeEvent = <AssetHubPolkadot as Chain>::RuntimeEvent;
		assert_expected_events!(
			AssetHubPolkadot,
			vec![
				RuntimeEvent::MessageQueue(
					pallet_message_queue::Event::Processed { success: true, .. }
				) => {},
				RuntimeEvent::ForeignAssets(pallet_assets::Event::Issued { owner, .. }) => {
					owner: *owner == token_owner,
				},
			]
		);
	});

	// Assert before and after balances

	// Note: XCM fees are taken from the Integritee's sovereign account
	// Todo: Assert Sovereign Account balances on the different chains

	assert_eq!(
		IntegriteeKusama::account_data_of(token_owner.clone()).free,
		token_owner_balance_before_on_ik - port_tokens_amount
	);

	assert_eq!(
		IntegriteePolkadot::account_data_of(token_owner.clone()).free,
		token_owner_balance_before_on_ip + port_tokens_amount
	);
}
