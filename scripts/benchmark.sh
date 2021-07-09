#!/bin/bash

INTEGRITEE_RUNTIME_WEIGHT_DIR=polkadot-parachains/integritee-runtime/src/weights
COLLATOR=./target/release/integritee-collator

mkdir -p $INTEGRITEE_RUNTIME_WEIGHT_DIR

pallets=(
  "frame_system" \
#  "pallet_assets" \ Fixme: pallet assets throws an error while benchmarking, but do we even need that pallet??
  "pallet_balances" \
  "pallet_timestamp" \
  "pallet_vesting" \
  "pallet_teerex" \
)

for pallet in ${pallets[*]}; do
  echo benchmarking "$pallet"...

  $COLLATOR \
  benchmark \
  --chain=integritee-rococo-local-dev \
  --steps=50 \
  --repeat=20 \
  --pallet="$pallet" \
  --extrinsic="*" \
  --execution=wasm \
  --wasm-execution=compiled \
  --heap-pages=4096 \
  --output=./$INTEGRITEE_RUNTIME_WEIGHT_DIR/"$pallet".rs \

done


# use the command below with the custom template if you also want to
# create a `WeightInfo` trait definition and test implementation to copy over
# to pallet_teerex's weight.rs.

#$COLLATOR \
#  benchmark \
#  --chain=integritee-rococo-local-dev \
#  --steps=50 \
#  --repeat=20 \
#  --pallet=pallet_teerex \
#  --extrinsic="*" \
#  --execution=wasm \
#  --wasm-execution=compiled \
#  --heap-pages=4096 \
#  --output=./$INTEGRITEE_RUNTIME_WEIGHT_DIR/pallet_teerex.rs
#  --template=./scripts/frame-weight-template-complete.hbs

