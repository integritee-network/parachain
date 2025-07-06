The goal of this directory is to provide templates for frontend-side XCM testing and dry-running calls.
Didn't work nicely with zombienet so far, but works with chopsticks against live chains

# dev setup

Install PAPI and bun

```
npm i -g polkadot-api
npm install -g bun
```

# run simple XCM tests

In another terminal, run chopsticks

```
nvm use 22
npx @acala-network/chopsticks@latest xcm --p=polkadot-asset-hub --p=polkadot-people
```

then run e.g.

```
bun xcm-v4-example-teleport-dot-from-pah-to-people.ts 
```

# run TEER bridging tests

## with chopsticks

As long as https://github.com/polkadot-fellows/runtimes/pull/794 isn't merged, you need to build a patched runtime
build patched asset hub runtimes. You may want to do this anyway to get debug logs. Live chain `production` build is
silent - `release` shows logs.

```
git clone https://github.com/encointer/runtimes.git
git checkout ab/trusted-aliasers-for-bridging
cargo build --release -p asset-hub-polkadot-runtime -p asset-hub-kusama-runtime
```

build the integritee runtimes with sudo enabled

```
git apply ./integration-tests/bridges/sudo-integritee.patch
cargo build --release -p integritee-kusama-runtime -p integritee-polkadot-runtime
```

fork IK, KAH, PAH and IP live chains

```
git clone https://github.com/brenzi/chopsticks.git
git checkout ab/xcm-v5-debug

# copy wasm files from above builds into repo root folder and adjust file names in ymls

npx @acala-network/chopsticks@latest xcm --p=./configs/kusama-asset-hub.yml --p=./configs/integritee-kusama.yml
// should be ports 8000 and 8001 respectively.
npx @acala-network/chopsticks@latest xcm --p=./configs/polkadot-asset-hub.yml --p=./configs/integritee-polkadot.yml
// should be ports 8002 and 8003 respectively.
```

run TEER bridging test, first as dry-run, then on chopsticks:

You may need to edit the script and set `CHOPSTICKS = true` to enable the correct websocket setup to use chopsticks
ports

```
bun xcm-v5-bridge-remark_on_itp_as_itk.ts
```

## with local zombienet

follow the zombienet instructions [here](../bridges/README.md) to setup your environment

then, you can run the tests against zombienet

```
cd ../bridges
./run-test.sh 0000-manual
```

once you see `Zombienet is ready for manual testing.` you should run sanity checks and fix issues if any:

```
bun xcm-sanity-checks.ts
```

run TEER bridging test, first as dry-run, then on chopsticks:

You may need to edit the script and set `CHOPSTICKS = false` to enable the correct websocket setup to use chopsticks
ports

```
bun xcm-v5-bridge-remark_on_itp_as_itk.ts
```

## developer hints

bun won't tell you about type errors. Use this to debug your code:

```

tsc --noEmit xcm-v5-remark_on_kah_as_itk.ts

# transpile entire directory:

tsc --noEmit -p ./tsconfig.json

```

### setup of this dir

If you want to start from scratch

```
bun init -y
bun i polkadot-api @polkadot-labs/hdkd  @polkadot-labs/hdkd-helpers
# zombienet uses more recent runtimes, so get descriptors from there
bun papi add dot -w ws://localhost:9942
bun papi add ksm -w ws://localhost:9945
bun papi add kah -w ws://localhost:9010
bun papi add kbh -w ws://localhost:8945
bun papi add pah -w ws://localhost:9910
bun papi add pbh -w ws://localhost:8943
# bun papi add ppeople -n polkadot_people
bun papi add itk --wasm ../../target/release/wbuild/integritee-kusama-runtime/integritee_kusama_runtime.wasm 
# or: bun papi add itk -w ws://localhost:9144
bun papi add itp -w ws://localhost:9244
```

and in the end, do not forget to generate types and properly install deps

```
# update all metadata and descriptors
bun papi
# run this to update deps from .papi
bun install
```
