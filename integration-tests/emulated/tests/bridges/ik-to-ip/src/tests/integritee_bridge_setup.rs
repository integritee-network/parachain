use crate::{
	tests::{
		asset_hub_polkadot_location, bridge_hub_polkadot_location, bridged_ksm_at_ah_polkadot,
		create_foreign_on_ah_kusama, create_foreign_on_ah_polkadot, create_reserve_asset_on_ip,
		ik_cousin_v5, ik_sibling, ik_sibling_v5, ip_sibling, ip_sibling_v5,
		set_up_pool_with_dot_on_ah_polkadot, set_up_pool_with_ksm_on_ah_kusama, teer_on_self,
	},
	*,
};

use emulated_integration_tests_common::{
    impls::Parachain,
    xcm_emulator::{ConvertLocation},
};
use kusama_polkadot_system_emulated_network::{
    integritee_kusama_emulated_chain::{
        integritee_kusama_runtime::TEER,
    },
};
use crate::tests::{bridged_dot_at_ah_kusama, create_reserve_asset_on_ik};

pub(crate) const KSM: u128 = 1_000_000_000_000;
pub(crate) const DOT: u128 = 10_000_000_000;

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

fn ip_local_root() -> AccountId {
	<IntegriteePolkadot as Parachain>::LocationToAccountId::convert_location(&teer_on_self())
		.unwrap()
}

fn setup_xcm_versions() {
	AssetHubKusama::force_xcm_version(asset_hub_polkadot_location(), XCM_VERSION);
	AssetHubPolkadot::force_xcm_version(ip_sibling_v5(), XCM_VERSION);
	AssetHubPolkadot::force_xcm_version(ik_cousin_v5(), XCM_VERSION);
	BridgeHubKusama::force_xcm_version(bridge_hub_polkadot_location(), XCM_VERSION);
}

pub(crate) fn ik_to_ip_bridge_setup() {
	setup_xcm_versions();

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

pub(crate) fn ip_to_ik_bridge_setup() {
	setup_xcm_versions();

	let ip_sibling_acc = ik_sibling_account();
	let ip_cousin_acc = ik_cousin_account();

	// Fund accounts

	// fund the KAH's SA on KBH for paying bridge transport fees
	BridgeHubPolkadot::fund_para_sovereign(AssetHubPolkadot::para_id(), 10 * DOT);

	AssetHubPolkadot::fund_accounts(vec![(ip_sibling_acc, 100 * DOT)]);
	AssetHubKusama::fund_accounts(vec![(ip_cousin_acc.clone(), 100 * KSM)]);

	IntegriteePolkadot::fund_accounts(vec![(ip_local_root(), 100 * TEER)]);

	create_foreign_on_ah_polkadot(ip_sibling(), false, vec![]);
	set_up_pool_with_dot_on_ah_polkadot(ip_sibling(), true);

	let bridged_dot_at_ah_kusama = bridged_dot_at_ah_kusama();
	create_foreign_on_ah_kusama(bridged_dot_at_ah_kusama.clone(), true, vec![]);
	set_up_pool_with_ksm_on_ah_kusama(bridged_dot_at_ah_kusama.clone(), true);

	create_foreign_on_ah_kusama(ik_sibling(), false, vec![]);
	set_up_pool_with_ksm_on_ah_kusama(ik_sibling(), true);

	create_reserve_asset_on_ik(0, Parent.into(), true, vec![]);
}
