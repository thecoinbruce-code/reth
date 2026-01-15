# Permia Mining Integration

## Current State

The Permia node has working CPU mining that produces valid PermiaHash PoW blocks,
but these blocks are not yet submitted to the chain database.

## Architecture

### Reth's Block Submission Flow (PoS)

```
Consensus Layer (Beacon Chain)
        │
        ▼
   Engine API (RPC)
        │
        ├── engine_newPayloadV4(payload)  → Validates & inserts block
        │
        └── engine_forkchoiceUpdated()    → Makes block canonical
        │
        ▼
   EngineTree (crates/engine/tree)
        │
        ▼
   Database (persists block)
```

### Permia's PoW Mining Flow (Target)

```
PermiaHash Miner (permia-miner)
        │
        ▼
   MinedBlock { hash, nonce, mix_hash }
        │
        ▼
   Block Builder (construct full block)
        │
        ▼
   ConsensusEngineHandle
        │
        ├── new_payload()      → Submit block
        │
        └── fork_choice_updated()  → Make canonical
        │
        ▼
   Chain Database
```

## Key Components

### 1. ConsensusEngineHandle
Location: `crates/engine/primitives/src/message.rs`

```rust
impl<Payload> ConsensusEngineHandle<Payload> {
    pub async fn new_payload(&self, payload: ExecutionData) -> Result<PayloadStatus, Error>;
    pub async fn fork_choice_updated(&self, state: ForkchoiceState, ...) -> Result<ForkchoiceUpdated, Error>;
}
```

### 2. BeaconEngineMessage
Location: `crates/engine/primitives/src/message.rs`

```rust
pub enum BeaconEngineMessage<Payload> {
    NewPayload { payload, tx },
    ForkchoiceUpdated { state, payload_attrs, version, tx },
}
```

### 3. ExecutionPayload Construction
For PoW, we need to construct an `ExecutionPayloadV3` from our mined block:

```rust
let payload = ExecutionPayloadV3 {
    parent_hash: mined_block.parent_hash,
    fee_recipient: miner_address,
    state_root: computed_state_root,
    receipts_root: computed_receipts_root,
    logs_bloom: computed_logs_bloom,
    prev_randao: mined_block.mix_hash,  // PoW mix_hash
    block_number: mined_block.number,
    gas_limit: block_gas_limit,
    gas_used: 0,  // Empty block initially
    timestamp: current_timestamp,
    extra_data: extra_data.into(),
    base_fee_per_gas: calculated_base_fee,
    block_hash: mined_block.hash,
    transactions: vec![],  // Empty initially
    withdrawals: vec![],
    blob_gas_used: 0,
    excess_blob_gas: 0,
};
```

## Implementation Steps

### Phase 1: Block Construction (TODO)
1. After mining nonce, construct full block with state root
2. This requires access to the state provider from the node

### Phase 2: Engine API Integration (TODO)
1. Get `ConsensusEngineHandle` from node components
2. Call `new_payload()` with constructed payload
3. Call `fork_choice_updated()` to make canonical

### Phase 3: Transaction Inclusion (TODO)
1. Pull transactions from mempool
2. Execute transactions and compute state roots
3. Include in mined blocks

## Current Workaround

For devnet testing, mining is demonstrated but blocks show as `0x0` in RPC
because they're not submitted to the chain. The mining hashrate (~35K H/s)
and block discovery is working correctly.

## P2P Sync Considerations

Reth is designed for Proof-of-Stake where blocks come from the consensus layer
via Engine API, not from P2P gossip. For Permia's PoW model:

### Current Limitation
- Nodes connect via P2P (peer count shows connection)
- But blocks are NOT propagated via P2P gossip
- Each node in dev mode produces blocks locally via LocalMiner

### Solution Path
To enable true P2P block sync for PermiaHash PoW:

1. **Custom Block Propagation**: Implement block announcement/request handlers
   that validate PermiaHash PoW before accepting blocks from peers.

2. **Consensus Integration**: Replace Reth's beacon consensus with PermiaHash
   validation in the block import pipeline.

3. **Block Gossip**: Enable traditional PoW-style block gossip where miners
   broadcast new blocks and peers validate/import them.

### Files to Modify for P2P PoW
- `crates/net/eth-wire/` - Block announcement messages
- `crates/net/network/` - Block propagation handlers
- `crates/consensus/consensus/` - Block validation interface

## Related Files

- `crates/permia/miner/src/node_miner.rs` - Mining task
- `crates/permia/miner/src/worker.rs` - PoW nonce search
- `crates/engine/primitives/src/message.rs` - Engine messages
- `crates/engine/tree/src/tree/mod.rs` - Block insertion logic
- `bin/permia/src/main.rs` - Node entry point
