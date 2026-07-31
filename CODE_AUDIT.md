# Code Audit & Implementation Patterns

**For Features #237, #236, #224, #242**

---

## Part 1: Current Codebase Analysis

### Module Overview

| Module | Lines | Purpose | Status |
|--------|-------|---------|--------|
| `lib.rs` | 2,400+ | Contract entry point, 27+ endpoints | Active, wired |
| `types.rs` | 900+ | Data structures, storage keys, enums | Active |
| `storage.rs` | 350+ | Storage helpers, median computation | Active |
| `admin.rs` | 550+ | Admin management, configuration | Active |
| `prices.rs` | 1,400+ | Price submission, aggregation | Active |
| `sources.rs` | 800+ | Source registry, heartbeat tracking | Active |
| `timelock.rs` | 240+ | Timelock operations (single-admin) | Partial (needs multi-sig) |
| `multisig.rs` | 450+ | Multi-sig governance layer | **DEAD CODE** (not wired) |
| `events.rs` | 1,100+ | 40+ event types | Active |
| `test.rs` | 2,000+ | 76 unit tests | Passing |

### Key Patterns Used Throughout

#### 1. Storage Access Pattern
```rust
// Read with TTL extension
let key = DataKey::SomeKey;
if env.storage().persistent().has(&key) {
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}
let value = env.storage()
    .persistent()
    .get(&key)
    .unwrap_or(default_value);

// Write with TTL
env.storage().persistent().set(&key, &value);
env.storage()
    .persistent()
    .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
```

**Constants:**
```rust
const LEDGER_THRESHOLD: u32 = 518400;  // ~60 days
const LEDGER_BUMP: u32 = 518400;
```

#### 2. Authorization Pattern
```rust
let admin = get_admin(env);
admin.require_auth();  // Panics if caller != admin
```

#### 3. Error Handling Pattern
```rust
if some_error_condition {
    panic_with_error!(env, ErrorCode::SomeError);
}
```

#### 4. Event Emission Pattern
```rust
SomeEvent {
    field1: value1,
    field2: value2,
}.publish(env);
```

#### 5. Configuration Getter Pattern
```rust
pub fn get_some_config(env: &Env) -> SomeType {
    let key = DataKey::CfgSomeConfig;
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    }
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(DEFAULT_VALUE)
}
```

---

## Part 2: Multi-Sig Integration Points

### Current Multisig Module State

**File:** `multisig.rs` (450 lines)

**Existing Functions (compiled but unreachable):**
- `set_governors(env, governors, required)` ✓
- `get_governors(env)` ✓
- `get_required_approvals(env)` ✓
- `propose_ms_operation(env, proposer, op_type, data)` ✓
- `approve_ms_operation(env, approver, op_id)` ✓
- `get_ms_operation_status(env, op_id)` ✓
- `execute_ms_operation(env, executor, op_id)` ✓
- `cancel_ms_operation(env, canceller, op_id)` ✓

**What Needs to Change:**

1. Add to `lib.rs` `#[contractimpl]` block:
```rust
pub fn set_governors(env: Env, governors: Vec<Address>, required: u32) {
    multisig::set_governors(&env, governors, required);
}

pub fn get_governors(env: Env) -> Vec<Address> {
    multisig::get_governors(&env)
}

// ... etc for all multisig functions
```

2. Extend storage keys in `types.rs`:
   - Already has: `MsGovernors`, `MsRequiredApprovals`, `MsOp(u32)`, `MsQueueHead`, `MsQueueTail`, `MsOpCount`
   - Need to add: `MsOperationApprovals(u32)`, `MsApprovalTime(u32)`, `MsOperationStatus(u32)`

3. Update `MultiSigOperation` struct:
```rust
pub struct MultiSigOperation {
    pub id: u32,
    pub op_type: OperationType,
    pub data: Bytes,
    pub proposer: Address,
    pub proposed_ledger: u32,
    pub approvals: Vec<Address>,           // ADD THIS
    pub approval_time_ledger: Option<u32>, // ADD THIS (when N-th approval reached)
    pub status: OperationStatus,           // ADD THIS: Proposed | Approved | Executed | Cancelled
    pub next_id: u32,                      // Linked list: next op ID
}

pub enum OperationStatus {
    Proposed,
    Approved,
    Executed,
    Cancelled,
}
```

---

## Part 3: Time-Based Scheduling

### Timelock Module Extension

**File:** `timelock.rs` (240 lines)

**Current Execution Check:**
```rust
pub fn execute_operation(env: &Env, op_id: u32) {
    let admin = get_admin(env);
    admin.require_auth();
    
    let pending_op: PendingOperation = env
        .storage()
        .persistent()
        .get(&DataKey::TlPendingOp(op_id))
        .ok_or_else(|| panic_with_error!(env, ErrorCode::OperationNotFound))
        .unwrap();
    
    let timelock_duration: u32 = env
        .storage()
        .persistent()
        .get(&DataKey::CfgTimelockDuration)
        .unwrap_or(10);
    let current_ledger = env.ledger().sequence();
    let elapsed = current_ledger - pending_op.proposed_ledger;
    
    if elapsed < timelock_duration {
        panic_with_error!(env, ErrorCode::TimelockNotReady);
    }
    // ... execute
}
```

**What Needs to Change:**

1. Extend `PendingOperation`:
```rust
pub struct PendingOperation {
    pub id: u32,
    pub op_type: OperationType,
    pub data: Bytes,
    pub proposed_by: Address,
    pub proposed_ledger: u32,
    pub execution_timestamp: Option<u64>,  // ADD: Unix timestamp for time-based ops
    pub execution_ledger_offset: Option<u32>, // ADD: Fallback ledger-based offset
}
```

2. New execution check function:
```rust
fn check_execution_ready(env: &Env, pending_op: &PendingOperation) -> bool {
    if let Some(ts) = pending_op.execution_timestamp {
        // Time-based: check if current time >= target time
        let current_ts = env.ledger().timestamp();
        current_ts >= ts
    } else if let Some(offset) = pending_op.execution_ledger_offset {
        // Ledger-based: check if elapsed ledgers >= offset
        let current_ledger = env.ledger().sequence();
        current_ledger >= pending_op.proposed_ledger + offset
    } else {
        // Default timelock duration
        let timelock_duration: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::CfgTimelockDuration)
            .unwrap_or(10);
        let current_ledger = env.ledger().sequence();
        current_ledger >= pending_op.proposed_ledger + timelock_duration
    }
}
```

3. New proposal functions in `lib.rs`:
```rust
pub fn propose_operation_at_timestamp(
    env: Env,
    op_type: OperationType,
    data: Bytes,
    target_timestamp: u64,
) -> u32 {
    timelock::propose_operation_at_timestamp(&env, op_type, data, target_timestamp)
}

pub fn propose_operation_after_delay(
    env: Env,
    op_type: OperationType,
    data: Bytes,
    delay_ledgers: u32,
) -> u32 {
    timelock::propose_operation_after_delay(&env, op_type, data, delay_ledgers)
}
```

---

## Part 4: EMA Smoothing Implementation

### Prices Module Integration

**File:** `prices.rs` (1,400 lines)

**Current Aggregation Point:**
Located in `trigger_aggregation()` function. Computes median/mean/trimmed-mean, then stores in `DataKey::Aggregate(asset)`.

**What Needs to Change:**

1. Add EMA computation after aggregation:
```rust
// In trigger_aggregation(), after computing aggregate_price:

let ema_alpha = get_asset_ema_alpha(env, &asset);
if ema_alpha > 0 {
    let previous_ema = get_asset_ema_value(env, &asset)
        .unwrap_or(aggregate_price);
    
    // new_ema = (aggregate_price * alpha + previous_ema * (10000 - alpha)) / 10000
    let new_ema = {
        let weighted_new = aggregate_price
            .checked_mul(ema_alpha as i128)
            .ok_or_else(|| panic_with_error!(env, ErrorCode::ArithmeticOverflow))?;
        let weighted_old = previous_ema
            .checked_mul((10000 - ema_alpha) as i128)
            .ok_or_else(|| panic_with_error!(env, ErrorCode::ArithmeticOverflow))?;
        (weighted_new + weighted_old) / 10000
    };
    
    write_asset_ema_value(env, &asset, new_ema);
    
    AggregatePrice {
        price: new_ema,  // Return EMA, not raw aggregate
        timestamp,
        num_sources,
        decimals,
        is_override: false,
    }
} else {
    AggregatePrice {
        price: aggregate_price,  // No smoothing
        timestamp,
        num_sources,
        decimals,
        is_override: false,
    }
}
```

2. New storage helpers in `storage.rs`:
```rust
pub fn get_asset_ema_alpha(env: &Env, asset: &Address) -> u32 {
    let key = DataKey::AssetEMAAlpha(asset.clone());
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    }
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(0)  // Default: disabled
}

pub fn get_asset_ema_value(env: &Env, asset: &Address) -> Option<i128> {
    let key = DataKey::AssetEMAValue(asset.clone());
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    }
    env.storage().persistent().get(&key)
}

pub fn write_asset_ema_value(env: &Env, asset: &Address, value: i128) {
    let key = DataKey::AssetEMAValue(asset.clone());
    env.storage().persistent().set(&key, &value);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}
```

3. New admin endpoint in `admin.rs`:
```rust
pub fn set_asset_ema_alpha(env: &Env, asset: Address, alpha: u32) {
    let admin = get_admin(env);
    admin.require_auth();
    
    if alpha > 10000 {
        panic_with_error!(env, ErrorCode::InvalidConfiguration);
    }
    
    env.storage()
        .persistent()
        .set(&DataKey::AssetEMAAlpha(asset.clone()), &alpha);
    
    AssetEMASmoothingEnabledEvent {
        asset: asset.clone(),
        alpha,
    }
    .publish(env);
}
```

4. New query endpoint in `lib.rs`:
```rust
pub fn get_asset_ema(env: Env, asset: Address) -> Option<i128> {
    storage::get_asset_ema_value(&env, &asset)
}
```

