# manual testing script for the bridges
# this is nothing but an endless loop after spawning the zombienet to allow manual testing on the zombienet for as long as it takes


set -e

source "$FRAMEWORK_PATH/utils/common.sh"
source "$FRAMEWORK_PATH/utils/zombienet.sh"

export ENV_PATH=`realpath ${BASH_SOURCE%/*}/../../environments/polkadot-kusama`

$ENV_PATH/spawn.sh --init --start-relayer &
env_pid=$!

ensure_process_file $env_pid $TEST_DIR/polkadot.env 600
polkadot_dir=`cat $TEST_DIR/polkadot.env`
echo

ensure_process_file $env_pid $TEST_DIR/kusama.env 300
kusama_dir=`cat $TEST_DIR/kusama.env`
echo

echo "Zombienet is ready for manual testing."

while true; do
  sleep 1
done