# Implementation Plan: Governance & DAO Features

**Date:** 2026-07-28  
**Scope:** Features #237, #236, #224, #242  
**Status:** Planning Phase

---

## Executive Summary

Four interconnected governance and aggregation features will enhance the Stellar Price Oracle's decentralization and robustness:

| # | Feature | Complexity | Dependencies |
|---|---------|-----------|--------------|
| #237 | DAO-governed parameter changes | HIGH | #236, timelock |
| #236 | Multi-signature admin wallet | HIGH | timelock |
| #224 | EMA smoothing on aggregates | LOW | prices module |
| #242 | Time-based scheduling for admin actions | MEDIUM | timelock |

**Implementation order:** #236 → #242 → #224 → #237

---

## Current State Analysis

### Existing Infrastructure

**Timelock (Implemented):**
- `timelock.rs` provides single-admin-only gating
- Operations: `propose_operation()` → wait N ledgers → `execute_operation()`
- 8 operation types already defined: `Upgrade`, `SetAdmin`, `SetMinSources`, `SetMaxHistory`, `SetResolution`, `SetDecimals`, `SetDescription`, `SetTimestampThreshold`
- Uses `DataKey::TlPendingOp(id)` for storage
- Events: `OperationProposedEvent`, `OperationExecutedEvent`, `OperationCancelledEvent`

**Multi-sig (Partially Implemented):**
- `multisig.rs` module exists with governor management
- Functions: `set_governors()`, `propose_ms_operation()`, `approve_ms_operation()`, `execute_ms_operation()`
- Linked-list queue pattern for ordered execution: `MsQueueHead`, `MsQueueTail`, `MsOp(id)`
- Requires N-of-M approvals before timelock timer starts
- **Status:** Module compiled but NOT wired into `lib.rs` (dead_code warning)

**Prices Module:**
- Current aggregation methods: `Median`, `Mean`, `TrimmedMean` (enum in `types.rs`)
- Uses `compute_median()`, `compute_mean()`, `compute_trimmed_mean()` in `storage.rs`
- No EMA support currently

**Admin Module:**
- Single `Admin` address stored in `DataKey::Admin`
- Must call `admin.require_auth()` for protected operations
- Controlled via `set_admin()` / `get_admin_address()`

---

## Feature Breakdown

### #236: Multi-Signature Admin Wallet Integration ⭐ FOUNDATION

**Purpose:** Replace single-admin with N-of-M multi-sig to eliminate centralization risk.

**Design:**

1. **Wire Multisig Module into Contract**
   - Add multisig functions to `lib.rs` `#[contractimpl]`
   - Public endpoints:
     - `set_governors(governors: Vec<Address>, required: u32)` — admin only, sets governor list
     - `get_governors() -> Vec<Address>` — read all governors
     - `get_required_approvals() -> u32` — read approval threshold
     - `propose_ms_operation(proposer, op_type, data)` — any governor can propose
     - `approve_ms_operation(approver, op_id)` — governor votes yes
     - `revoke_ms_approval(revoker, op_id)` — governor retracts vote (before threshold reached)
     - `get_ms_operation_status(op_id) -> (op_type, approvals, approvers, status)`
     - `cancel_ms_operation(op_id)` — admin or proposer can cancel pending ops

2. **Approval Workflow**
   - Governor proposes operation → enters `Proposed` state, `approvals: []`
   - Each governor can approve; added to `approvals: [...]`
   - When `approvals.len() == required_approvals`:
     - Operation moves to `approved` state
     - Timelock clock begins (ledger + `CfgTimelockDuration`)
   - Only after both timelock AND multi-sig approval → admin can execute
   - Revocation possible before threshold (to prevent approval spam)

3. **Storage Keys (Extend Types)**
   ```rust
   MsGovernors                    // Vec<Address>
   MsRequiredApprovals            // u32
   MsOpCount                      // u32 (monotonic ID counter)
   MsOp(u32)                      // MultiSigOperation
   MsQueueHead                    // u32 (linked-list head)
   MsQueueTail                    // u32 (linked-list tail)
   MsOperationApprovals(u32)      // Vec<Address>
   MsApprovalTime(u32)            // u32 (ledger when N-th approval reached)
   MsOperationStatus(u32)         // enum: Proposed | Approved | Executed | Cancelled
   ```

4. **Events**
   - `MsGovernorsUpdatedEvent { governor_count, required_approvals }`
   - `MsOperationProposedEvent { op_id, proposer, op_type, data_hash }`
   - `MsApprovalAddedEvent { op_id, approver, approval_count }`
   - `MsApprovalReachedEvent { op_id, timestamp }` — when threshold crossed
   - `MsApprovalRevokedEvent { op_id, approver }`
   - `MsOperationExecutedEvent { op_id, executor }`
   - `MsOperationCancelledEvent { op_id, cancelled_by }`

5. **Tests**
   - 2-of-3 governance: propose → 1st approval → wait → 2nd approval → execute
   - Revocation: approve, then revoke before threshold → counter resets
   - Ordered queue: propose A, approve A (reaches threshold), propose B, approve B, execute A then B
   - Unauthorized: non-governor cannot propose or approve
   - Edge: threshold = 0 should panic; threshold > governor count should panic

---

### #242: Time-Based Admin Action Scheduling (with Multi-sig Support) ⭐ EXTENSION

**Purpose:** Enable precise governance: "execute at timestamp" or "execute after delay_seconds".

**Design:**

1. **Extend Timelock Operations**
   - Add variants to `OperationType`:
     ```rust
     pub enum OperationType {
         // Existing:
         Upgrade,
         SetAdmin,
         SetMinSources,
         SetMaxHistory,
         SetResolution,
         SetDecimals,
         SetDescription,
         SetTimestampThreshold,
         
         // New:
         ExecuteAtTimestamp,        // op_data: (timestamp: u64, inner_op: Bytes)
         ExecuteAfterDelay,         // op_data: (delay_seconds: u32, inner_op: Bytes)
     }
     ```

2. **Time-Based Execution Logic**
   - `propose_operation_at_timestamp(op_type, data, target_timestamp: u64) -> op_id`
     - Validates `target_timestamp > now() + min_delay` (e.g., min 1 hour)
     - Creates a `PendingOperation` with `execution_timestamp` field
     - Proposal ledger = current ledger; execution ledger = calculate from timestamp
   - `execute_operation_at_timestamp(op_id)`
     - Checks `current_timestamp >= pending_op.execution_timestamp`
     - If ready, executes; if too early, panics `TimelockNotReady`

3. **Integration with Multi-sig**
   - Multi-sig governors can propose time-based operations
   - Same approval workflow: need N approvals, THEN wait for time, THEN execute
   - Two-layer protection: governance approval + time-based execution

4. **Storage Keys**
   ```rust
   TlExecutionTimestamp(u32)      // u64 (Unix timestamp for time-based ops)
   TlExecutionLedger(u32)         // u32 (Fallback ledger-based execution if timestamp unavailable)
   ```

5. **Events**
   - `OperationScheduledAtTimestampEvent { op_id, target_timestamp }`
   - `OperationScheduledAfterDelayEvent { op_id, delay_seconds }`

6. **Tests**
   - Schedule operation at future timestamp; execute before → panic
   - Schedule operation at future timestamp; execute at/after → success
   - Multi-sig + time-based: propose, approve, wait for time, execute
   - Cancel a time-scheduled operation before execution

---

### #224: Implement EMA Smoothing on Aggregates (Orthogonal Feature)

**Purpose:** Optional exponential moving average to reduce short-term volatility and resist manipulation.

**Design:**

1. **EMA Configuration**
   - Per-asset EMA alpha parameter (0–10000 basis points = 0.00% to 100%)
   - Alpha = 100 basis points ≈ 0.01 weight on new price, 0.99 on previous EMA
   - Admin-configurable via `set_asset_ema_alpha(asset, alpha)` (admin only, uses timelock)
   - Default: disabled (alpha = 0)

2. **Computation**
   - When aggregated price computed:
     ```
     if alpha == 0:
         return aggregated_price  // No smoothing
     else:
         previous_ema = read_ema(asset)  // Default to aggregated_price on first call
         new_ema = (aggregated_price * alpha + previous_ema * (10000 - alpha)) / 10000
         write_ema(asset, new_ema)
         return new_ema
     ```
   - EMA stored under `DataKey::AssetEMAValue(asset)`

3. **Storage Keys**
   ```rust
   AssetEMAAlpha(Address)         // u32 (0–10000 basis points)
   AssetEMAValue(Address)         // i128 (current EMA value)
   AssetEMALastUpdate(Address)    // u32 (ledger when last updated)
   ```

4. **Events**
   - `AssetEMASmoothingEnabledEvent { asset, alpha }`
   - `AssetEMAUpdatedEvent { asset, new_ema, timestamp }`

5. **Contract Changes**
   - Modify `trigger_aggregation()` to compute EMA if alpha > 0
   - New endpoint: `get_asset_ema(asset) -> Option<i128>`
   - New admin endpoint: `set_asset_ema_alpha(asset, alpha)` — with timelock

6. **Tests**
   - Alpha = 0 → no smoothing, price unchanged
   - Alpha = 10000 → full replacement (new aggregate replaces EMA every time)
   - Alpha = 5000 → 50-50 mix of new price and old EMA
   - Arithmetic overflow/underflow on large prices
   - Division rounding (prefer truncation toward zero)

---

### #237: DAO-Governed Parameter Change System ⭐ ADVANCED

**Purpose:** Token-holder voting for parameter changes; governance token integration.

**Design:**

1. **Governance Token Contract Interface**
   - Assume external governance token contract (e.g., SYSO on Soroban)
   - Interface: read balance for an address, total supply
   - Token contract address stored in `DataKey::GovernanceTokenContract`
   - Admin sets via `set_governance_token_contract(token_address)` (admin only, uses timelock)

2. **Voting System**
   - Create `governance.rs` module
   - **Proposal Types:**
     - Parameter change (e.g., `SetMinSources(u32)`, `SetDescription(String)`)
     - Upgrade WASM
     - Add/remove oracle source
     - Change EMA alpha
   
   - **Voting Workflow:**
     1. Token holder proposes with snapshot block/ledger
     2. Snapshot captured: governance token balances at proposal ledger
     3. Voting window: N ledgers (configurable, default 7 days ≈ ~604800 ledgers)
     4. Each token holder votes once; voting power = token balance at snapshot
     5. Vote counted as "yes" (1), "no" (0), or "abstain" (tracked separately)
     6. After voting window: calculate yes%, no%, abstain%
     7. Quorum = min_participation% of total supply at snapshot
     8. If yes% > 50% AND quorum met → proposal approved
     9. Approved proposal enters timelock + multi-sig
     10. After timelock + N multi-sig approvals → execute

3. **Storage Keys (New Governance Module)**
   ```rust
   GovernanceTokenContract        // Address of external token contract
   ProposalCount                  // u32 (monotonic counter)
   Proposal(u32)                  // GovernanceProposal struct
   ProposalVotes(u32, Address)    // u8 enum: Yes | No | Abstain
   ProposalVotingPower(u32, Address) // i128 (balance at snapshot)
   VotingParameters {
       quorum_percentage: u32,    // 0–100
       voting_window_ledgers: u32, // Default: 604800 (≈7 days)
   }
   ```

4. **Data Structures**
   ```rust
   pub enum VoteOption {
       Yes,
       No,
       Abstain,
   }
   
   pub struct GovernanceProposal {
       pub id: u32,
       pub proposer: Address,
       pub title: String,
       pub description: String,
       pub proposal_type: ProposalType,
       pub snapshot_ledger: u32,
       pub voting_start_ledger: u32,
       pub voting_end_ledger: u32,
       pub yes_votes: i128,
       pub no_votes: i128,
       pub abstain_votes: i128,
       pub total_supply_at_snapshot: i128,
       pub executed: bool,
   }
   
   pub enum ProposalType {
       ParameterChange(OperationType, Bytes), // Reuse OperationType
       Upgrade(BytesN<32>),
       AddSource(Address, String),
       RemoveSource(Address),
       SetEMAAlpha(Address, u32),
   }
   ```

5. **Public Endpoints**
   - `create_proposal(proposer, title, description, proposal_type) -> proposal_id`
   - `vote(voter, proposal_id, vote_option)` — token holder only
   - `get_proposal(proposal_id) -> GovernanceProposal`
   - `list_proposals(start, end) -> Vec<GovernanceProposal>`
   - `finalize_voting(proposal_id)` — after voting window, calculate result
   - `execute_approved_proposal(proposal_id)` — admin or multi-sig executor
   - `set_voting_parameters(quorum%, voting_window_ledgers)` — admin
   - `set_governance_token_contract(contract_address)` — admin

6. **Verification Workflow**
   - Proposer must own ≥ min_proposal_stake (configurable, default 0.1% of supply)
   - Votes counted from snapshot ledger; cannot vote twice on same proposal
   - Voting power fetched from governance token contract at proposal creation
   - After voting window closes, must call `finalize_voting()` to lock results
   - Finalized approved proposal goes into multi-sig queue
   - Multi-sig governors vote and approve
   - After multi-sig approval + timelock, execute via `execute_approved_proposal()`

7. **Events**
   - `ProposalCreatedEvent { proposal_id, proposer, proposal_type }`
   - `VoteCastEvent { proposal_id, voter, vote_option }`
   - `VotingFinalizedEvent { proposal_id, yes_votes, no_votes, result: approved | rejected }`
   - `ProposalExecutedEvent { proposal_id, executed_by }`

8. **Tests**
   - 51% approval, ≥ quorum → execute
   - 50% approval, ≥ quorum → reject
   - <10% participation → reject (quorum not met)
   - Double-vote protection: voter votes twice → second vote rejected
   - Snapshot stability: proposal snapshot locked at creation; later token transfers don't affect voting power
   - Integration test: create proposal → token holders vote → multi-sig approvers vote → execute parameter change

---

## Implementation Roadmap

### Phase 1: Multi-Sig Foundation (#236)
**Duration:** 3–4 days  
**Effort:** 40% of total work

**Tasks:**
1. Wire `multisig.rs` into `lib.rs` (add `#[contractimpl]` functions)
2. Extend `types.rs` with multi-sig data structures and storage keys
3. Implement governor management endpoints
4. Implement proposal/approval/execution workflow
5. Add event types for multi-sig in `events.rs`
6. Write comprehensive tests (2-of-3, 3-of-5, revocation, ordering)
7. Run full test suite; verify zero clippy warnings

**Verification:**
```bash
cargo build -p price-oracle --target wasm32v1-none --release
cargo test -p price-oracle --lib
cargo clippy -p price-oracle -- -D warnings
```

---

### Phase 2: Time-Based Scheduling (#242)
**Duration:** 2 days  
**Effort:** 20% of total work  
**Depends On:** Phase 1

**Tasks:**
1. Extend `OperationType` enum with `ExecuteAtTimestamp` and `ExecuteAfterDelay`
2. Add `TlExecutionTimestamp` and `TlExecutionLedger` storage keys
3. Modify `propose_operation()` to accept optional timestamp parameter
4. Modify `execute_operation()` to check timestamp eligibility
5. Add public endpoints: `propose_operation_at_timestamp()`, `propose_operation_after_delay()`
6. Integrate with multi-sig: time-based operations pass through multi-sig approval first
7. Add event types for time-based scheduling
8. Write tests: pre-maturity execution failure, post-maturity success, multi-sig + time-based

**Verification:** Same as Phase 1

---

### Phase 3: EMA Smoothing (#224)
**Duration:** 1.5 days  
**Effort:** 15% of total work  
**Depends On:** None (orthogonal)

**Tasks:**
1. Add EMA-related storage keys to `types.rs`: `AssetEMAAlpha`, `AssetEMAValue`, `AssetEMALastUpdate`
2. Add `set_asset_ema_alpha()` admin function (uses timelock)
3. Modify `trigger_aggregation()` in `prices.rs` to compute EMA if alpha > 0
4. Add `get_asset_ema(asset) -> Option<i128>` query endpoint
5. Add EMA event types
6. Write tests: EMA computation correctness, no smoothing (alpha=0), full replacement (alpha=10000), rounding behavior
7. Add arithmetic safety checks for large prices

**Verification:** Same as Phase 1

---

### Phase 4: DAO Governance (#237)
**Duration:** 4–5 days  
**Effort:** 25% of total work  
**Depends On:** Phase 1 + Phase 2

**Tasks:**
1. Create `governance.rs` module with voting infrastructure
2. Add governance token contract interface (read balance, total supply)
3. Implement proposal creation and validation
4. Implement voting and double-vote protection
5. Implement vote finalization and quorum/approval threshold checking
6. Integrate approved proposals into multi-sig queue
7. Add all governance data structures and storage keys
8. Add governance event types
9. Write comprehensive tests:
   - 51% approval, 10% quorum met → execute
   - 49% approval → reject
   - <quorum participation → reject
   - Double-vote protection
   - Snapshot immutability
   - Multi-sig + DAO integration test
10. Benchmark gas costs for large voting populations (1000+ token holders)

**Verification:** Same as Phase 1 + gas benchmarks

---

## Architecture Decisions

### Why This Order?

1. **#236 first:** All other governance features depend on multi-sig as the execution layer. It's the foundation.
2. **#242 second:** Time-based scheduling is a natural extension of timelock, and completes the governance execution layer.
3. **#224 orthogonal:** EMA can be implemented independently. Doing it mid-stream adds variety to the work cadence.
4. **#237 last:** Most complex feature; depends on stable multi-sig + timelock infrastructure to build upon.

### Storage Conventions

- **Governance keys** prefixed with `Ms` (multi-sig) and `Gov` (DAO governance)
- **EMA keys** prefixed with `AssetEMA` for clarity
- **Timelock keys** prefixed with `Tl` (existing convention preserved)
- All new keys added to `DataKey` enum in `types.rs`

### Testing Strategy

- **Unit tests:** Each module tests its own logic (unit)
- **Integration tests:** Cross-module workflows (multi-sig + timelock, multi-sig + DAO, EMA + aggregation)
- **Scenario tests:** Real-world workflows (2-of-3 multi-sig votes to change min_sources, then EMA alpha change)
- **Edge case tests:** Arithmetic limits, zero values, max values, empty collections

### Error Handling

- Reuse existing `ErrorCode` enum; add new variants as needed
- All panics use `panic_with_error!(env, ErrorCode::...)` pattern
- No silent failures; all constraint violations produce explicit errors

### Event Emission

- All state changes emit events for indexer/monitoring
- Events included in per-function documentation
- Event types added to `events.rs` with `#[contracttype]` and `.publish(env)` calls

---

## Risk Mitigation

### Risks

| Risk | Mitigation |
|------|-----------|
| Multi-sig fails to integrate properly | Phase 1 uses existing implementation with incremental wiring; comprehensive tests cover all paths |
| Time-based execution races ledger time | Use millisecond-granularity timestamps; ledger sequence as fallback |
| EMA arithmetic overflows on extreme prices | Test with max i128 ± boundary values; use `checked_mul()` / `saturating_div()` |
| DAO voting centralization (large token holders) | Quadratic voting (optional future enhancement); multi-sig veto gate for emergency actions |
| Gas costs explode with large governance sets | Benchmark with 100, 500, 1000 token holders; optimize vote counting if needed |
| Replay attacks across proposals | Each proposal gets unique ID; vote struct includes proposal_id; voters can only vote once per proposal |

### Testing Checklist

- [ ] All 76 existing tests pass
- [ ] Add 15+ tests for multi-sig (Phase 1)
- [ ] Add 8+ tests for time-based scheduling (Phase 2)
- [ ] Add 6+ tests for EMA (Phase 3)
- [ ] Add 20+ tests for DAO governance (Phase 4)
- [ ] Zero clippy warnings
- [ ] Zero compiler warnings
- [ ] Documented all new public endpoints
- [ ] All events emit correctly (spot-check with events.rs)

---

## Deliverables

### Code Changes

1. **Wiring & Integration**
   - `lib.rs`: Add ~40 new `#[contractimpl]` endpoints
   - `multisig.rs`: Verify and extend as needed
   - `governance.rs`: New module with 200+ lines

2. **Data Structures**
   - `types.rs`: +50 lines (storage keys + structs)
   - `errors.rs`: +5 new error codes

3. **Events**
   - `events.rs`: +20 event types

4. **Modules**
   - `timelock.rs`: +50 lines (time-based execution)
   - `prices.rs`: +30 lines (EMA computation)
   - `admin.rs`: +20 lines (EMA alpha configuration)

5. **Tests**
   - `test.rs`: +500 lines (new test cases)

### Documentation

1. **README.md** - Update with governance system overview
2. **docs/GOVERNANCE.md** - Architecture, voting process, integration guide
3. **docs/MULTISIG.md** - Multi-sig setup, approval workflow, emergency procedures
4. **docs/EMA.md** - EMA parameters, computation, use cases
5. **docs/TIME_SCHEDULING.md** - Timestamp-based execution examples

### Deployment Checklist

- [ ] All tests pass locally
- [ ] Code reviewed and approved
- [ ] Security audit for governance token integration
- [ ] Deployed to testnet; lifecycle test successful
- [ ] Monitoring dashboards updated (events, gas usage)
- [ ] Runbook created for emergency de-governance (if needed)

---

## Success Criteria

1. **All four features deployed and operational on testnet**
2. **All 76 existing tests + 49 new tests pass (100% pass rate)**
3. **Zero clippy warnings; zero compiler warnings**
4. **Multi-sig 2-of-3 governance vote successfully changes parameter**
5. **Time-based operation scheduled and executed at correct ledger/timestamp**
6. **EMA smoothing correctly reduces price volatility without numeric errors**
7. **DAO proposal with 51% token holder approval and quorum met → parameter change executed**
8. **All documentation updated and peer-reviewed**
9. **Gas benchmarks documented for governance operations**
10. **Production deployment approved by security team**

---

## Known Issues & Future Work

### Known Issues (To Track)

- `#[allow(dead_code)]` in `lib.rs` silences unused warnings for reputation, correlation, etc. This PR will populate multi-sig endpoints; similar wiring needed for other modules in future PRs.
- EMA rounding: using truncation-toward-zero; monitor for cumulative drift over 100+ aggregations.

### Future Enhancements

- **Quadratic voting** for DAO (weight = sqrt(token balance)) to reduce whale dominance
- **Vote delegation** for DAO (token holders can delegate voting power to delegates)
- **Ranked-choice voting** (runoff elections) for multi-option proposals
- **Emergency pause** for multi-sig (veto any pending operation)
- **Time-weighted voting** for DAO (voting power decays post-snapshot)
- **Cross-contract governance** (DAO can vote on behalf of other contracts)

---

## References

- **SEP-40 Oracle Standard:** https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0040.md
- **Soroban SDK:** https://docs.rs/soroban-sdk/
- **Stellar Smart Contracts:** https://developers.stellar.org/docs/smart-contracts/overview

---

**Author:** Kiro AI  
**Last Updated:** 2026-07-28  
**Next Review:** Post-implementation
