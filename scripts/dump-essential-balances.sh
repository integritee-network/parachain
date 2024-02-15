#!/bin/bash
# make sure polkadot-js-api is in the path
#
# example
# ---------------
# ./dump-essential-balances.sh > fee1.txt
# DO YOUR THING
# ./dump-essential-balances.sh > fee2.txt
#
# compare balances pre/post YOUR THING
# diff -y -W 80 fee1.txt fee2.txt

ASSET_HUB=9954
INTEGRITEE=9944
ROCOCO=9999

function print_balances() {
    echo "*** print balances for $1"
    echo "Asset Hub ROC"
    polkadot-js-api --ws ws://127.0.0.1:$ASSET_HUB query.system.account $1 | jq .account.data.free
    echo "Asset Hub TEER"
    polkadot-js-api --ws ws://127.0.0.1:$ASSET_HUB query.foreignAssets.account '{ "parents": "1", "interior": { "X1": { "Parachain": "2015" }}}' $1 | jq .account.balance
    #integritee-cli -p $ASSET_HUB balance $1
    echo "Rococo ROC"
    polkadot-js-api --ws ws://127.0.0.1:$ROCOCO query.system.account $1 | jq .account.data.free
    #integritee-cli -p $ROCOCO balance $1
    echo "Integritee TEER"
    polkadot-js-api --ws ws://127.0.0.1:$INTEGRITEE query.system.account $1 | jq .account.data.free
    echo "Integritee ROC"
    polkadot-js-api --ws ws://127.0.0.1:$INTEGRITEE query.assets.account 0 $1 | jq .account.balance
    #integritee-cli -p $INTEGRITEE balance $1
}

echo "*** total supply of TEER@Integritee"
polkadot-js-api --ws ws://127.0.0.1:9944 query.balances.totalIssuance | jq .totalIssuance
echo "*** TEER Treasury balance"
polkadot-js-api --ws ws://127.0.0.1:$INTEGRITEE query.system.account 5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z | jq .account.data.free
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





