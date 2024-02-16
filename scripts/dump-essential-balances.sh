#!/bin/bash
# make sure polkadot-js-api is in the path
# run zombienet locally
#
# example
# ---------------
# ./dump-essential-balances.sh > fee1.txt
# DO YOUR THING
# ./dump-essential-balances.sh > fee2.txt
#
# compare balances pre/post YOUR THING
# diff -y -W 80 fee1.txt fee2.txt

ASSET_HUB="ws://127.0.0.1:9954"
INTEGRITEE="ws://127.0.0.1:9944"
ROCOCO="ws://127.0.0.1:9999"

# subalfred key --type pallet --show-prefix 'py/trsry'
TREASURY=5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z

function print_balances() {
    echo "*** print balances for $1"
    echo "Asset Hub ROC"
    polkadot-js-api --ws $ASSET_HUB query.system.account $1 | jq .account.data.free
    echo "Asset Hub TEER"
    polkadot-js-api --ws $ASSET_HUB query.foreignAssets.account '{ "parents": "1", "interior": { "X1": { "Parachain": "2015" }}}' $1 | jq .account.balance
    #integritee-cli -p $ASSET_HUB balance $1
    echo "Rococo ROC"
    polkadot-js-api --ws $ROCOCO query.system.account $1 | jq .account.data.free
    #integritee-cli -p $ROCOCO balance $1
    echo "Integritee TEER"
    polkadot-js-api --ws $INTEGRITEE query.system.account $1 | jq .account.data.free
    echo "Integritee ROC"
    polkadot-js-api --ws $INTEGRITEE query.assets.account 0 $1 | jq .account.balance
    #integritee-cli -p $INTEGRITEE balance $1
}

echo "*** total supply of TEER@Integritee"
polkadot-js-api --ws $INTEGRITEE query.balances.totalIssuance | jq .totalIssuance
echo "*** ROC derivative total issuance on Integritee"
polkadot-js-api --ws $INTEGRITEE query.assets.asset 0 | jq .asset.supply
echo "*** TEER Treasury@Integritee balance"
polkadot-js-api --ws $INTEGRITEE query.system.account $TREASURY | jq .account.data.free
echo "*** ROC Treasury@Integritee balance"
polkadot-js-api --ws $INTEGRITEE query.assets.account 0 $TREASURY | jq .account.balance

echo "*** Alice"
print_balances 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
echo "*** Parachain 2015"
print_balances 5Ec4AhPXwiUhF3u21i56Uxc5XFUNEkKAq79kutWUAx1jyz47
echo "*** Parachain 1000"
print_balances 5Ec4AhPZk8STuex8Wsi9TwDtJQxKqzPJRCH7348Xtcs9vZLJ
echo "*** Sibling 2015"
print_balances 5Eg2fntM2SfbNedAZi2Bbbwtmj9fxANGtkoCAr2y3g3JqoH4
echo "*** Sibling 1000"
print_balances 5Eg2fntNprdN3FgH4sfEaaZhYtddZQSQUqvYJ1f2mLtinVhV





