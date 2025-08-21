use crate::{
	tests::{
		assert_asset_hub_kusama_message_processed, assert_asset_hub_polkadot_message_processed,
		assert_bridge_hub_kusama_message_received, assert_bridge_hub_polkadot_message_accepted,
		integritee_bridge_setup::{ip_to_ik_bridge_setup, KSM},
	},
	*,
};
use emulated_integration_tests_common::xcm_emulator::log;
use kusama_polkadot_system_emulated_network::{
	integritee_kusama_emulated_chain::{
		genesis::AssetHubLocation,
		integritee_kusama_runtime::{integritee_common::porteer::burn_native_xcm, TEER},
	},
	integritee_polkadot_emulated_chain::integritee_polkadot_runtime::ExistentialDeposit,
};
use sp_core::sr25519;
use system_parachains_constants::genesis_presets::get_account_id_from_seed;
use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApi;

#[test]
fn ip_to_ik_xcm_works_without_forwarding_with_endowed_ip_beneficiary() {
	ip_to_ik_xcm(None, true)
}

#[test]
fn ip_to_ik_xcm_works_with_forwarding_with_endowed_ip_beneficiary() {
	ip_to_ik_xcm(Some(AssetHubLocation::get()), true)
}

#[test]
fn ip_to_ik_xcm_works_without_forwarding_with_nonexisting_ip_beneficiary() {
	ip_to_ik_xcm(None, false)
}

#[test]
fn ip_to_ik_xcm_works_with_forwarding_with_nonexisting_ip_beneficiary() {
	ip_to_ik_xcm(Some(AssetHubLocation::get()), false)
}

fn ip_to_ik_xcm(forward_teer_location: Option<Location>, fund_token_holder_on_ip: bool) {
	ip_to_ik_bridge_setup();

	log::info!("Setup Done! Sending XCM.");

	let token_owner = get_account_id_from_seed::<sr25519::Public>("teer_hodler");

	// Token Owner needs to have some KSM on AssetHub
	AssetHubKusama::fund_accounts(vec![(token_owner.clone(), 100 * KSM)]);

	let port_tokens_amount = 100 * TEER;

	let token_owner_balance_before_on_ip = 2 * port_tokens_amount;

	let token_owner_balance_before_on_ik: Balance = match fund_token_holder_on_ip {
		true => 100 * TEER,
		false => 0,
	};

	if token_owner_balance_before_on_ik > 0 {
		IntegriteeKusama::fund_accounts(vec![(
			token_owner.clone(),
			token_owner_balance_before_on_ik,
		)]);
	}

	IntegriteePolkadot::fund_accounts(vec![(
		token_owner.clone(),
		token_owner_balance_before_on_ip,
	)]);

	<IntegriteePolkadot as TestExt>::execute_with(|| {
		type RuntimeEvent = <IntegriteePolkadot as Chain>::RuntimeEvent;
		type Porteer = <IntegriteePolkadot as IntegriteePolkadotPallet>::Porteer;

		Porteer::port_tokens(
			<IntegriteePolkadot as Chain>::RuntimeOrigin::signed(token_owner.clone()),
			port_tokens_amount,
			forward_teer_location.clone(),
		)
		.unwrap();

		assert_expected_events!(
			IntegriteePolkadot,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
				RuntimeEvent::Porteer(pallet_porteer::Event::PortedTokens { .. }) => {},
			]
		);
	});

	// Assert Events on all hops until the IP

	assert_asset_hub_polkadot_message_processed();

	assert_bridge_hub_polkadot_message_accepted(true);
	assert_bridge_hub_kusama_message_received();

	assert_asset_hub_kusama_message_processed();

	assert_integritee_kusama_tokens_minted(
		token_owner.clone(),
		port_tokens_amount,
		forward_teer_location.is_some(),
	);

	// Assert before and after balances

	// Note: XCM fees are taken from the Integritee's sovereign account
	// Todo: Assert Sovereign Account balances on the different chains

	assert_eq!(
		IntegriteePolkadot::account_data_of(token_owner.clone()).free,
		token_owner_balance_before_on_ip - port_tokens_amount
	);

	if forward_teer_location.is_some() {
		assert_asset_hub_kusama_tokens_forwarded(token_owner.clone());

		if fund_token_holder_on_ip {
			let xcm = burn_native_xcm(Location::here(), 0, 0);
			let local_fee = query_integritee_kusama_xcm_execution_fee(xcm);

			assert_eq!(
				IntegriteeKusama::account_data_of(token_owner.clone()).free,
				token_owner_balance_before_on_ip - local_fee
			);
		} else {
			// Ensure that token forwarding respects the ED.
			assert_eq!(
				IntegriteeKusama::account_data_of(token_owner.clone()).free,
				ExistentialDeposit::get()
			);
		}
	} else {
		assert_eq!(
			IntegriteeKusama::account_data_of(token_owner.clone()).free,
			token_owner_balance_before_on_ik + port_tokens_amount
		);
	}
}

fn assert_asset_hub_kusama_tokens_forwarded(who: AccountId) {
	AssetHubKusama::execute_with(|| {
		type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;
		assert_expected_events!(
			AssetHubKusama,
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

fn assert_integritee_kusama_tokens_minted(
	beneficiary: AccountId,
	ported_tokens_amount: Balance,
	tokens_forwarded: bool,
) {
	// We can see the following logs, but these are expected, as the first 2 traders fail until
	// we get the right one:
	// 2025-07-19T18:42:17.124871Z ERROR xcm::weight: FixedRateOfFungible::buy_weight Failed to substract from payment amount=3275251420 error=AssetsInHolding { fungible: {AssetId(Location { parents: 1, interior: Here }): 20000000000}, non_fungible: {} }
	<IntegriteeKusama as TestExt>::execute_with(|| {
		type RuntimeEvent = <IntegriteeKusama as Chain>::RuntimeEvent;

		if tokens_forwarded {
			assert_expected_events!(
				IntegriteeKusama,
				vec![
					RuntimeEvent::MessageQueue(
						pallet_message_queue::Event::Processed { success: true, .. }
					) => {},
					RuntimeEvent::Porteer(pallet_porteer::Event::MintedPortedTokens {
						who, amount,
					}) => { who: *who == beneficiary, amount: *amount == ported_tokens_amount, },
					RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. }) => {},
				]
			);
		} else {
			assert_expected_events!(
				IntegriteeKusama,
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

fn query_integritee_kusama_xcm_execution_fee(xcm: Xcm<()>) -> Balance {
	<IntegriteeKusama as TestExt>::execute_with(|| {
		type Runtime = <IntegriteeKusama as Chain>::Runtime;

		let local_weight = Runtime::query_xcm_weight(VersionedXcm::V5(xcm)).unwrap();

		let local_fee = Runtime::query_weight_to_asset_fee(
			local_weight,
			VersionedAssetId::from(AssetId(Location::here())),
		)
		.unwrap();

		local_fee
	})
}
