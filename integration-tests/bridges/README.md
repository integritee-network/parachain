# Bridges Tests for Local Polkadot <> Kusama Bridge

This folder contains zombienet based integration test for both onchain and offchain bridges code.
The tests are designed to be run manually.

## setup

```
mkdir -p ~/local_bridge_testing/bin
cd ~/local_bridge_testing/bin
wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-stable2412/polkadot
wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-stable2412/polkadot-execute-worker
wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-stable2412/polkadot-prepare-worker
wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-stable2412/polkadot-parachain
wget https://github.com/paritytech/zombienet/releases/download/v1.3.133/zombienet-linux-x64 -O zombienet
chmod +x polkadot*
chmod +x zombienet
chmod +x integritee-collator

yarn global add @polkadot/api-cli

# in another folder of your choice, build the relay
cd
git clone https://github.com/paritytech/parity-bridges-common.git
cd parity-bridges-common
cargo +nightly build -p substrate-relay --release
cp ./target/release/substrate-relay ~/local_bridge_testing/bin/

# in runtimes repo, build chainspec generator with sudo:
cd 
git clone https://github.com/polkadot-fellows/runtimes.git
cd runtimes
git checkout v1.6.0
# actually, a patch is needed, use this instead until the patch is released: https://github.com/encointer/runtimes/tree/ab/trusted-aliaser-patch
git apply ./integration-tests/bridges/sudo-relay.patch
cargo +nightly build --release -p chain-spec-generator --no-default-features --features fast-runtime,polkadot,kusama,bridge-hub-kusama,bridge-hub-polkadot,asset-hub-kusama,asset-hub-polkadot

# in another folder of your choice, build the integritee-collator with sudo
cd
git clone https://github.com/integritee-network/parachain.git
cd parachain
git apply ./integration-tests/bridges/sudo-integritee.patch
cargo build --release
cp ./target/release/integritee-collator ~/local_bridge_testing/bin/
```

## automated testing (manually triggered)

After that, you can run `./run-tests.sh <test_name>` command in `runtimes/integration-tests/bridges`
.
E.g. `./run-test.sh 0001-polkadot-kusama-asset-transfer`.
or
E.g. `FRAMEWORK_REPO_PATH=/home/username/polkadot-sdk ./run-test.sh 0001-polkadot-kusama-asset-transfer`.

## manual testing

If you'd like to interact with the local test networks, run this instead

```
./run-test.sh 0000-manual
``` 

then you can point your browser to

* Integritee Network (Kusama) https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9144#
* Integritee Network (Polkadot) https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9244#
* Asset Hub Kusama https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9010#
* Asset Hub Polkadot https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9910#

### a few calls to try

XCMv5

* send 1 KSM from KAH to IK(
  Alice): https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9010#/extrinsics/decode/0x1f0b050101007d1f0500010100d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0504010000070010a5d4e80000000000
* send 1 DOT from PAH to IP(
  Alice) https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9910#/extrinsics/decode/0x1f0b05010100dd1f0500010100d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d05040100000700e40b54020000000000

### interventions

this setup is brittle and it can happen that not all setup calls succeed. To iterate a setup step, do:

```
cd integration-tests/bridges/environments/polkadot-kusama
export FRAMEWORK_PATH=~/local_bridge_testing/downloads/polkadot-sdk/bridges/testing/framework/
source "$FRAMEWORK_PATH/utils/bridges.sh"
source "$FRAMEWORK_PATH/utils/zombienet.sh"

# re-run init scripts
./helper.sh init-asset-hub-polkadot-local

# manually run zndsl (adjust test DIR as per zombienet output:
export ZOMBIENET_BINARY=~/local_bridge_testing/bin/zombienet
export TEST_DIR=/tmp/bridges-tests-run-vEMZt
export ENV_PATH=~/integritee/parachain/integration-tests/bridges/environments/polkadot-kusama
polkadot_dir=`cat $TEST_DIR/polkadot.env`
kusama_dir=`cat $TEST_DIR/kusama.env`
run_zndsl ../../tests/0001-polkadot-kusama-asset-transfer/ksm-reaches-polkadot.zndsl $polkadot_dir
```
