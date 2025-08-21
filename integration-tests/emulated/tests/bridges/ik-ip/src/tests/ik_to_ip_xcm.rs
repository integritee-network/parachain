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

//! Tests regarding the bridge from IK to IP.

use crate::{
	tests::{
		assert_asset_hub_kusama_message_processed, assert_asset_hub_polkadot_message_processed,
		assert_bridge_hub_kusama_message_accepted, assert_bridge_hub_polkadot_message_received,
		integritee_bridge_setup::{ik_to_ip_bridge_setup, DOT},
		query_integritee_kusama_xcm_execution_fee, query_integritee_polkadot_xcm_execution_fee,
	},
	*,
};
use emulated_integration_tests_common::xcm_emulator::log;
use kusama_polkadot_system_emulated_network::{
	integritee_kusama_emulated_chain::{
		genesis::AssetHubLocation,
		integritee_kusama_runtime::{integritee_common::xcm_helpers::burn_native_xcm, TEER},
	},
	integritee_polkadot_emulated_chain::integritee_polkadot_runtime::ExistentialDeposit,
};
use sp_core::sr25519;
use system_parachains_constants::genesis_presets::get_account_id_from_seed;

#[test]
fn ik_to_ip_xcm_works_without_forwarding_with_endowed_ip_beneficiary() {
	ik_to_pk_xcm(None, true)
}

#[test]
fn ik_to_ip_xcm_works_with_forwarding_with_endowed_ip_beneficiary() {
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
				RuntimeEvent::Porteer(pallet_porteer::Event::PortedTokens { .. }) => {},
				RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. }) => {},
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

	// Todo: Assert Sovereign Account balances on the different chains
	// https://github.com/integritee-network/parachain/issues/337

	let xcm = burn_native_xcm(Location::here(), 0, 0);
	let local_fee = query_integritee_kusama_xcm_execution_fee(xcm);
	let ah_sibling_fee = query_integritee_kusama_ah_sibling_remote_fee();
	assert_eq!(
		IntegriteeKusama::account_data_of(token_owner.clone()).free,
		token_owner_balance_before_on_ik - port_tokens_amount - local_fee - ah_sibling_fee
	);

	if forward_teer_location.is_some() {
		assert_asset_hub_polkadot_tokens_forwarded(token_owner.clone());

		if fund_token_holder_on_ip {
			let xcm = burn_native_xcm(Location::here(), 0, 0);
			let local_fee = query_integritee_polkadot_xcm_execution_fee(xcm);

			assert_eq!(
				IntegriteePolkadot::account_data_of(token_owner.clone()).free,
				token_owner_balance_before_on_ip - local_fee
			);
		} else {
			// Ensure that token forwarding respects the ED.
			assert_eq!(
				IntegriteePolkadot::account_data_of(token_owner.clone()).free,
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
					RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. }) => {},
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

fn query_integritee_kusama_ah_sibling_remote_fee() -> Balance {
	<IntegriteeKusama as TestExt>::execute_with(|| {
		type Porteer = <IntegriteeKusama as IntegriteeKusamaPallet>::Porteer;
		Porteer::xcm_fee_config().hop1
	})
}
