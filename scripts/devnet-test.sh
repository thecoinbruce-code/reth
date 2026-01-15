#!/bin/bash
# Permia Devnet Multi-Node Test Script
#
# This script starts two Permia nodes and tests block production and P2P sync.
# Node 1 produces blocks (miner), Node 2 syncs from Node 1 via P2P.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RETH_DIR="$(dirname "$SCRIPT_DIR")"
PERMIA_BIN="$RETH_DIR/target/release/permia"

# Node directories
NODE1_DIR="/tmp/permia-devnet/node1"
NODE2_DIR="/tmp/permia-devnet/node2"
NODE1_LOG="/tmp/permia-devnet/node1.log"

# Cleanup
cleanup() {
    echo ""
    echo "Cleaning up..."
    pkill -f "permia node" 2>/dev/null || true
    sleep 1
}

trap cleanup EXIT

# Create directories
rm -rf /tmp/permia-devnet
mkdir -p "$NODE1_DIR" "$NODE2_DIR"

echo "=========================================="
echo "Permia Devnet Multi-Node Test (P2P Sync)"
echo "=========================================="
echo ""
echo "Configuration:"
echo "  - Node 1: Miner (produces blocks every 1s)"
echo "  - Node 2: Syncs from Node 1 via P2P"
echo "  - Chain: Permia Devnet (ID: 42071)"

# Start Node 1 (Miner)
echo ""
echo "[Node 1] Starting miner on port 30303..."
$PERMIA_BIN node \
    --dev \
    --dev.block-time 1s \
    --datadir "$NODE1_DIR" \
    --http --http.port 8545 \
    --authrpc.port 8551 \
    --port 30303 \
    2>&1 | tee "$NODE1_LOG" | sed 's/^/[Node1] /' &
NODE1_PID=$!

# Wait for Node 1 to start and extract enode
echo "[Node 1] Waiting for startup..."
sleep 5

# Extract enode from log
NODE1_ENODE=$(grep -o 'enode://[^[:space:]]*' "$NODE1_LOG" | head -1 || echo "")
if [ -z "$NODE1_ENODE" ]; then
    echo "[Node 1] Warning: Could not extract enode, using node_info RPC..."
    NODE1_ENODE=$(curl -s -X POST http://127.0.0.1:8545 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"admin_nodeInfo","params":[],"id":1}' | jq -r '.result.enode' 2>/dev/null || echo "")
fi

echo "[Node 1] Running with PID $NODE1_PID"
if [ -n "$NODE1_ENODE" ]; then
    echo "[Node 1] Enode: ${NODE1_ENODE:0:80}..."
fi

# Start Node 2 (Sync) - NOT in dev mode, syncs from Node 1 via P2P
# Node 2 uses same chain spec but doesn't produce blocks locally
echo ""
echo "[Node 2] Starting sync node on port 30304 (non-mining)..."
if [ -n "$NODE1_ENODE" ]; then
    $PERMIA_BIN node \
        --chain dev \
        --datadir "$NODE2_DIR" \
        --http --http.port 8546 \
        --authrpc.port 8552 \
        --port 30304 \
        --trusted-peers "$NODE1_ENODE" \
        --full \
        2>&1 | sed 's/^/[Node2] /' &
else
    # Fallback without trusted peers
    $PERMIA_BIN node \
        --chain dev \
        --datadir "$NODE2_DIR" \
        --http --http.port 8546 \
        --authrpc.port 8552 \
        --port 30304 \
        --full \
        2>&1 | sed 's/^/[Node2] /' &
fi
NODE2_PID=$!

sleep 5
echo "[Node 2] Running with PID $NODE2_PID"

# Let Node 1 produce blocks and Node 2 sync
echo ""
echo "Letting nodes run for 20 seconds (production + sync)..."
sleep 20

# Check status via RPC
echo ""
echo "=========================================="
echo "Node Status"
echo "=========================================="

echo ""
echo "[Node 1] Block number:"
NODE1_BLOCK=$(curl -s -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}')
echo "$NODE1_BLOCK" | jq .

echo ""
echo "[Node 2] Block number:"
NODE2_BLOCK=$(curl -s -X POST http://127.0.0.1:8546 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}')
echo "$NODE2_BLOCK" | jq .

echo ""
echo "[Node 1] Chain ID:"
curl -s -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' | jq .

# Parse block numbers
N1=$(echo "$NODE1_BLOCK" | jq -r '.result' | xargs printf "%d" 2>/dev/null || echo "0")
N2=$(echo "$NODE2_BLOCK" | jq -r '.result' | xargs printf "%d" 2>/dev/null || echo "0")

# Check peer count
echo ""
echo "[Node 1] Peer count:"
curl -s -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' | jq .

echo ""
echo "[Node 2] Peer count:"
curl -s -X POST http://127.0.0.1:8546 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' | jq .

echo ""
echo "=========================================="
echo "Results"
echo "=========================================="
echo ""
echo "Node 1 block height: $N1"
echo "Node 2 block height: $N2"

# Check block production
if [ "$N1" -gt 10 ]; then
    echo "✓ Node 1 is producing blocks (height: $N1)"
else
    echo "✗ Node 1 is NOT producing blocks"
fi

# Check P2P sync
if [ "$N2" -gt 0 ]; then
    echo "✓ Node 2 is syncing via P2P (height: $N2)"
    if [ "$N2" -ge "$((N1 - 5))" ]; then
        echo "✓ Nodes are in sync (within 5 blocks)"
    else
        echo "⚠ Node 2 is behind by $((N1 - N2)) blocks"
    fi
else
    echo "✗ Node 2 is NOT syncing (height: 0)"
fi

echo ""
echo "=========================================="
echo "Test Complete"
echo "=========================================="
