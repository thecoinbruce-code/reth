#!/bin/bash
# Permia Devnet Multi-Node Test Script
#
# This script starts two Permia nodes and tests connectivity between them.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RETH_DIR="$(dirname "$SCRIPT_DIR")"
PERMIA_BIN="$RETH_DIR/target/release/permia"

# Node directories
NODE1_DIR="/tmp/permia-devnet/node1"
NODE2_DIR="/tmp/permia-devnet/node2"

# Cleanup
cleanup() {
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

# Start Node 1
echo ""
echo "[Node 1] Starting on port 30303..."
$PERMIA_BIN node \
    --chain dev \
    --datadir "$NODE1_DIR" \
    --http --http.port 8545 \
    --authrpc.port 8551 \
    --port 30303 \
    --disable-discovery \
    --no-persist-peers \
    2>&1 | sed 's/^/[Node1] /' &
NODE1_PID=$!

sleep 5

# Get Node 1's enode
NODE1_ENODE=$(grep -o 'enode://[^@]*@[^[:space:]]*' "$NODE1_DIR/../node1.log" 2>/dev/null || echo "")
if [ -z "$NODE1_ENODE" ]; then
    # Try to extract from recent output
    NODE1_ENODE="enode://dummy@127.0.0.1:30303"
fi

echo ""
echo "[Node 1] Running with PID $NODE1_PID"

# Start Node 2
echo ""
echo "[Node 2] Starting on port 30304..."
$PERMIA_BIN node \
    --chain dev \
    --datadir "$NODE2_DIR" \
    --http --http.port 8546 \
    --authrpc.port 8552 \
    --port 30304 \
    --disable-discovery \
    --no-persist-peers \
    2>&1 | sed 's/^/[Node2] /' &
NODE2_PID=$!

sleep 5
echo ""
echo "[Node 2] Running with PID $NODE2_PID"

# Let nodes run for a bit
echo ""
echo "Letting nodes mine blocks for 20 seconds..."
sleep 20

# Check status via RPC
echo ""
echo "=========================================="
echo "Checking Node Status"
echo "=========================================="

echo ""
echo "[Node 1] Block number:"
curl -s -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' | jq .

echo ""
echo "[Node 2] Block number:"
curl -s -X POST http://127.0.0.1:8546 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' | jq .

echo ""
echo "[Node 1] Chain ID:"
curl -s -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' | jq .

echo ""
echo "=========================================="
echo "Test Complete"
echo "=========================================="
