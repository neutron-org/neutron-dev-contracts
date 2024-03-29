# before execution: create baryon_testwallet in keyring-backend test in NEUTRON_DIR/data/baryon-1 directory
# test in pion-1 testnet
CONTRACT=./artifacts/client_updater.wasm
CHAINID=pion-1
KEYS_HOME=~/.baryon-1
NEUTROND_BIN=neutrond
NODE=https://rpc-palvus.pion-1.ntrn.tech:443
TEST_WALLET=baryon_testwallet
TEST_ADDR=$(${NEUTROND_BIN} keys show ${TEST_WALLET} --keyring-backend test -a --home ${KEYS_HOME})
GAS_PRICES=0.0025untrn

echo "Store contract"
RES=$(${NEUTROND_BIN} tx wasm store ${CONTRACT} \
    --from ${TEST_ADDR} \
    --gas 50000000 \
    --chain-id ${CHAINID} \
    --broadcast-mode=block \
    --gas-prices ${GAS_PRICES}  -y \
    --output json \
    --keyring-backend test \
    --home ${KEYS_HOME} \
    --node ${NODE})
CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[1].value')
echo $RES
echo $CONTRACT_CODE_ID

INIT_MSG="{}"

echo "Instantiate"
RES=$(${NEUTROND_BIN} tx wasm instantiate $CONTRACT_CODE_ID \
    "$INIT_MSG" \
    --from ${TEST_ADDR} \
    --admin ${TEST_ADDR}  -y \
    --chain-id ${CHAINID} \
    --output json \
    --broadcast-mode=block \
    --label "init" \
    --keyring-backend test \
    --gas-prices ${GAS_PRICES} \
    --home ${KEYS_HOME} \
    --node ${NODE})
echo $RES
CONTRACT_ADDRESS=$(echo $RES | jq -r '.logs[0].events[0].attributes[0].value')
echo $CONTRACT_ADDRESS

echo "Client update"
RES=$(${NEUTROND_BIN} tx wasm execute $CONTRACT_ADDRESS \
    '{"submit_client_update_proposal":{"title": "update pion-1 client 07-tendermint-4 to new one", "description": "update client", "subject_client_id": "07-tendermint-4", "substitute_client_id": "07-tendermint-9"}}' \
    --amount "500untrn" \
    --from ${TEST_ADDR}  -y \
    --chain-id ${CHAINID} \
    --output json \
    --broadcast-mode=block \
    --gas-prices ${GAS_PRICES} \
    --gas 1000000 \
    --keyring-backend test \
    --home ${KEYS_HOME} \
    --node ${NODE})
echo $RES | jq

echo "Client update 2"
RES=$(${NEUTROND_BIN} tx wasm execute $CONTRACT_ADDRESS \
    '{"submit_client_update_proposal":{"title": "update pion-1 client 07-tendermint-5 to new one", "description": "update client 2", "subject_client_id": "07-tendermint-5", "substitute_client_id": "07-tendermint-10"}}' \
    --amount "500untrn" \
    --from ${TEST_ADDR}  -y \
    --chain-id ${CHAINID} \
    --output json \
    --broadcast-mode=block \
    --gas-prices ${GAS_PRICES} \
    --gas 1000000 \
    --keyring-backend test \
    --home ${KEYS_HOME} \
    --node ${NODE})
echo $RES | jq
