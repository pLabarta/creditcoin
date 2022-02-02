#!/bin/bash

if [ "$MODE" = "rpc" ]; then
    ws='--ws-external'
    rpc='--rpc-external'
    cors="--rpc-cors ${CORS:-all}"
    pruning="--pruning ${PRUNING:-archive}"
else
    mining_key="--mining-key $(/bin/creditcoin-node generate-mining-key --chain /chainspec/testnetSpec.json --quiet)"
    validator='--validator'
fi
if [ "$RESYNC" ]; then
    /bin/creditcoin-node purge-chain --chain /chainspec/testnetSpec.json
fi
if [ "$PROMETHEUS" = "1" ]; then
    prometheus='--prometheus-external'
fi
if [ -n "$BOOTNODE_IP" ] || [ -n "$BOOTNODE_FQDN" ]; then
    boot_id="${BOOTNODE_PEER_ID:-12D3KooWSnPk7hN9epDUonVuNVgTvxbmnuQbGP8ghMvuoH9GAsz5}"
    if [ "$NODE_NAME" ]; then
        name="--name $NODE_NAME"
    fi
    if [ "$BOOTNODE_FQDN" ]; then
        bootnode="--bootnodes /dns4/$BOOTNODE_FQDN/tcp/30333/p2p/$boot_id"
    else
        bootnode="--bootnodes /ip4/$BOOTNODE_IP/tcp/30333/p2p/$boot_id"
    fi
    /bin/creditcoin-node --chain /chainspec/testnetSpec.json $name $bootnode \
     --port 30333 --ws-port 9944 --rpc-port 9933 \
     --public-addr "/dns4/$FQDN/tcp/30333" \
     --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
     --base-path "${DATA:-/data}" $validator $mining_key $ws $rpc $cors $pruning $prometheus $EXTRA_ARGS
else
    key="${NODE_KEY:-c5eb4a9ada5c9dd76378d000f046e8cde064d68e96a1df569190eee70afba8e7}"
    node_name="${NODE_NAME:-bootnode}"
    /bin/creditcoin-node --chain /chainspec/testnetSpec.json --node-key "$key" --name "$node_name" \
     --port 30333 --ws-port 9944 --rpc-port 9933 \
     --public-addr "/dns4/$FQDN/tcp/30333" \
     --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
     --base-path "${DATA:-/data}" $validator $mining_key $ws $rpc $cors $pruning $prometheus $EXTRA_ARGS
fi
