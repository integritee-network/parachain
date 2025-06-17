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
bun papi add pah -n polkadot_asset_hub
bun papi add ppeople -n polkadot_people
# bun papi add ik -w wss://kusama.api.integritee.network -c ../../polkadot-parachains/chain-specs/integritee-kusama.json
bun papi add ik -w ws://localhost:9144 -c ../../polkadot-parachains/chain-specs/integritee-kusama.json
# bun papi add ip -w wss://polkadot.api.integritee.network -c ../../polkadot-parachains/chain-specs/integritee-polkadot.json 
bun papi add ip -w ws://localhost:9244 -c ../../polkadot-parachains/chain-specs/integritee-polkadot.json
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

