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

## debug

bun won't tell you about type errors. Use this to debug your code:

```
tsc --noEmit xcm-v5-remark_on_kah_as_itk.ts
# transpile entire directory:
tsc --noEmit -p ./tsconfig.json 
```