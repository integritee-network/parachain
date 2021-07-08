#!/bin/bash

INTEGRITEE_RUNTIME_WEIGHT_DIR=polkadot-parachains/integritee-runtime/src/weights
COLLATOR=./target/release/integritee-collator

mkdir -p $INTEGRITEE_RUNTIME_WEIGHT_DIR

echo benchmarking frame_system...

$COLLATOR \
  benchmark \
  --chain=integritee-rococo-local-dev \
  --steps=50 \
  --repeat=20 \
  --pallet=frame_system \
  --extrinsic="*" \
  --execution=wasm \
  --wasm-execution=compiled \
  --heap-pages=4096 \
  --output=./$INTEGRITEE_RUNTIME_WEIGHT_DIR/frame_system.rs \

echo benchmarking pallet_teerex...

$COLLATOR \
  benchmark \
  --chain=integritee-rococo-local-dev \
  --steps=50 \
  --repeat=20 \
  --pallet=pallet_teerex \
  --extrinsic="*" \
  --execution=wasm \
  --wasm-execution=compiled \
  --heap-pages=4096 \
  --output=./$INTEGRITEE_RUNTIME_WEIGHT_DIR/pallet_teerex.rs \
  --template=./scripts/frame-weight-template.hbs
# use the above template if you also want to also create a type definition to copy over to pallet_teerex's `WeightInfo`
# definition and to create an implementation for `()` for tests

