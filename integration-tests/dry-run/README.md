The goal of this directory is to provide templates for frontend-side XCM testing and dry-running calls.
Didn't work nicely with zombienet so far, but works with chopsticks against live chains

# dev setup

Install PAPI and bun

```
npm i -g polkadot-api
npm install -g bun
```

# setup of this dir

If you want to start from scratch

```
bun init -y
bun add @polkadot-labs/hdkd  @polkadot-labs/hdkd-helpers
bun papi add dot -n polkadot
bun papi add ksm -n ksmcc3
bun papi add kah -n ksmcc3_asset_hub
bun papi add kbh -n ksmcc3_bridge_hub
bun papi add pah -n polkadot_asset_hub
bun papi add pbh -n polkadot_bridge_hub
bun papi add ppeople -n polkadot_people
# bun papi add ik -w wss://kusama.api.integritee.network -c ../../polkadot-parachains/chain-specs/integritee-kusama.json
bun papi add ik -w ws://localhost:9144 -c /tmp/bridges-tests-run-alwjO/bridge_hub_kusama_local_network/integritee-kusama-local-dev-2015-plain.json
# bun papi add ip -w wss://polkadot.api.integritee.network -c ../../polkadot-parachains/chain-specs/integritee-polkadot.json 
bun papi add ip -w ws://localhost:9244 -c /tmp/bridges-tests-run-alwjO/bridge_hub_polkadot_local_network/integritee-polkadot-local-dev-2039-plain.json
```

and in the end, do not forget to generate types

```
npx papi
```

# run tests

In another terminal, run chopsticks

```
nvm use 22
npx @acala-network/chopsticks@latest xcm --p=polkadot-asset-hub --p=polkadot-people
```

or run bridging tests with zombienet

```
cd ../bridges
./run-test.sh 0000-manual
```

then run any of the xcm scripts with

```
bun xcm-v4-example-teleport-dot-from-pah-to-people.ts 
```

known to work with `../bridges` zombienet `./run-tests.sh 0000-manual`:

```
bun xcm-v5-example-teleport-dot-from-pah-to-pbh-zombienet.ts
```