use crate::{
	tests::{
		asset_hub_polkadot_location, create_foreign_on_ah_kusama, ik_on_ahk, ik_on_ahk_v5,
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
use sp_runtime::traits::Bounded;
use xcm::{
	latest::AssetTransferFilter::ReserveDeposit, v3::Error::WeightLimitReached,
	v5::AssetTransferFilter::Teleport,
};
use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApi;

fn ik_on_ahk_account() -> AccountId {
	// Todo: replace with asset_hub_kusama_runtime, but the emulated network doesn't expose it.
	integritee_kusama_runtime::xcm_config::LocationToAccountId::convert_location(&ik_on_ahk_v5())
		.unwrap()
}

#[test]
fn ik_to_ip_xcm_works() {
	const INITIAL_TEER_BALANCE: u128 = 100 * TEER;
	const ONE_KSM: u128 = 1_000_000_000_000;
	const INITIAL_KSM_BALANCE: u128 = 100 * ONE_KSM;
	let recipient = AccountId::new([5u8; 32]);

	let ahk = (Parent, Parachain(1000));

	let root_on_local =
		integritee_kusama_runtime::xcm_config::LocationToAccountId::convert_location(
			&teer_on_self(),
		)
		.unwrap();

	let ik_on_ahk_acc = ik_on_ahk_account();

	<AssetHubKusama as TestExt>::execute_with(|| {
		type Assets = <AssetHubKusama as AssetHubKusamaPallet>::Assets;
		type Balances = <AssetHubKusama as AssetHubKusamaPallet>::Balances;

		assert_ok!(<Balances as M<_>>::mint_into(&ik_on_ahk_acc, INITIAL_KSM_BALANCE));

	});

	create_foreign_on_ah_kusama(ik_on_ahk(), false, vec![(ik_on_ahk_account(), 100 * TEER)]);

	set_up_pool_with_ksm_on_ah_kusama(ik_on_ahk(), true);

	<IntegriteeKusama as TestExt>::execute_with(|| {
		type Runtime = <IntegriteeKusama as Chain>::Runtime;
		type RuntimeEvent = <IntegriteeKusama as Chain>::RuntimeEvent;
		type Balances = <IntegriteeKusama as IntegriteeKusamaPallet>::Balances;
		assert_ok!(<Balances as M<_>>::mint_into(&root_on_local, INITIAL_TEER_BALANCE));

		let weight = Runtime::query_xcm_weight(VersionedXcm::from(ik_xcm())).unwrap();
		<IntegriteeKusama as IntegriteeKusamaPallet>::PolkadotXcm::execute(
			RawOrigin::Root.into(),
			bx!(VersionedXcm::from(ik_xcm())),
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
				// Todo: This fails with the following logs:
				// 2025-07-02T17:44:35.387562Z  INFO events::IntegriteeKusama: PolkadotXcm(Event::Attempted { outcome: Complete { used: Weight { ref_time: 5000000, proof_size: 5120 } } })
				// 2025-07-02T17:44:35.388641Z DEBUG xcm::weight: Failed to substract from payment error=AssetsInHolding { fungible: {AssetId(Location { parents: 1, interior: X1([Parachain(2015)]) }): 67698188749358}, non_fungible: {} }
				// 2025-07-02T17:44:35.389271Z DEBUG xcm::process: XCM execution failed at instruction index=1 error=TooExpensive
				// 2025-07-02T17:44:35.389874Z  INFO xcm::hrmp::KusamaMockNet: Horizontal messages processed by para_id 1000: [(Id(2015), 6, "0x00050c02040101007d1f000b2e0a7e36923d300101007d1f000b2e0a7e36923d2ca89957eec6987309afe892785bbcdc1c00decc106f94c3216b71a2dff0def246")]
					pallet_message_queue::Event::Processed { success: true, .. }
				) => {},
			]
		);
	});
}

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
			remote_xcm: Default::default(),
		},
	])
}

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
			remote_xcm: Default::default(),
		},
	])
}
