#!/bin/bash
# Remark: wasm and state will be identical for different relay-chains

spec=$1

COLLATOR=${2:-./target/release/integritee-collator}
DUMP_DIR=./chain_dumps

echo "dumping spec for $spec"
$COLLATOR build-spec --chain $spec >  $DUMP_DIR/${spec}.json
$COLLATOR build-spec --chain $spec --raw > $DUMP_DIR/${spec}-raw.json
$COLLATOR export-genesis-state --chain $DUMP_DIR/${spec}-raw.json --parachain-id 2015 > $DUMP_DIR/${spec}.state
$COLLATOR export-genesis-wasm --chain $DUMP_DIR/${spec}-raw.json >  $DUMP_DIR/${spec}.wasm
