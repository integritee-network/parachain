use crate::{
	tests::{
		assert_bridge_hub_kusama_message_accepted, assert_bridge_hub_polkadot_message_received,
		asset_hub_polkadot_location, bridge_hub_polkadot_location, bridged_ksm_at_ah_polkadot,
		create_foreign_on_ah_kusama, create_foreign_on_ah_polkadot, create_reserve_asset_on_ip,
		ik_cousin_v5, ik_sibling, ik_sibling_v5, ip_sibling, ip_sibling_v5,
		set_up_pool_with_dot_on_ah_polkadot, set_up_pool_with_ksm_on_ah_kusama, teer_on_self,
	},
	*,
};
use emulated_integration_tests_common::{
	impls::Parachain,
	xcm_emulator::{log, ConvertLocation},
};
use frame_support::ord_parameter_types;
use kusama_polkadot_system_emulated_network::{
	integritee_kusama_emulated_chain::{
		genesis::AssetHubLocation, integritee_kusama_runtime::TEER,
	},
	integritee_polkadot_emulated_chain::integritee_polkadot_runtime::ExistentialDeposit,
};
use sp_core::{hex2array, sr25519};
use system_parachains_constants::genesis_presets::get_account_id_from_seed;

fn ik_sibling_account() -> AccountId {
	AssetHubKusama::sovereign_account_id_of(ik_sibling_v5())
}

fn ik_cousin_account() -> AccountId {
	AssetHubPolkadot::sovereign_account_of_parachain_on_other_global_consensus(
		KusamaId,
		IntegriteeKusama::para_id(),
	)
}

fn ik_local_root() -> AccountId {
	<IntegriteeKusama as Parachain>::LocationToAccountId::convert_location(&teer_on_self()).unwrap()
}

#[test]
fn ik_to_ip_xcm_works_without_forwarding() {
	ik_to_pk_xcm(None, true)
}

#[test]
fn ik_to_ip_xcm_works_with_forwarding() {
	ik_to_pk_xcm(Some(AssetHubLocation::get()), true)
}

#[test]
fn ik_to_ip_xcm_works_without_forwarding_with_nonexisting_ip_beneficiary() {
	ik_to_pk_xcm(None, false)
}

#[test]
fn ik_to_ip_xcm_works_with_forwarding_with_nonexisting_ip_beneficiary() {
	ik_to_pk_xcm(Some(AssetHubLocation::get()), false)
}

const KSM: u128 = 1_000_000_000_000;
const DOT: u128 = 10_000_000_000;

fn ik_to_pk_xcm(forward_teer_location: Option<Location>, fund_token_holder_on_ip: bool) {
	ik_to_ip_bridge_setup();

	log::info!("Setup Done! Sending XCM.");

	let token_owner = get_account_id_from_seed::<sr25519::Public>("teer_hodler");

	// Token Owner needs to have some DOT on AssetHub
	AssetHubPolkadot::fund_accounts(vec![(token_owner.clone(), 100 * DOT)]);

	let port_tokens_amount = 100 * TEER;

	let token_owner_balance_before_on_ik = 2 * port_tokens_amount;

	let token_owner_balance_before_on_ip: Balance = match fund_token_holder_on_ip {
		true => 100 * TEER,
		false => 0,
	};

	if token_owner_balance_before_on_ip > 0 {
		IntegriteePolkadot::fund_accounts(vec![(
			token_owner.clone(),
			token_owner_balance_before_on_ip,
		)]);
	}

	IntegriteeKusama::fund_accounts(vec![(token_owner.clone(), token_owner_balance_before_on_ik)]);

	<IntegriteeKusama as TestExt>::execute_with(|| {
		type RuntimeEvent = <IntegriteeKusama as Chain>::RuntimeEvent;
		type Porteer = <IntegriteeKusama as IntegriteeKusamaPallet>::Porteer;

		Porteer::port_tokens(
			<IntegriteeKusama as Chain>::RuntimeOrigin::signed(token_owner.clone()),
			port_tokens_amount,
			forward_teer_location.clone(),
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

	assert_asset_hub_kusama_message_processed();

	assert_bridge_hub_kusama_message_accepted(true);
	assert_bridge_hub_polkadot_message_received();

	assert_asset_hub_polkadot_message_processed();

	assert_integritee_polkadot_tokens_minted(
		token_owner.clone(),
		port_tokens_amount,
		forward_teer_location.is_some(),
	);

	// Assert before and after balances

	// Note: XCM fees are taken from the Integritee's sovereign account
	// Todo: Assert Sovereign Account balances on the different chains

	assert_eq!(
		IntegriteeKusama::account_data_of(token_owner.clone()).free,
		token_owner_balance_before_on_ik - port_tokens_amount
	);

	if forward_teer_location.is_some() {
		assert_asset_hub_polkadot_tokens_forwarded(token_owner.clone());

		// The forwarder makes sure that there are at least 2 ED on the account, but then some fees have to be paid.
		if fund_token_holder_on_ip {
			// Todo: how to compute the local fees
			// assert_eq!(
			// 	IntegriteePolkadot::account_data_of(token_owner.clone()).free,
			// 	token_owner_balance_before_on_ip
			// );
		} else {
			// Ensure that token forwarding respects the ED.
			assert!(
				IntegriteePolkadot::account_data_of(token_owner.clone()).free <
					2 * ExistentialDeposit::get()
			);
			assert!(
				IntegriteePolkadot::account_data_of(token_owner.clone()).free >
					ExistentialDeposit::get()
			);
		}
	} else {
		assert_eq!(
			IntegriteePolkadot::account_data_of(token_owner.clone()).free,
			token_owner_balance_before_on_ip + port_tokens_amount
		);
	}
}

fn ik_to_ip_bridge_setup() {
	// Set XCM versions
	AssetHubKusama::force_xcm_version(asset_hub_polkadot_location(), XCM_VERSION);
	AssetHubPolkadot::force_xcm_version(ip_sibling_v5(), XCM_VERSION);
	AssetHubPolkadot::force_xcm_version(ik_cousin_v5(), XCM_VERSION);
	BridgeHubKusama::force_xcm_version(bridge_hub_polkadot_location(), XCM_VERSION);

	let ik_sibling_acc = ik_sibling_account();
	let ik_cousin_acc = ik_cousin_account();

	// Fund accounts

	// fund the KAH's SA on KBH for paying bridge transport fees
	BridgeHubKusama::fund_para_sovereign(AssetHubKusama::para_id(), 10 * KSM);

	AssetHubKusama::fund_accounts(vec![(ik_sibling_acc, 100 * KSM)]);
	AssetHubPolkadot::fund_accounts(vec![(ik_cousin_acc.clone(), 100 * DOT)]);

	IntegriteeKusama::fund_accounts(vec![(ik_local_root(), 100 * TEER)]);

	create_foreign_on_ah_kusama(ik_sibling(), false, vec![]);
	set_up_pool_with_ksm_on_ah_kusama(ik_sibling(), true);

	let bridged_ksm_at_ah_polkadot = bridged_ksm_at_ah_polkadot();
	create_foreign_on_ah_polkadot(bridged_ksm_at_ah_polkadot.clone(), true, vec![]);
	set_up_pool_with_dot_on_ah_polkadot(bridged_ksm_at_ah_polkadot.clone(), true);

	create_foreign_on_ah_polkadot(ip_sibling(), false, vec![]);
	set_up_pool_with_dot_on_ah_polkadot(ip_sibling(), true);

	create_reserve_asset_on_ip(0, Parent.into(), true, vec![]);
}

fn assert_asset_hub_kusama_message_processed() {
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
}

fn assert_asset_hub_polkadot_message_processed() {
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
}

fn assert_asset_hub_polkadot_tokens_forwarded(who: AccountId) {
	AssetHubPolkadot::execute_with(|| {
		type RuntimeEvent = <AssetHubPolkadot as Chain>::RuntimeEvent;
		assert_expected_events!(
			AssetHubPolkadot,
			vec![
				RuntimeEvent::MessageQueue(
					pallet_message_queue::Event::Processed { success: true, .. }
				) => {},
				RuntimeEvent::ForeignAssets(pallet_assets::Event::Issued { owner, .. }) => {
					owner: *owner == who,
				},
			]
		);
	});
}

fn assert_integritee_polkadot_tokens_minted(
	beneficiary: AccountId,
	ported_tokens_amount: Balance,
	tokens_forwarded: bool,
) {
	// We can see the following logs, but these are expected, as the first 2 traders fail until
	// we get the right one:
	// 2025-07-19T18:42:17.124871Z ERROR xcm::weight: FixedRateOfFungible::buy_weight Failed to substract from payment amount=3275251420 error=AssetsInHolding { fungible: {AssetId(Location { parents: 1, interior: Here }): 20000000000}, non_fungible: {} }
	<IntegriteePolkadot as TestExt>::execute_with(|| {
		type RuntimeEvent = <IntegriteePolkadot as Chain>::RuntimeEvent;

		if tokens_forwarded {
			assert_expected_events!(
				IntegriteePolkadot,
				vec![
					RuntimeEvent::MessageQueue(
						pallet_message_queue::Event::Processed { success: true, .. }
					) => {},
					RuntimeEvent::Porteer(pallet_porteer::Event::MintedPortedTokens {
						who, amount,
					}) => { who: *who == beneficiary, amount: *amount == ported_tokens_amount, },
					RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
				]
			);
		} else {
			assert_expected_events!(
				IntegriteePolkadot,
				vec![
					RuntimeEvent::MessageQueue(
						pallet_message_queue::Event::Processed { success: true, .. }
					) => {},
					RuntimeEvent::Porteer(pallet_porteer::Event::MintedPortedTokens {
						who, amount,
					}) => { who: *who == beneficiary, amount: *amount == ported_tokens_amount, },
				]
			);
		}
	});
}
