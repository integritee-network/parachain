#!/bin/bash
# Remark: wasm and state will be identical for different relay-chains

CHAIN_SPEC=$1
PARA_ID=${2:-2015}
COLLATOR=${3:-./target/release/integritee-collator}
DUMP_DIR=./chain_dumps

echo "dumping spec for: $CHAIN_SPEC"
echo "para_id:          ${PARA_ID}"
echo "collator:         ${COLLATOR}"
echo ""

$COLLATOR build-spec --chain ${CHAIN_SPEC} >$DUMP_DIR/${CHAIN_SPEC}.json
$COLLATOR build-spec --chain ${CHAIN_SPEC} --raw >$DUMP_DIR/${CHAIN_SPEC}-raw.json
sed -i "/\"para_id\": 2015/s/2015/${PARA_ID}/" $DUMP_DIR/${CHAIN_SPEC}.json
sed -i "/\"para_id\": 2015/s/2015/${PARA_ID}/" $DUMP_DIR/${CHAIN_SPEC}-raw.json

$COLLATOR export-genesis-state --chain $DUMP_DIR/${CHAIN_SPEC}-raw.json --parachain-id ${PARA_ID} >$DUMP_DIR/${CHAIN_SPEC}.state
$COLLATOR export-genesis-wasm --chain $DUMP_DIR/${CHAIN_SPEC}-raw.json >$DUMP_DIR/${CHAIN_SPEC}.wasm
