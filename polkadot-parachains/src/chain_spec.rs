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
#![allow(clippy::inconsistent_digit_grouping)]

use cumulus_primitives_core::ParaId;
use integritee_kusama_runtime::TEER;
use integritee_parachains_common::AccountId;
use parity_scale_codec::{Decode, Encode};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::Ss58Codec, sr25519, Public};
use sp_keyring::AccountKeyring::{Alice, Bob, Dave, Eve};
use std::str::FromStr;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = staging_xcm::prelude::XCM_VERSION;

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

pub fn pub_sr25519(ss58: &str) -> sr25519::Public {
	public_from_ss58::<sr25519::Public>(ss58)
}

pub fn public_from_ss58<TPublic: Public + FromStr>(ss58: &str) -> TPublic
where
	// what's up with this weird trait bound??
	<TPublic as FromStr>::Err: std::fmt::Debug,
{
	TPublic::from_ss58check(ss58).expect("supply valid ss58!")
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum GenesisKeys {
	/// Use integriTEE keys.
	Integritee,
	/// Use Keys from the keyring for a test setup
	WellKnown,
	/// Use integriTEE dev keys.
	IntegriteeDev,
}

struct WellKnownKeys;

impl WellKnownKeys {
	fn root() -> AccountId {
		Alice.to_account_id()
	}

	fn endowed() -> Vec<AccountId> {
		vec![Alice.to_account_id(), Bob.to_account_id()]
	}

	fn authorities() -> Vec<AccountId> {
		vec![Dave.public().into(), Eve.public().into()]
	}
}

struct IntegriteeKeys;

impl IntegriteeKeys {
	fn root() -> AccountId {
		pub_sr25519("2JcYbKMfEGidntYP1LpPWsCMxFvUbjaPyipRViat4Sn5nuqm").into()
	}
	fn authorities() -> Vec<AccountId> {
		vec![
			pub_sr25519("5GZJjbPPD9u6NDgK1ApYmbyGs7EBX4HeEz2y2CD38YJxjvQH").into(),
			pub_sr25519("5CcSd1GZus6Jw7rP47LLqMMmtr2KeXCH6W11ZKk1LbCQ9dPY").into(),
			pub_sr25519("5FsECrDjBXrh5hXmN4PhQfNPbjYYwwW7edu2UQ8G5LR1JFuH").into(),
			pub_sr25519("5HBdSEnswkqm6eoHzzX5PCeKoC15CCy88vARrT8XMaRRuyaE").into(),
			pub_sr25519("5GGxVLYTXS7JZAwVzisdXbsugHSD6gtDb3AT3MVzih9jTLQT").into(),
		]
	}
}

struct IntegriteeDevKeys;

impl IntegriteeDevKeys {
	fn root() -> AccountId {
		pub_sr25519("5DMCERPw2yC6LBWNKzswHKLCtuYdtmgKssLJAsPGPVp6fuMY").into()
	}
	fn authorities() -> Vec<AccountId> {
		vec![
			pub_sr25519("5GZJjbPPD9u6NDgK1ApYmbyGs7EBX4HeEz2y2CD38YJxjvQH").into(),
			pub_sr25519("5CcSd1GZus6Jw7rP47LLqMMmtr2KeXCH6W11ZKk1LbCQ9dPY").into(),
			pub_sr25519("5FsECrDjBXrh5hXmN4PhQfNPbjYYwwW7edu2UQ8G5LR1JFuH").into(),
			pub_sr25519("5HBdSEnswkqm6eoHzzX5PCeKoC15CCy88vARrT8XMaRRuyaE").into(),
			pub_sr25519("5GGxVLYTXS7JZAwVzisdXbsugHSD6gtDb3AT3MVzih9jTLQT").into(),
		]
	}
}
pub fn integritee_chain_spec(
	para_id: ParaId,
	genesis_keys: GenesisKeys,
	relay_chain: RelayChain,
) -> ChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "TEER".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 13.into());

	let (root, endowed, authorities) = match genesis_keys {
		GenesisKeys::Integritee =>
			(IntegriteeKeys::root(), vec![IntegriteeKeys::root()], IntegriteeKeys::authorities()),
		GenesisKeys::WellKnown =>
			(WellKnownKeys::root(), WellKnownKeys::endowed(), WellKnownKeys::authorities()),
		GenesisKeys::IntegriteeDev => (
			IntegriteeDevKeys::root(),
			vec![IntegriteeDevKeys::root()],
			IntegriteeDevKeys::authorities(),
		),
	};

	#[allow(deprecated)]
	match relay_chain {
		RelayChain::Polkadot | RelayChain::PolkadotLocal => ChainSpec::builder(
			integritee_polkadot_runtime::WASM_BINARY
				.expect("WASM binary was not built, please build it!"),
			Extensions { relay_chain: relay_chain.to_string(), para_id: para_id.into() },
		),
		_ => ChainSpec::builder(
			integritee_kusama_runtime::WASM_BINARY
				.expect("WASM binary was not built, please build it!"),
			Extensions { relay_chain: relay_chain.to_string(), para_id: para_id.into() },
		),
	}
	.with_name("Integritee Network")
	.with_id(&format!("integritee-{}", relay_chain.to_string()))
	.with_protocol_id(relay_chain.protocol_id())
	.with_chain_type(relay_chain.chain_type())
	.with_properties(properties)
	.with_genesis_config_patch(integritee_genesis_config(
		root.clone(),
		endowed.clone(),
		authorities.clone(),
		para_id,
	))
	.build()
}

fn integritee_genesis_config(
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	invulnerables: Vec<AccountId>,
	parachain_id: ParaId,
) -> serde_json::Value {
	serde_json::json!({
		"collatorSelection": integritee_kusama_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.clone(),
			candidacy_bond: 500 * TEER,
			..Default::default()
		},
		"session": integritee_kusama_runtime::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|acc| {
					(
						acc.clone(),                         // account id
						acc.clone(),                         // validator id
						integritee_kusama_runtime::SessionKeys { aura: Decode::decode(&mut acc.encode().as_ref()).unwrap() }, // session keys
					)
				})
				.collect(),
		},
		"balances": {
			"balances": endowed_accounts.iter().cloned().map(|k| (k, 1_000 * TEER)).collect::<Vec<_>>(),
		},
		"parachainInfo": {
			"parachainId": parachain_id,
		},
		"polkadotXcm": {
			"safeXcmVersion": Some(SAFE_XCM_VERSION),
		},
		"teerex": {
			"allowSgxDebugMode": true,
			"allowSkippingAttestation": true,
		},
		"council": {
			"members": vec![root_key.clone()]
		},
		"technicalCommittee": {
			"members": vec![root_key.clone()]
		},
	})
}

pub enum RelayChain {
	RococoLocal,
	WestendLocal,
	KusamaLocal,
	PolkadotLocal,
	Rococo,
	Westend,
	Kusama,
	Polkadot,
	Moonbase,
	Paseo,
	PaseoLocal,
}

pub fn shell_rococo_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../chain-specs/integritee-rococo.json")[..])
}

pub fn shell_westend_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../chain-specs/integritee-westend.json")[..])
}

pub fn shell_kusama_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../chain-specs/integritee-kusama.json")[..])
}

pub fn shell_kusama_lease2_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../chain-specs/shell-kusama-lease2.json")[..])
}

pub fn shell_kusama_lease3_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../chain-specs/shell-kusama-lease3.json")[..])
}

pub fn shell_polkadot_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../chain-specs/integritee-polkadot.json")[..])
}

pub fn integritee_moonbase_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../chain-specs/integritee-moonbase.json")[..])
}

pub fn integritee_paseo_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../chain-specs/integritee-paseo.json")[..])
}

impl ToString for RelayChain {
	fn to_string(&self) -> String {
		match self {
			RelayChain::RococoLocal => "rococo-local".into(),
			RelayChain::WestendLocal => "westend-local".into(),
			RelayChain::KusamaLocal => "kusama-local".into(),
			RelayChain::PolkadotLocal => "polkadot-local".into(),
			RelayChain::Rococo => "rococo".into(),
			RelayChain::Westend => "westend".into(),
			RelayChain::Kusama => "kusama".into(),
			RelayChain::Polkadot => "polkadot".into(),
			RelayChain::Moonbase => "westend_moonbase_relay_testnet".into(),
			RelayChain::Paseo => "paseo".into(),
			RelayChain::PaseoLocal => "paseo-local".into(),
		}
	}
}

impl RelayChain {
	fn chain_type(&self) -> ChainType {
		match self {
			RelayChain::RococoLocal => ChainType::Local,
			RelayChain::WestendLocal => ChainType::Local,
			RelayChain::KusamaLocal => ChainType::Local,
			RelayChain::PolkadotLocal => ChainType::Local,
			RelayChain::PaseoLocal => ChainType::Local,
			RelayChain::Rococo => ChainType::Live,
			RelayChain::Westend => ChainType::Live,
			RelayChain::Kusama => ChainType::Live,
			RelayChain::Polkadot => ChainType::Live,
			RelayChain::Moonbase => ChainType::Live,
			RelayChain::Paseo => ChainType::Live,
		}
	}
	fn protocol_id(&self) -> &str {
		match self {
			RelayChain::RococoLocal => "teer-rl",
			RelayChain::WestendLocal => "teer-wl",
			RelayChain::KusamaLocal => "teer-kl",
			RelayChain::PolkadotLocal => "teer-pl",
			RelayChain::PaseoLocal => "teer-ol",
			RelayChain::Rococo => "teer-r",
			RelayChain::Westend => "teer-w",
			RelayChain::Kusama => "teer-k",
			RelayChain::Polkadot => "teer-p",
			RelayChain::Moonbase => "teer-m",
			RelayChain::Paseo => "teer-o",
		}
	}
}
