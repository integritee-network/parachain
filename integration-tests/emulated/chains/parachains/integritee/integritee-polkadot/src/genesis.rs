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

// Substrate

// Cumulus
use emulated_integration_tests_common::{
	accounts, build_genesis_storage, collators, SAFE_XCM_VERSION,
};
use parachains_common::Balance;
use staging_xcm::prelude::*;

pub const PARA_ID: u32 = 2039;
pub const ED: Balance = integritee_polkadot_runtime::ExistentialDeposit::get();

frame_support::parameter_types! {
	pub UniversalLocation: InteriorLocation = [GlobalConsensus(Polkadot), Parachain(PARA_ID)].into();
}

pub fn genesis() -> sp_core::storage::Storage {
	let alice = sp_keyring::Sr25519Keyring::Alice.to_account_id();

	let genesis_config = integritee_polkadot_runtime::RuntimeGenesisConfig {
		system: integritee_polkadot_runtime::SystemConfig::default(),
		balances: integritee_polkadot_runtime::BalancesConfig {
			balances: accounts::init_balances()
				.iter()
				.cloned()
				.map(|k| (k, ED * 4096 * 4096))
				.collect(),
			dev_accounts: None,
		},
		democracy: Default::default(),
		council: integritee_polkadot_runtime::CouncilConfig {
			members: vec![alice.clone()],
			..Default::default()
		},
		parachain_info: integritee_polkadot_runtime::ParachainInfoConfig {
			parachain_id: PARA_ID.into(),
			..Default::default()
		},
		collator_selection: integritee_polkadot_runtime::CollatorSelectionConfig {
			invulnerables: collators::invulnerables().iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: ED * 16,
			..Default::default()
		},
		session: integritee_polkadot_runtime::SessionConfig {
			keys: collators::invulnerables()
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                                       // account id
						acc,                                               // validator id
						integritee_polkadot_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
			..Default::default()
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		polkadot_xcm: integritee_polkadot_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		assets: Default::default(),
		pool_assets: Default::default(),
		teerex: integritee_polkadot_runtime::TeerexConfig {
			allow_sgx_debug_mode: true,
			allow_skipping_attestation: true,
			_config: Default::default(),
		},
		technical_committee: integritee_polkadot_runtime::TechnicalCommitteeConfig {
			members: vec![alice],
			..Default::default()
		},
		porteer: integritee_polkadot_runtime::PorteerConfig {
			porteer_config: pallet_porteer::PorteerConfig {
				send_enabled: true,
				receive_enabled: true,
			},
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(
		&genesis_config,
		integritee_polkadot_runtime::WASM_BINARY
			.expect("WASM binary was not built, please build it!"),
	)
}
