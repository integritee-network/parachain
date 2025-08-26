#!/bin/bash

# Create `WeightInfo` implementations for all the pallets and store it in the weight module of the `integritee-kusama`.

INTEGRITEE_RUNTIME_WEIGHT_DIR=polkadot-parachains/integritee-kusama/src/weights
COLLATOR=./target/release/integritee-collator

mkdir -p $INTEGRITEE_RUNTIME_WEIGHT_DIR

$COLLATOR benchmark pallet \
    --chain integritee-rococo-local-dev \
    --list |\
  tail -n+2 |\
  cut -d',' -f1 |\
  uniq > "integritee_runtime_pallets"

# For each pallet found in the previous command, run benches on each function
while read -r line; do
  pallet="$(echo "$line" | cut -d' ' -f1)";
  echo benchmarking "$pallet"...

  $COLLATOR \
  benchmark pallet \
  --chain=integritee-rococo-local-dev \
  --steps=50 \
  --repeat=20 \
  --pallet="$pallet" \
  --extrinsic="*" \
  --heap-pages=4096 \
  --output=./$INTEGRITEE_RUNTIME_WEIGHT_DIR/"$pallet".rs
done < "integritee_runtime_pallets"
rm "integritee_runtime_pallets"

# Todo: This is a hack now, see https://github.com/integritee-network/parachain/issues/343
mv $INTEGRITEE_RUNTIME_WEIGHT_DIR/pallet_xcm_benchmarks::fungible.rs $INTEGRITEE_RUNTIME_WEIGHT_DIR/xcm/pallet_xcm_benchmarks_fungible.rs
mv $INTEGRITEE_RUNTIME_WEIGHT_DIR/pallet_xcm_benchmarks::generic.rs $INTEGRITEE_RUNTIME_WEIGHT_DIR/xcm/pallet_xcm_benchmarks_generic.rs
