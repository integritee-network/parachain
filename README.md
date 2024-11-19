# Integritee Parachain

This is the repository to run Integritee as a parachain on Kusama and Rococo. It is forked from
the [Cumulus](https://github.com/paritytech/cumulus) repository.

## Bump Dependencies

1. Run these commands with the suitable polkadot version.

```bash
diener update --cumulus --branch polkadot-v0.9.xx
diener update --substrate --branch polkadot-v0.9.xx
diener update --polkadot --branch release-v0.9.xx
```

2. Search and replace `integritee-network/` dependencies to point to new `polkadot-v0.9.xx` branch.
3. Search and replace `orml` dependencies to point to new `polkadot-v0.9.xx` branch.

## Launch a local setup including a Relay Chain and a Parachain

### Zombienet

1. Download zombienet from the releases in the [zombienet](https://github.com/paritytech/zombienet) repository.
2. Copy it to `~/.local/bin/`
3. Launch it in your terminal

```
zombienet-linux-x64 spawn --provider native zombienet/rococo-local-with-integritee-and-shell.toml
```

**Note:** You can also set a chain-spec path to the zombienet config, but the config param is named `chain_spec_path`.

### Manually launch a local Rococo Testnet

Follow the steps provided in https://docs.substrate.io/tutorials/v3/cumulus/start-relay/

But keep the following in mind:

- Our chain has a paraid of 2015 on Rococo and Kusama (
  see [polkadot-parachains/src/command.rs#44-49](/polkadot-parachains/src/command.rs#44-49))
- For testing on rococo-local use the chain spec `integritee-rococo-local-dev`
- More chain specs can be found in [polkadot-parachains/src/command.rs](/polkadot-parachains/src/command.rs)

## Runtime upgrade

Two runtimes are contained in this repository. First, the shell-runtime, which has been extended compared to the
upstream shell-runtime. It has some additional modules including sudo to facilitate a
runtime upgrade with the [sudo upgrade](https://substrate.dev/docs/en/tutorials/forkless-upgrade/sudo-upgrade) method.
Second, it runs with the same executor instance as the integritee-runtime, such that an eventual upgrade is simpler to
perform, i.e., only the runtime
needs to be upgraded whereas the client can remain the same. Hence, all modules revolving around aura have been
included, which provide data the client needs.

### Upgrade procedure

Prepare a local shell network and generate the `integritee-runtime` wasm blob, which contains the upgraded runtime to be
executed after the runtime upgrade.

```bash
# spawn local setup (NOTE: the shell-runtime parachain id needs to be changed to match the integritee-kusama's.)
zombienet-linux-x64 spawn --provider native zombienet/rococo-local-with-shell.toml

# generate wasm blob
 ./target/release/integritee-collator export-genesis-wasm --chain integritee-rococo-local-dev > integritee-rococo-local-dev.wasm
```

After the parachain starts producing blocks, a runtime upgrade can be initiated via the polkadot-js/apps interface.

![image](./docs/sudo-set-code.png)

If successful, a `parachainSystem.validationFunctionStored` event is thrown followed by a
`parachainSystem.validationFunctionApplied` event some blocks later. After this procedure, the `Teerex` module should be
available in the extrinsics tab in polkadot-js/apps.

### Caveats

* Don't forget to enable file upload if you perform drag and drop for the `genesisHead` and `validationCode`. If it is
  not enabled, Polkadot-js will interpret the path as a string and won't complain but the registration will fail.
* Don't forget to add the argument `--chain integritee-rococo-local-dev` for the custom chain config. This argument is
  omitted in the [Cumulus Workshop](https://substrate.dev/cumulus-workshop/).
* The relay chain and the collator need to be about equally recent. This might require frequent rebasing of this
  repository on the corresponding release branch.

## Benchmark

The current weights have been benchmarked with the following reference hardware:

* Core(TM) i7-10875H
* 32GB of RAM
* NVMe SSD

The benchmarks are run with the following script:

```shell
./scripts/benchmark_all_pallets.sh
```

## state migrations try-runtime

```
cargo build --release --features try-runtime
try-runtime --runtime ./target/release/wbuild/integritee-runtime/integritee_runtime.compact.compressed.wasm on-runtime-upgrade --checks=pre-and-post live --uri wss://kusama.api.integritee.network:443
```

## testing with chopsticks

To test runtime upgrades

```
nvm use 20
npx @acala-network/chopsticks@latest --config integritee-kusama  --wasm-override ./target/release/wbuild/integritee-runtime/integritee_runtime.compact.compressed.wasm
```

to test XCM

```
npx @acala-network/chopsticks@latest xcm --p=karura --p=integritee-kusama
```

see other options in chopsticks help

## More Resources

* Thorough Readme about Rococo and Collators in general in the
  original [repository](https://github.com/paritytech/cumulus) of this fork.
* Parachains Development in the [Polkadot Wiki](https://wiki.polkadot.network/docs/build-pdk)
* encointer parachain readme: https://github.com/encointer/encointer-parachain
