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
  --output=./$INTEGRITEE_RUNTIME_WEIGHT_DIR/frame_system.rs