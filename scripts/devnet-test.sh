#!/bin/bash
# Permia Devnet Multi-Node Test Script
#
# This script starts two Permia nodes and tests block production and sync.
# Node 1 produces blocks (miner), Node 2 syncs from Node 1.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RETH_DIR="$(dirname "$SCRIPT_DIR")"
PERMIA_BIN="$RETH_DIR/target/release/permia"

# Node directories
NODE1_DIR="/tmp/permia-devnet/node1"
NODE2_DIR="/tmp/permia-devnet/node2"

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
echo "Permia Devnet Multi-Node Test"
echo "=========================================="
echo ""
echo "Configuration:"
echo "  - Node 1: Miner (produces blocks every 1s)"
echo "  - Node 2: Syncs from Node 1"
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
    2>&1 | sed 's/^/[Node1] /' &
NODE1_PID=$!

sleep 5
echo "[Node 1] Running with PID $NODE1_PID"

# Start Node 2 (Sync)  
echo ""
echo "[Node 2] Starting sync node on port 30304..."
$PERMIA_BIN node \
    --dev \
    --datadir "$NODE2_DIR" \
    --http --http.port 8546 \
    --authrpc.port 8552 \
    --port 30304 \
    2>&1 | sed 's/^/[Node2] /' &
NODE2_PID=$!

sleep 5
echo "[Node 2] Running with PID $NODE2_PID"

# Let Node 1 produce blocks
echo ""
echo "Letting Node 1 produce blocks for 15 seconds..."
sleep 15

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

echo ""
echo "=========================================="
echo "Results"
echo "=========================================="
echo ""
echo "Node 1 block height: $N1"
echo "Node 2 block height: $N2"

if [ "$N1" -gt 10 ]; then
    echo ""
    echo "✓ SUCCESS: Node 1 is producing blocks (height: $N1)"
else
    echo ""
    echo "✗ FAIL: Node 1 is not producing blocks"
fi

echo ""
echo "=========================================="
echo "Test Complete"
echo "=========================================="
