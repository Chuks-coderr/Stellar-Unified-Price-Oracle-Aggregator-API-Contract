# Integration Guide: 4 Critical Features

This document provides step-by-step guidance for integrating the four new features into the existing codebase.

## What's Ready

✅ All 4 feature modules are fully implemented with comprehensive unit tests  
✅ Storage schemas defined and non-conflicting  
✅ Public contract endpoints added to `lib.rs`  
✅ Error codes added  
✅ Type definitions complete  

## What Needs Integration

The new features are **self-contained** and work independently. However, for full effectiveness, consider these integration points:

### 1. Feature #227: Per-Asset Decimals Integration

**Where to integrate:** `prices.rs` — price aggregation and storage

**Current code pattern:**
```rust
let decimals = get_decimals(env);  // Contract-wide
```

**Updated pattern:**
```rust
let decimals = per_asset_decimals::get_asset_decimals(env, &asset);
// Falls back to contract-wide if per-asset not set
```

**Files to modify:**
- `prices.rs` — in `submit_price()` and aggregation logic
- `history.rs` — when storing historical prices
- `storage.rs` — in helper functions that work with decimals

**Estimated effort:** 15 minutes (4-5 call sites)

---

### 2. Feature #226: Cross-Chain Verification Integration

**Current status:** Fully standalone  
**Integration level:** Optional (disabled by default)

**Recommended integration point:**

After aggregation in `prices.rs`:
```rust
// Optional cross-chain verification
if cross_chain_verify::is_cross_chain_verification_enabled(&env) {
    if let Some(cross_chain_entry) = 
        cross_chain_verify::get_cross_chain_price(&env, &asset, &oracle) {
        let verified = cross_chain_verify::verify_cross_chain_price(
            &env,
            aggregate_price,
            decimals,
            &cross_chain_entry,
        );
        if !verified {
            // Emit alert event
            // Optionally freeze aggregation
        }
    }
}
```

**Estimated effort:** 10 minutes (can be done later)

---

### 3. Feature #238: Admin Operation Limits Integration

**Where to integrate:** `admin.rs` and `assets.rs`

**Pattern for each admin operation:**

```rust
// Before operation
admin_op_limits::validate_admin_op_allowed(&env, operation_type)?;

// ... perform operation ...

// After operation succeeds
admin_op_limits::increment_admin_op_counter(&env, operation_type);
```

**Operations to guard (from `AdminOperationType` enum):**
- `add_source` (in `sources.rs`)
- `remove_source` (in `sources.rs`)
- `register_asset` (in `assets.rs`)
- `unregister_asset` (in `assets.rs`)
- `set_decimals` (in `admin.rs`)
- `set_resolution` (in `admin.rs`)

**Call sites:**
1. `sources::add_source()` — guard with op_type 0
2. `sources::remove_source()` — guard with op_type 1
3. `assets::register_asset()` — guard with op_type 2
4. `assets::unregister_asset()` — guard with op_type 3
5. `admin::set_decimals()` — guard with op_type 4
6. `admin::set_resolution()` — guard with op_type 5

**Example integration:**
```rust
pub fn add_source(env: &Env, address: Address, name: String) {
    let admin = get_admin(env);
    admin.require_auth();
    
    // NEW: Check daily limit
    admin_op_limits::validate_admin_op_allowed(env, 0); // AddSource
    
    // ... existing validation ...
    
    // ... register source ...
    
    // NEW: Increment counter on success
    admin_op_limits::increment_admin_op_counter(env, 0);
}
```

**Estimated effort:** 30 minutes (6 call sites with pattern application)

---

### 4. Feature #225: Submission Deadline Enforcement Integration

**Where to integrate:** `prices.rs` — in `submit_price()` and aggregation

**Integration point 1: Validate submissions**
```rust
// In submit_price() when storing submission
submission_deadline::validate_submission_deadline(&env, env.ledger().sequence())?;
```

**Integration point 2: Filter during aggregation**
```rust
// When collecting submissions for aggregation
let all_submissions = /* collect from all sources */;
let valid_submissions = submission_deadline::filter_valid_submissions(&env, all_submissions);
// Use valid_submissions for aggregation
```

**Note:** If no round is configured, all submissions pass (backward compatible)

**Estimated effort:** 15 minutes (2 call sites with straightforward integration)

---

## Implementation Checklist

- [ ] **Feature #227 Integration**
  - [ ] Modify `prices.rs` to use `get_asset_decimals()` in `submit_price()`
  - [ ] Modify aggregation logic to read decimals per asset
  - [ ] Modify history storage to use per-asset decimals
  - [ ] Test with multi-asset scenario (BTC=8, ETH=18, USDC=6)

- [ ] **Feature #226 Integration** (optional, can defer)
  - [ ] Add cross-chain verification check after aggregation
  - [ ] Emit alert event on deviation
  - [ ] Test with mock cross-chain prices

- [ ] **Feature #238 Integration**
  - [ ] Add limit check to `add_source()`
  - [ ] Add limit check to `remove_source()`
  - [ ] Add limit check to `register_asset()`
  - [ ] Add limit check to `unregister_asset()`
  - [ ] Add limit check to `set_decimals()`
  - [ ] Add limit check to `set_resolution()`
  - [ ] Test: perform 5 add_source calls (should succeed), 6th should fail
  - [ ] Test: verify counter resets at midnight UTC

- [ ] **Feature #225 Integration**
  - [ ] Add deadline validation to `submit_price()`
  - [ ] Add submission filtering to aggregation logic
  - [ ] Test: submissions within window accepted, outside rejected
  - [ ] Test: no round configured accepts all (backward compatibility)

- [ ] **Final Verification**
  - [ ] All tests pass: `cargo test -p price-oracle --lib`
  - [ ] No clippy warnings: `cargo clippy -p price-oracle -- -D warnings`
  - [ ] Code formatted: `cargo fmt --check`
  - [ ] Build succeeds: `cargo build -p price-oracle --target wasm32v1-none --release`

---

## Code Location Reference

### New Modules (fully implemented, ready to use)
- `src/per_asset_decimals.rs` — Feature #227
- `src/cross_chain_verify.rs` — Feature #226
- `src/admin_op_limits.rs` — Feature #238
- `src/submission_deadline.rs` — Feature #225

### Modified Files (types and infrastructure)
- `src/types.rs` — DataKey variants, structs, enums
- `src/errors.rs` — New error codes
- `src/lib.rs` — Public endpoints

### Files Needing Integration (existing code)
- `src/prices.rs` — aggregation and storage
- `src/admin.rs` — admin configuration
- `src/assets.rs` — asset management
- `src/sources.rs` — source management

---

## Testing After Integration

### Unit Test Verification
```bash
# Run all tests
cargo test -p price-oracle --lib

# Run specific feature tests
cargo test per_asset_decimals --lib
cargo test cross_chain_verify --lib
cargo test admin_op_limits --lib
cargo test submission_deadline --lib
```

### Integration Test Scenarios

**Scenario 1: Multi-Asset Decimals**
1. Register BTC (8 decimals), ETH (18 decimals), USDC (6 decimals)
2. Set per-asset decimals for each
3. Submit prices from sources
4. Verify aggregation uses correct decimals per asset
5. Verify history stores correct decimals

**Scenario 2: Admin Operation Limits**
1. Set AddSource limit to 3
2. Call add_source 3 times (all succeed)
3. Call add_source 4th time (should fail with OperationLimitExceeded)
4. Wait until next UTC day
5. Call add_source again (should succeed)

**Scenario 3: Submission Deadlines**
1. Start aggregation round (ledgers 100-200)
2. Submit price from ledger 99 (should fail: OutOfSubmissionWindow)
3. Submit price from ledger 100 (should succeed)
4. Submit price from ledger 201 (should fail: OutOfSubmissionWindow)
5. Clear round
6. Submit price from any ledger (should succeed)

**Scenario 4: Cross-Chain Verification**
1. Enable cross-chain verification
2. Set deviation threshold to 500 bps (5%)
3. Submit cross-chain price from external oracle
4. Verify our price against it (within threshold → pass)
5. Submit cross-chain price with 10% deviation
6. Verify our price against it (exceeds threshold → fail)

---

## Error Handling

When integrating limit checks, handle the new error codes:

```rust
use crate::types::ErrorCode;

// During implementation
admin_op_limits::validate_admin_op_allowed(&env, op_type)?;
// Or panic if preferred:
admin_op_limits::validate_admin_op_allowed(&env, op_type); // panics on limit

// Error code values
const OPERATION_LIMIT_EXCEEDED: u32 = 54;
const OUT_OF_SUBMISSION_WINDOW: u32 = 55;
```

---

## Performance Considerations

**Per-Asset Decimals (#227)**
- O(1) storage lookup per submission
- Falls back to contract-wide if not set
- No performance impact

**Cross-Chain Verification (#226)**
- Optional verification (disabled by default)
- One O(1) storage lookup per verification
- Decimal normalization uses simple arithmetic
- No impact on critical path

**Admin Operation Limits (#238)**
- O(1) check before operation
- O(1) counter increment after
- Day calculation: simple division (`timestamp / 86400`)
- No performance impact on price submission

**Submission Deadlines (#225)**
- O(1) validation per submission
- O(n) filtering during aggregation (where n = number of submissions)
- Backward compatible (no round = all accepted)
- Minimal performance impact

---

## Security Considerations

1. **Feature #227 (Per-Asset Decimals)**
   - Validated range [0, 18]
   - Prevents overflow in calculations
   - Audit trail via `set_ledger`

2. **Feature #226 (Cross-Chain Verification)**
   - Threshold validated [0, 9999] bps
   - Disabled by default
   - Decimal normalization prevents precision loss
   - Independent of price submission (read-only verification)

3. **Feature #238 (Admin Operation Limits)**
   - Daily limits can't be set to 0 (enforced)
   - Day epoch is UTC-based (no server clock exploitation)
   - Counters increment after operation succeeds (no partial state)
   - Separate counters per operation type

4. **Feature #225 (Submission Deadlines)**
   - Invalid windows rejected (end ≤ start)
   - Optional (backward compatible)
   - Submissions outside window excluded, not failed
   - Audit trail via `created_ledger`

---

## Rollback Plan

Each feature can be independently disabled:

- **#227**: Don't call `set_asset_decimals()` — falls back to contract-wide
- **#226**: Don't call `set_cross_chain_verification_enabled(true)` — disabled by default
- **#238**: Set limits to very high values (e.g., 1000 per day)
- **#225**: Don't call `start_aggregation_round()` — no round active = all accepted

---

## Support & Questions

For detailed implementation examples, refer to:
- `FEATURE_IMPLEMENTATION.md` — Technical design & algorithms
- Module test suites — Expected behavior and edge cases
- `lib.rs` — Public endpoint signatures

