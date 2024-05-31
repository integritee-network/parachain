//! Substrate Parachain Node Template CLI
//!
//! this file has no customizations for integritee runtimes. Upon upgrades of polkadot-sdk,
//! just overwrite from parachain_template

#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
#[macro_use]
mod service_shell;
mod cli;
mod command;
mod rpc;

fn main() -> sc_cli::Result<()> {
	command::run()
}
