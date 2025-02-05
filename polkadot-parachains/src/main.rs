//! Substrate Parachain Node Template CLI
//!
//! this file has no customizations for integritee runtimes. Upon upgrades of polkadot-sdk,
//! just overwrite from parachain_template

#![warn(missing_docs)]

mod chain_spec;
mod cli;
mod command;
mod rpc;
mod service;

fn main() -> sc_cli::Result<()> {
	command::run()
}
