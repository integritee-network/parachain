// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::*;
use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApi;

// mod asset_transfers;
mod ik_to_ip_xcm;
mod integritee_bridge_setup;
mod ip_to_ik_xcm;
mod register_bridged_assets;
mod send_xcm;

pub(crate) fn teer_on_self() -> Location {
	Location::new(0, Here)
}

pub(crate) fn ik_sibling() -> xcm::v4::Location {
	xcm::v4::Location::new(1, [xcm::v4::Junction::Parachain(IntegriteeKusama::para_id().into())])
}

pub(crate) fn ik_sibling_v5() -> Location {
	Location::new(1, [Parachain(IntegriteeKusama::para_id().into())])
}

pub(crate) fn ik_cousin_v5() -> Location {
	Location::new(2, [GlobalConsensus(Kusama), Parachain(IntegriteeKusama::para_id().into())])
}

pub(crate) fn ip_sibling() -> xcm::v4::Location {
	xcm::v4::Location::new(1, [xcm::v4::Junction::Parachain(IntegriteePolkadot::para_id().into())])
}

pub(crate) fn ip_sibling_v5() -> Location {
	Location::new(1, [Parachain(2039)])
}

pub(crate) fn ip_cousin_v5() -> Location {
	Location::new(2, [GlobalConsensus(Polkadot), Parachain(IntegriteePolkadot::para_id().into())])
}

pub(crate) fn asset_hub_kusama_location() -> Location {
	Location::new(2, [GlobalConsensus(Kusama), Parachain(AssetHubKusama::para_id().into())])
}

pub(crate) fn asset_hub_polkadot_location() -> Location {
	Location::new(2, [GlobalConsensus(Polkadot), Parachain(AssetHubPolkadot::para_id().into())])
}

pub(crate) fn bridge_hub_kusama_location() -> Location {
	Location::new(2, [GlobalConsensus(Kusama), Parachain(BridgeHubKusama::para_id().into())])
}

pub(crate) fn bridge_hub_polkadot_location() -> Location {
	Location::new(2, [GlobalConsensus(Polkadot), Parachain(BridgeHubPolkadot::para_id().into())])
}

// KSM and wKSM
pub(crate) fn ksm_at_ah_kusama() -> xcm::v4::Location {
	xcm::v4::Parent.into()
}
pub(crate) fn bridged_ksm_at_ah_polkadot() -> xcm::v4::Location {
	xcm::v4::Location::new(2, [xcm::v4::Junction::GlobalConsensus(xcm::v4::NetworkId::Kusama)])
}

// wDOT
pub(crate) fn bridged_dot_at_ah_kusama() -> xcm::v4::Location {
	xcm::v4::Location::new(2, [xcm::v4::Junction::GlobalConsensus(xcm::v4::NetworkId::Polkadot)])
}

pub(crate) fn create_foreign_on_ah_kusama(
	id: xcm::v4::Location,
	sufficient: bool,
	prefund_accounts: Vec<(AccountId, u128)>,
) {
	let owner = AssetHubKusama::account_id_of(ALICE);
	let min = ASSET_MIN_BALANCE;
	AssetHubKusama::force_create_foreign_asset(id, owner, sufficient, min, prefund_accounts);
}

pub(crate) fn create_foreign_on_ah_polkadot(
	id: xcm::v4::Location,
	sufficient: bool,
	prefund_accounts: Vec<(AccountId, u128)>,
) {
	let owner = AssetHubPolkadot::account_id_of(ALICE);
	AssetHubPolkadot::force_create_foreign_asset(
		id,
		owner,
		sufficient,
		ASSET_MIN_BALANCE,
		prefund_accounts,
	);
}

pub(crate) fn create_reserve_asset_on_ip(
	id: u32,
	reserve_asset_location: Location,
	sufficient: bool,
	prefund_accounts: Vec<(AccountId, u128)>,
) {
	let owner = IntegriteePolkadot::account_id_of(ALICE);
	IntegriteePolkadot::force_create_asset(
		id,
		owner,
		sufficient,
		ASSET_MIN_BALANCE,
		prefund_accounts,
	);

	IntegriteePolkadot::execute_with(|| {
		type AssetRegistry = <IntegriteePolkadot as IntegriteePolkadotPallet>::AssetRegistry;

		let sudo_origin = <IntegriteePolkadot as Chain>::RuntimeOrigin::root();

		AssetRegistry::register_reserve_asset(sudo_origin, id, reserve_asset_location)
			.expect("Failed to register reserve asset");

		type RuntimeEvent = <IntegriteePolkadot as Chain>::RuntimeEvent;
		assert_expected_events!(
			IntegriteePolkadot,
			vec![
				RuntimeEvent::AssetRegistry(
				pallet_asset_registry::Event::ReserveAssetRegistered {
						asset_id, ..
					}
				) => { asset_id: *asset_id == id, },
			]
		);
	})
}

pub(crate) fn create_reserve_asset_on_ik(
	id: u32,
	reserve_asset_location: Location,
	sufficient: bool,
	prefund_accounts: Vec<(AccountId, u128)>,
) {
	let owner = IntegriteeKusama::account_id_of(ALICE);
	IntegriteeKusama::force_create_asset(
		id,
		owner,
		sufficient,
		ASSET_MIN_BALANCE,
		prefund_accounts,
	);

	IntegriteeKusama::execute_with(|| {
		type AssetRegistry = <IntegriteeKusama as IntegriteeKusamaPallet>::AssetRegistry;

		let sudo_origin = <IntegriteeKusama as Chain>::RuntimeOrigin::root();

		AssetRegistry::register_reserve_asset(sudo_origin, id, reserve_asset_location)
			.expect("Failed to register reserve asset");

		type RuntimeEvent = <IntegriteeKusama as Chain>::RuntimeEvent;
		assert_expected_events!(
			IntegriteeKusama,
			vec![
				RuntimeEvent::AssetRegistry(
				pallet_asset_registry::Event::ReserveAssetRegistered {
						asset_id, ..
					}
				) => { asset_id: *asset_id == id, },
			]
		);
	})
}

pub(crate) fn foreign_balance_on_ah_kusama(id: xcm::v4::Location, who: &AccountId) -> u128 {
	AssetHubKusama::execute_with(|| {
		type Assets = <AssetHubKusama as AssetHubKusamaPallet>::ForeignAssets;
		<Assets as Inspect<_>>::balance(id, who)
	})
}
pub(crate) fn foreign_balance_on_ah_polkadot(id: xcm::v4::Location, who: &AccountId) -> u128 {
	AssetHubPolkadot::execute_with(|| {
		type Assets = <AssetHubPolkadot as AssetHubPolkadotPallet>::ForeignAssets;
		<Assets as Inspect<_>>::balance(id, who)
	})
}

pub(crate) fn set_up_pool_with_dot_on_ah_polkadot(asset: xcm::v4::Location, is_foreign: bool) {
	let dot: xcm::v4::Location = xcm::v4::Parent.into();
	AssetHubPolkadot::execute_with(|| {
		type RuntimeEvent = <AssetHubPolkadot as Chain>::RuntimeEvent;
		let owner = AssetHubPolkadotSender::get();
		let signed_owner = <AssetHubPolkadot as Chain>::RuntimeOrigin::signed(owner.clone());

		if is_foreign {
			assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::ForeignAssets::mint(
				signed_owner.clone(),
				asset.clone(),
				owner.clone().into(),
				3_000_000_000_000,
			));
		} else {
			let asset_id = match asset.interior.last() {
				Some(xcm::v4::Junction::GeneralIndex(id)) => *id as u32,
				_ => unreachable!(),
			};
			assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::Assets::mint(
				signed_owner.clone(),
				asset_id.into(),
				owner.clone().into(),
				3_000_000_000_000,
			));
		}
		assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::AssetConversion::create_pool(
			signed_owner.clone(),
			Box::new(dot.clone()),
			Box::new(asset.clone()),
		));
		assert_expected_events!(
			AssetHubPolkadot,
			vec![
				RuntimeEvent::AssetConversion(pallet_asset_conversion::Event::PoolCreated { .. }) => {},
			]
		);
		assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::AssetConversion::add_liquidity(
			signed_owner.clone(),
			Box::new(dot),
			Box::new(asset),
			1_000_000_000_000,
			2_000_000_000_000,
			1,
			1,
			owner,
		));
		assert_expected_events!(
			AssetHubPolkadot,
			vec![
				RuntimeEvent::AssetConversion(pallet_asset_conversion::Event::LiquidityAdded {..}) => {},
			]
		);
	});
}

// set up pool
pub(crate) fn set_up_pool_with_ksm_on_ah_kusama(asset: xcm::v4::Location, is_foreign: bool) {
	let ksm: xcm::v4::Location = xcm::v4::Parent.into();
	AssetHubKusama::execute_with(|| {
		type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;
		let owner = AssetHubKusamaSender::get();
		let signed_owner = <AssetHubKusama as Chain>::RuntimeOrigin::signed(owner.clone());

		if is_foreign {
			assert_ok!(<AssetHubKusama as AssetHubKusamaPallet>::ForeignAssets::mint(
				signed_owner.clone(),
				asset.clone(),
				owner.clone().into(),
				3_000_000_000_000,
			));
		} else {
			let asset_id = match asset.interior.last() {
				Some(xcm::v4::Junction::GeneralIndex(id)) => *id as u32,
				_ => unreachable!(),
			};
			assert_ok!(<AssetHubKusama as AssetHubKusamaPallet>::Assets::mint(
				signed_owner.clone(),
				asset_id.into(),
				owner.clone().into(),
				3_000_000_000_000,
			));
		}
		assert_ok!(<AssetHubKusama as AssetHubKusamaPallet>::AssetConversion::create_pool(
			signed_owner.clone(),
			Box::new(ksm.clone()),
			Box::new(asset.clone()),
		));
		assert_expected_events!(
			AssetHubKusama,
			vec![
				RuntimeEvent::AssetConversion(pallet_asset_conversion::Event::PoolCreated { .. }) => {},
			]
		);
		assert_ok!(<AssetHubKusama as AssetHubKusamaPallet>::AssetConversion::add_liquidity(
			signed_owner.clone(),
			Box::new(ksm),
			Box::new(asset),
			1_000_000_000_000,
			2_000_000_000_000,
			1,
			1,
			owner,
		));
		assert_expected_events!(
			AssetHubKusama,
			vec![
				RuntimeEvent::AssetConversion(pallet_asset_conversion::Event::LiquidityAdded {..}) => {},
			]
		);
	});
}

pub(crate) fn send_assets_from_asset_hub_kusama(
	destination: Location,
	assets: Assets,
	fee_idx: u32,
) -> DispatchResult {
	let signed_origin =
		<AssetHubKusama as Chain>::RuntimeOrigin::signed(AssetHubKusamaSender::get());
	let beneficiary: Location =
		AccountId32Junction { network: None, id: AssetHubPolkadotReceiver::get().into() }.into();

	AssetHubKusama::execute_with(|| {
		<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::limited_reserve_transfer_assets(
			signed_origin,
			bx!(destination.into()),
			bx!(beneficiary.into()),
			bx!(assets.into()),
			fee_idx,
			WeightLimit::Unlimited,
		)
	})
}

pub(crate) fn assert_bridge_hub_kusama_message_accepted(expected_processed: bool) {
	BridgeHubKusama::execute_with(|| {
		type RuntimeEvent = <BridgeHubKusama as Chain>::RuntimeEvent;

		if expected_processed {
			assert_expected_events!(
				BridgeHubKusama,
				vec![
					// pay for bridge fees
					RuntimeEvent::Balances(pallet_balances::Event::Burned { .. }) => {},
					// message exported
					RuntimeEvent::BridgePolkadotMessages(
						pallet_bridge_messages::Event::MessageAccepted { .. }
					) => {},
					// message processed successfully
					RuntimeEvent::MessageQueue(
						pallet_message_queue::Event::Processed { success: true, .. }
					) => {},
				]
			);
		} else {
			assert_expected_events!(
				BridgeHubKusama,
				vec![
					RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed {
						success: false,
						..
					}) => {},
				]
			);
		}
	});
}

pub(crate) fn assert_bridge_hub_polkadot_message_accepted(expected_processed: bool) {
	BridgeHubPolkadot::execute_with(|| {
		type RuntimeEvent = <BridgeHubPolkadot as Chain>::RuntimeEvent;

		if expected_processed {
			assert_expected_events!(
				BridgeHubPolkadot,
				vec![
					// pay for bridge fees
					RuntimeEvent::Balances(pallet_balances::Event::Burned { .. }) => {},
					// message exported
					// Todo: This seems to be missing upstream.
					// RuntimeEvent::BridgePolkadotMessages(
					// 	pallet_bridge_messages::Event::MessageAccepted { .. }
					// ) => {},
					// message processed successfully
					RuntimeEvent::MessageQueue(
						pallet_message_queue::Event::Processed { success: true, .. }
					) => {},
				]
			);
		} else {
			assert_expected_events!(
				BridgeHubPolkadot,
				vec![
					RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed {
						success: false,
						..
					}) => {},
				]
			);
		}
	});
}

pub(crate) fn assert_bridge_hub_polkadot_message_received() {
	BridgeHubPolkadot::execute_with(|| {
		type RuntimeEvent = <BridgeHubPolkadot as Chain>::RuntimeEvent;
		assert_expected_events!(
			BridgeHubPolkadot,
			vec![
				// message sent to destination
				RuntimeEvent::XcmpQueue(
					cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. }
				) => {},
			]
		);
	})
}

pub(crate) fn assert_bridge_hub_kusama_message_received() {
	BridgeHubKusama::execute_with(|| {
		type RuntimeEvent = <BridgeHubKusama as Chain>::RuntimeEvent;
		assert_expected_events!(
			BridgeHubKusama,
			vec![
				// message sent to destination
				RuntimeEvent::XcmpQueue(
					cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. }
				) => {},
			]
		);
	})
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

fn query_integritee_kusama_xcm_execution_fee(xcm: Xcm<()>) -> Balance {
	<IntegriteeKusama as TestExt>::execute_with(|| {
		type Runtime = <IntegriteeKusama as Chain>::Runtime;

		let local_weight = Runtime::query_xcm_weight(VersionedXcm::V5(xcm)).unwrap();

		Runtime::query_weight_to_asset_fee(
			local_weight,
			VersionedAssetId::from(AssetId(Location::here())),
		)
		.unwrap()
	})
}

fn query_integritee_polkadot_xcm_execution_fee(xcm: Xcm<()>) -> Balance {
	<IntegriteePolkadot as TestExt>::execute_with(|| {
		type Runtime = <IntegriteePolkadot as Chain>::Runtime;

		let local_weight = Runtime::query_xcm_weight(VersionedXcm::V5(xcm)).unwrap();

		Runtime::query_weight_to_asset_fee(
			local_weight,
			VersionedAssetId::from(AssetId(Location::here())),
		)
		.unwrap()
	})
}
