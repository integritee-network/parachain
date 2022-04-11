# Integritee Parachain

This is the repository to run Integritee as a parachain on Kusama and Rococo. It is forked from the [Cumulus](https://github.com/paritytech/cumulus) repository.

## Launch a local setup including a Relay Chain and a Parachain

Quick: see [polkadot-launch](https://github.com/paritytech/polkadot-launch.git)
```
node ../polkadot-launch/dist/cli.js launch-kusama-local-with-shell.json
```
or
```
polkadot-launch launch-kusama-local-with-shell.json
```
depending on how you installed `polkadot-launch`.

### Manually Launch a local Rococo Testnet

#### Launch a Rococo Relay Chain

First, check out which polkadot release to use in the .json files in the [polkadot-launch](./polkadot-launch) folder:
```json
{
	"relaychain": {
		"bin": "../../../bin/polkadot-0.9.18", // <-- release to use
    		"chain": "kusama-local",
        // --snip--
```
Then get and build `polkadot`:
```bash
# Compile Polkadot with the real overseer feature
git clone https://github.com/paritytech/polkadot
# Switch into the Polkadot directory
cd polkadot
# Checkout the proper commit
git checkout <release tag>  # release tag, just like in the example above: polkadot-0.9.18
cargo build --release
# If this fails due to  `feature `edition2021` is required` you have set an outdated rust default version. To fix this, you can simply run:
cargo +nightly-2022-01-31 build --release # or any other up-to-date nightly version

# Generate a raw chain spec
./target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode --raw > rococo-local-cfde.json
```
Start the `Alice` validator:
```bash
./target/release/polkadot --chain rococo-local-cfde.json --alice --validator --tmp
```
When the node starts you will see several log messages. Take note of the node's Peer ID in the logs. We will need it when connecting other nodes to it. It will look something like this:
```bash
Local node identity is: 12D3KooWGjsmVmZCM1jPtVNp6hRbbkGBK3LADYNniJAKJ19NUYiq
```
Start the `Bob` validator (in a second terminal):
```bash
./target/release/polkadot --chain rococo-local-cfde.json --validator --bob --bootnodes /ip4/<Alice IP>/tcp/30333/p2p/<Alice Peer ID> --tmp --port 30334 --ws-port 9945
```

More information can be found in the Substrate tutorial [Start a Relay Chain](https://docs.substrate.io/tutorials/v3/cumulus/start-relay/)


#### Register the Parachain
Go to [Polkadot Apps](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/parachains/parathreads). Register the parachain via
Network > Parachains sub-page, click on Parathreads tab and use the + ParaId button. After registering, the collator should start producing blocks when the next era starts.
![image](https://d33wubrfki0l68.cloudfront.net/ab3d311e37364a9706f2747b98b24fc259398152/2c4ba/static/4e9213b9ee2f65cc7fa9ccddd73679a3/c1b63/paraid-reserve.png
)


**Note:** You may also register the parachain via the `paraSudoWrapper` module (see [Launch the Parachain](launch-the-parachain) for generating genesis state and wasm):

![image](https://user-images.githubusercontent.com/2915325/99548884-1be13580-2987-11eb-9a8b-20be658d34f9.png)

You may need to add custom type overwrites in Settings -> Developer:
```
{
  "Address": "MultiAddress",
  "LookupSource": "MultiAddress",
  "ShardIdentifier": "Hash"
}
```

#### Launch the Parachain
More information can be found in https://docs.substrate.io/tutorials/v3/cumulus/connect-parachain/.

Build the parachain:
```bash
# Compile
git clone https://github.com/integritee-network/parachain
# Switch into the Integritee parachain directory & build it
cd parachain
cargo build --release

```
Generate custom parachain specification:
```bash
./target/release/integritee-collator build-spec --disable-default-bootnode > integritee-local-dev-plain.json
```
and update the `para_id` to the `para_id` you reserved on the relay-chain (default is `2000`):
```json
// --snip--
  "para_id": 2015, // <--- your already registered ID
  // --snip--
      "parachainInfo": {
        "parachainId": 2015 // <--- your already registered ID
      },
  // --snip--
```
Then generate a raw chain spec derived from your modified plain chain spec:

```bash
./target/release/parachain-collator build-spec --chain integritee-local-dev-plain.json --raw --disable-default-bootnode > integritee-local-dev.json
```
Export genesis and wasm states:
```bash
# Export genesis state
./target/release/integritee-collator export-genesis-state --chain integritee-local-dev.json > integritee-local-dev.state

# Export genesis wasm
./target/release/integritee-collator export-genesis-wasm --chain integritee-local-dev.json > integritee-local-dev.wasm
```
Start the first collator node:
```bash
./target/release/integritee-collator --alice --force-authoring --collator --tmp --chain integritee-local-dev.json --port 40335 --ws-port 9946 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30337 --ws-port 9981
```


### Deploy on rococo

Prepare genesis state and wasm as follows:

```bash
# Export genesis state
./target/release/integritee-collator export-genesis-state --chain integritee-rococo-local-dev > integritee-rococo-local-dev.state

# Export genesis wasm
./target/release/integritee-collator export-genesis-wasm --chain integritee-rococo-local-dev > integritee-rococo-local-dev.wasm

```
then propose the parachain on rococo relay-chain

run collator
```
integritee-collator \
        --collator \
        --chain integritee-rococo-local-dev \
        --rpc-cors all \
        --name integritee-rococo-collator-1 \
        -- --execution wasm --chain rococo

```

### Runtime upgrade
Two runtimes are contained in this repository. First, the shell-runtime, which has been extended compared to the upstream shell-runtime. It has some additional modules including sudo to facilitate a
runtime upgrade with the [sudo upgrade](https://substrate.dev/docs/en/tutorials/forkless-upgrade/sudo-upgrade) method. Second, it runs with the same executor instance as the integritee-runtime, such that an eventual upgrade is simpler to perform, i.e., only the runtime
needs to be upgraded whereas the client can remain the same. Hence, all modules revolving around aura have been included, which provide data the client needs.

#### Upgrade procedure
Prepare a local shell network and generate the `integritee-runtime` wasm blob, which contains the upgraded runtime to be executed after the runtime upgrade.
```bash
# launch local setup
node ../polkadot-launch/dist/cli.js polkadot-launch/launch-rococo-local-with-shell.json

# generate wasm blob
 ./target/release/integritee-collator export-genesis-wasm --chain integritee-rococo-local-dev > integritee-rococo-local-dev.wasm
```

After the parachain starts producing blocks, a runtime upgrade can be initiated via the polkadot-js/apps interface.

![image](./docs/sudo-set-code.png)

If successful, a `parachainSystem.validationFunctionStored` event is thrown followed by a `parachainSystem.validationFunctionApplied` event some blocks later. After this procedure, the `Teerex` module should be available in the extrinsics tab in polkadot-js/apps.

### Caveats
* Don't forget to enable file upload if you perform drag and drop for the `genesisHead` and `validationCode`. If it is not enabled, Polkadot-js will interpret the path as a string and won't complain but the registration will fail.
* Don't forget to add the argument `--chain integritee-rococo-local-dev.json` for the custom chain config. This argument is omitted in the [Cumulus Workshop](https://substrate.dev/cumulus-workshop/).
* The relay chain and the collator need to be about equally recent. This might require frequent rebasing of this repository on the corresponding release branch.

## Benchmark
The current weights have been benchmarked with the following reference hardware:
* Core(TM) i7-10875H
* 32GB of RAM
* NVMe SSD

The benchmarks are run with the following script:

```shell
./scripts/benchmark_all_pallets.sh
```


### More Resources
* Thorough Readme about Rococo and Collators in general in the original [repository](https://github.com/paritytech/cumulus) of this fork.
* Parachains Development in the [Polkadot Wiki](https://wiki.polkadot.network/docs/build-pdk)
