# Test Strategy for Governance Features

**Features:** #236 (Multi-sig), #242 (Time-based), #224 (EMA), #237 (DAO)

---

## Test Organization

Tests will be added to `contracts/price-oracle/src/test.rs` (currently 2,000+ lines, 76 tests).

### New Test Modules Structure

```rust
#[cfg(test)]
mod tests {
    // Existing: test_initialize, test_admin, test_sources, etc. (76 tests)
    
    // NEW TESTS (~49 total)
    
    mod multisig_tests {
        // Phase 1: #236 (15 tests)
    }
    
    mod timelock_scheduling_tests {
        // Phase 2: #242 (8 tests)
    }
    
    mod ema_smoothing_tests {
        // Phase 3: #224 (6 tests)
    }
    
    mod dao_governance_tests {
        // Phase 4: #237 (20 tests)
    }
}
```

---

## Phase 1: Multi-Sig Tests (#236) — 15 Tests

### Test 1: Basic Governor Setup
```rust
#[test]
fn test_multisig_set_governors() {
    let env = Env::default();
    let admin = Address::random(&env);
    let gov1 = Address::random(&env);
    let gov2 = Address::random(&env);
    let gov3 = Address::random(&env);
    
    initialize_contract(&env, &admin);
    
    // Set 2-of-3 governors
    multisig::set_governors(&env, vec![&env, gov1, gov2, gov3], 2);
    
    let governors = multisig::get_governors(&env);
    assert_eq!(governors.len(), 3);
    assert_eq!(multisig::get_required_approvals(&env), 2);
}
```

### Test 2: Invalid Governor Threshold
```rust
#[test]
#[should_panic(expected = "InvalidConfiguration")]
fn test_multisig_threshold_exceeds_count() {
    let env = Env::default();
    let admin = Address::random(&env);
    
    initialize_contract(&env, &admin);
    
    // Try to require 5 approvals from 3 governors
    multisig::set_governors(&env, vec![&env, gov1, gov2, gov3], 5);
}
```

### Test 3: Propose Operation (Non-Governor Fails)
```rust
#[test]
#[should_panic(expected = "NotAuthorized")]
fn test_multisig_propose_non_governor() {
    let env = Env::default();
    let admin = Address::random(&env);
    let gov1 = Address::random(&env);
    let non_gov = Address::random(&env);
    
    initialize_contract(&env, &admin);
    multisig::set_governors(&env, vec![&env, gov1], 1);
    
    // Non-governor tries to propose
    multisig::propose_ms_operation(&env, non_gov, OperationType::SetMinSources, Bytes::new(&env));
}
```

### Test 4: 2-of-3 Approval Workflow
```rust
#[test]
fn test_multisig_2of3_approval_workflow() {
    let env = Env::default();
    let admin = Address::random(&env);
    let gov1 = Address::random(&env);
    let gov2 = Address::random(&env);
    let gov3 = Address::random(&env);
    
    initialize_contract(&env, &admin);
    multisig::set_governors(&env, vec![&env, gov1, gov2, gov3], 2);
    
    // Governor 1 proposes
    let op_id = multisig::propose_ms_operation(
        &env,
        gov1.clone(),
        OperationType::SetMinSources,
        encode_u32(&env, 5),
    );
    
    let status = multisig::get_ms_operation_status(&env, op_id);
    assert_eq!(status.approvals.len(), 0);
    assert_eq!(status.status, OperationStatus::Proposed);
    
    // Governor 1 approves (1/2)
    multisig::approve_ms_operation(&env, gov1.clone(), op_id);
    let status = multisig::get_ms_operation_status(&env, op_id);
    assert_eq!(status.approvals.len(), 1);
    assert_eq!(status.status, OperationStatus::Proposed);
    
    // Governor 2 approves (2/2) → crosses threshold
    multisig::approve_ms_operation(&env, gov2.clone(), op_id);
    let status = multisig::get_ms_operation_status(&env, op_id);
    assert_eq!(status.approvals.len(), 2);
    assert_eq!(status.status, OperationStatus::Approved);  // Now approved!
    
    // Check: approval_time_ledger set
    assert!(status.approval_time_ledger.is_some());
}
```

### Test 5: Double-Approval Prevention
```rust
#[test]
#[should_panic(expected = "NotAuthorized")]  // or custom error
fn test_multisig_cannot_approve_twice() {
    let env = Env::default();
    let admin = Address::random(&env);
    let gov1 = Address::random(&env);
    
    initialize_contract(&env, &admin);
    multisig::set_governors(&env, vec![&env, gov1], 1);
    
    let op_id = multisig::propose_ms_operation(&env, gov1.clone(), OperationType::SetMinSources, ...);
    multisig::approve_ms_operation(&env, gov1.clone(), op_id);
    
    // Try to approve again
    multisig::approve_ms_operation(&env, gov1.clone(), op_id);
}
```

### Test 6: Approval Revocation
```rust
#[test]
fn test_multisig_revoke_approval_before_threshold() {
    let env = Env::default();
    let admin = Address::random(&env);
    let gov1 = Address::random(&env);
    let gov2 = Address::random(&env);
    
    initialize_contract(&env, &admin);
    multisig::set_governors(&env, vec![&env, gov1, gov2], 2);
    
    let op_id = multisig::propose_ms_operation(&env, gov1.clone(), OperationType::SetMinSources, ...);
    multisig::approve_ms_operation(&env, gov1.clone(), op_id);
    
    let status = multisig::get_ms_operation_status(&env, op_id);
    assert_eq!(status.approvals.len(), 1);
    assert_eq!(status.status, OperationStatus::Proposed);
    
    // Governor 1 revokes approval
    multisig::revoke_ms_approval(&env, gov1.clone(), op_id);
    
    let status = multisig::get_ms_operation_status(&env, op_id);
    assert_eq!(status.approvals.len(), 0);
    assert_eq!(status.status, OperationStatus::Proposed);  // Back to Proposed
}
```

### Test 7: Cannot Revoke After Threshold Crossed
```rust
#[test]
#[should_panic(expected = "OperationAlreadyApproved")]
fn test_multisig_cannot_revoke_after_approved() {
    let env = Env::default();
    let admin = Address::random(&env);
    let gov1 = Address::random(&env);
    let gov2 = Address::random(&env);
    
    initialize_contract(&env, &admin);
    multisig::set_governors(&env, vec![&env, gov1, gov2], 2);
    
    let op_id = multisig::propose_ms_operation(&env, gov1.clone(), OperationType::SetMinSources, ...);
    multisig::approve_ms_operation(&env, gov1.clone(), op_id);
    multisig::approve_ms_operation(&env, gov2.clone(), op_id);  // Threshold reached
    
    // Try to revoke after approval
    multisig::revoke_ms_approval(&env, gov1.clone(), op_id);
}
```

### Test 8: Ordered Queue Execution
```rust
#[test]
fn test_multisig_ordered_queue_execution() {
    let env = Env::default();
    let admin = Address::random(&env);
    let gov1 = Address::random(&env);
    
    initialize_contract(&env, &admin);
    multisig::set_governors(&env, vec![&env, gov1], 1);
    
    // Propose operation A
    let op_a = multisig::propose_ms_operation(&env, gov1.clone(), OperationType::SetMinSources, ...);
    multisig::approve_ms_operation(&env, gov1.clone(), op_a);
    
    // Propose operation B
    let op_b = multisig::propose_ms_operation(&env, gov1.clone(), OperationType::SetMaxHistory, ...);
    multisig::approve_ms_operation(&env, gov1.clone(), op_b);
    
    // Queue head should be A
    assert_eq!(multisig::get_queue_head(&env), op_a);
    
    // Advance time to meet timelock
    env.ledger().set_sequence(10000 + 11);
    
    // Execute A (dequeues A, B becomes head)
    multisig::execute_ms_operation(&env, admin.clone(), op_a);
    assert_eq!(multisig::get_queue_head(&env), op_b);
    
    // Can only execute B now, not skip ahead
    multisig::execute_ms_operation(&env, admin.clone(), op_b);
}
```

### Test 9: Cancel Operation
```rust
#[test]
fn test_multisig_cancel_operation() {
    let env = Env::default();
    let admin = Address::random(&env);
    let gov1 = Address::random(&env);
    
    initialize_contract(&env, &admin);
    multisig::set_governors(&env, vec![&env, gov1], 1);
    
    let op_id = multisig::propose_ms_operation(&env, gov1.clone(), OperationType::SetMinSources, ...);
    multisig::approve_ms_operation(&env, gov1.clone(), op_id);
    
    // Admin cancels
    multisig::cancel_ms_operation(&env, admin.clone(), op_id);
    
    let status = multisig::get_ms_operation_status(&env, op_id);
    assert_eq!(status.status, OperationStatus::Cancelled);
}
```

### Test 10: Timelock After Multi-Sig Approval
```rust
#[test]
fn test_multisig_plus_timelock() {
    let env = Env::default();
    let admin = Address::random(&env);
    let gov1 = Address::random(&env);
    
    initialize_contract(&env, &admin);
    multisig::set_governors(&env, vec![&env, gov1], 1);
    timelock::set_timelock_duration(&env, 10);
    
    let op_id = multisig::propose_ms_operation(&env, gov1.clone(), OperationType::SetMinSources, ...);
    multisig::approve_ms_operation(&env, gov1.clone(), op_id);
    
    // Status: Approved, but timelock not met
    let status = multisig::get_ms_operation_status(&env, op_id);
    assert_eq!(status.status, OperationStatus::Approved);
    
    // Try to execute too early → fails
    let result = multisig::execute_ms_operation(&env, admin.clone(), op_id);
    assert_eq!(result, Err(ErrorCode::TimelockNotReady));
    
    // Advance time
    env.ledger().set_sequence(10000 + 11);
    
    // Now execute succeeds
    multisig::execute_ms_operation(&env, admin.clone(), op_id).unwrap();
    assert_eq!(multisig::get_ms_operation_status(&env, op_id).status, OperationStatus::Executed);
}
```

### Tests 11-15: Additional Edge Cases
- Test 11: Governor list update (change M-of-N after some proposals pending)
- Test 12: Non-existent operation → OperationNotFound
- Test 13: Already-executed operation → cannot approve/revoke
- Test 14: Authorization: only governor can approve/revoke (not random address)
- Test 15: Events: verify MsApprovalAddedEvent, MsApprovalReachedEvent emitted correctly



---

## Phase 2: Time-Based Scheduling Tests (#242) — 8 Tests

### Test 16: Propose Operation at Timestamp
```rust
#[test]
fn test_timelock_propose_at_timestamp() {
    let env = Env::default();
    let admin = Address::random(&env);
    initialize_contract(&env, &admin);
    
    let current_ts = env.ledger().timestamp();
    let target_ts = current_ts + 3600;  // 1 hour from now
    
    let op_id = timelock::propose_operation_at_timestamp(
        &env,
        OperationType::SetMinSources,
        encode_u32(&env, 5),
        target_ts,
    );
    
    let pending_op = timelock::get_pending_operation(&env, op_id);
    assert_eq!(pending_op.execution_timestamp, Some(target_ts));
}
```

### Test 17: Execute Too Early (Timestamp)
```rust
#[test]
#[should_panic(expected = "TimelockNotReady")]
fn test_timelock_execute_too_early_timestamp() {
    let env = Env::default();
    let admin = Address::random(&env);
    initialize_contract(&env, &admin);
    
    let current_ts = env.ledger().timestamp();
    let target_ts = current_ts + 3600;  // 1 hour from now
    
    let op_id = timelock::propose_operation_at_timestamp(&env, OperationType::SetMinSources, ..., target_ts);
    
    // Execute immediately (timestamp not reached)
    timelock::execute_operation(&env, op_id);
}
```

### Test 18: Execute at Timestamp
```rust
#[test]
fn test_timelock_execute_at_timestamp() {
    let env = Env::default();
    let admin = Address::random(&env);
    initialize_contract(&env, &admin);
    
    let current_ts = env.ledger().timestamp();
    let target_ts = current_ts + 3600;
    
    let op_id = timelock::propose_operation_at_timestamp(&env, OperationType::SetMinSources, encode_u32(&env, 5), target_ts);
    
    // Advance ledger time to target
    env.ledger().set_timestamp(target_ts);
    
    // Execute succeeds
    timelock::execute_operation(&env, op_id);
}
```

### Test 19: Propose Operation After Delay (Ledgers)
```rust
#[test]
fn test_timelock_propose_after_delay_ledgers() {
    let env = Env::default();
    let admin = Address::random(&env);
    initialize_contract(&env, &admin);
    
    let delay_ledgers = 100;
    
    let op_id = timelock::propose_operation_after_delay(
        &env,
        OperationType::SetMinSources,
        encode_u32(&env, 5),
        delay_ledgers,
    );
    
    let pending_op = timelock::get_pending_operation(&env, op_id);
    assert_eq!(pending_op.execution_ledger_offset, Some(delay_ledgers));
}
```

### Test 20: Execute After Delay (Ledgers)
```rust
#[test]
fn test_timelock_execute_after_delay_ledgers() {
    let env = Env::default();
    let admin = Address::random(&env);
    initialize_contract(&env, &admin);
    
    let start_ledger = env.ledger().sequence();
    let delay_ledgers = 100;
    
    let op_id = timelock::propose_operation_after_delay(&env, OperationType::SetMinSources, encode_u32(&env, 5), delay_ledgers);
    
    // Advance past delay
    env.ledger().set_sequence(start_ledger + delay_ledgers + 1);
    
    // Execute succeeds
    timelock::execute_operation(&env, op_id);
}
```

### Test 21: Multi-Sig + Time-Based Scheduling
```rust
#[test]
fn test_multisig_plus_timestamp_scheduling() {
    let env = Env::default();
    let admin = Address::random(&env);
    let gov1 = Address::random(&env);
    
    initialize_contract(&env, &admin);
    multisig::set_governors(&env, vec![&env, gov1], 1);
    timelock::set_timelock_duration(&env, 10);
    
    let current_ts = env.ledger().timestamp();
    let target_ts = current_ts + 7200;  // 2 hours
    
    // Governor proposes time-based operation
    let op_id = multisig::propose_ms_operation_at_timestamp(
        &env,
        gov1.clone(),
        OperationType::SetMinSources,
        encode_u32(&env, 5),
        target_ts,
    );
    
    multisig::approve_ms_operation(&env, gov1.clone(), op_id);
    
    // Cannot execute before timestamp
    let result = timelock::execute_operation(&env, op_id);
    assert_eq!(result, Err(ErrorCode::TimelockNotReady));
    
    // Advance time
    env.ledger().set_timestamp(target_ts);
    
    // Now execute succeeds
    timelock::execute_operation(&env, op_id).unwrap();
}
```

### Test 22-23: Edge Cases
- Test 22: Past timestamp → reject proposal
- Test 23: Cancel time-scheduled operation before execution

---

## Phase 3: EMA Smoothing Tests (#224) — 6 Tests

### Test 24: EMA Disabled by Default
```rust
#[test]
fn test_ema_disabled_by_default() {
    let env = Env::default();
    let admin = Address::random(&env);
    let asset = Address::random(&env);
    
    initialize_contract(&env, &admin);
    register_asset(&env, &asset);
    
    let alpha = admin::get_asset_ema_alpha(&env, &asset);
    assert_eq!(alpha, 0);  // Disabled
}
```

### Test 25: Set EMA Alpha
```rust
#[test]
fn test_set_asset_ema_alpha() {
    let env = Env::default();
    let admin = Address::random(&env);
    let asset = Address::random(&env);
    
    initialize_contract(&env, &admin);
    register_asset(&env, &asset);
    
    admin::set_asset_ema_alpha(&env, asset.clone(), 500);  // 5%
    
    let alpha = admin::get_asset_ema_alpha(&env, asset);
    assert_eq!(alpha, 500);
}
```

### Test 26: EMA Computation (50-50 Mix)
```rust
#[test]
fn test_ema_computation_50_50() {
    let env = Env::default();
    let admin = Address::random(&env);
    let asset = Address::random(&env);
    let source = Address::random(&env);
    
    initialize_contract(&env, &admin);
    register_asset(&env, &asset);
    add_source(&env, &source, "test_source");
    admin::set_asset_ema_alpha(&env, asset.clone(), 5000);  // 50%
    
    // First submission: price = 100
    submit_price(&env, &source, &asset, 100, env.ledger().timestamp());
    
    let agg1 = prices::get_price(&env, &asset);
    assert_eq!(agg1.price, 100);  // First EMA = aggregate
    
    // Second submission: price = 200
    submit_price(&env, &source, &asset, 200, env.ledger().timestamp());
    
    let agg2 = prices::get_price(&env, &asset);
    assert_eq!(agg2.price, 150);  // (200 * 0.5 + 100 * 0.5) = 150
}
```

### Test 27: EMA Computation (No Smoothing)
```rust
#[test]
fn test_ema_no_smoothing_alpha_zero() {
    let env = Env::default();
    let admin = Address::random(&env);
    let asset = Address::random(&env);
    let source = Address::random(&env);
    
    initialize_contract(&env, &admin);
    register_asset(&env, &asset);
    add_source(&env, &source, "test_source");
    admin::set_asset_ema_alpha(&env, asset.clone(), 0);  // Disabled
    
    submit_price(&env, &source, &asset, 100, env.ledger().timestamp());
    submit_price(&env, &source, &asset, 500, env.ledger().timestamp());
    
    let agg = prices::get_price(&env, &asset);
    assert_eq!(agg.price, 500);  // No smoothing: returns raw aggregate
}
```

### Test 28: EMA Full Replacement (Alpha 10000)
```rust
#[test]
fn test_ema_full_replacement_alpha_max() {
    let env = Env::default();
    let admin = Address::random(&env);
    let asset = Address::random(&env);
    let source = Address::random(&env);
    
    initialize_contract(&env, &admin);
    register_asset(&env, &asset);
    add_source(&env, &source, "test_source");
    admin::set_asset_ema_alpha(&env, asset.clone(), 10000);  // 100%
    
    submit_price(&env, &source, &asset, 100, env.ledger().timestamp());
    submit_price(&env, &source, &asset, 200, env.ledger().timestamp());
    
    let agg = prices::get_price(&env, &asset);
    assert_eq!(agg.price, 200);  // Full replacement: new price overwrites EMA
}
```

### Test 29: EMA Arithmetic Safety
```rust
#[test]
fn test_ema_arithmetic_safety() {
    let env = Env::default();
    let admin = Address::random(&env);
    let asset = Address::random(&env);
    let source = Address::random(&env);
    
    initialize_contract(&env, &admin);
    register_asset(&env, &asset);
    add_source(&env, &source, "test_source");
    admin::set_asset_ema_alpha(&env, asset.clone(), 5000);
    
    // Submit very large prices near i128::MAX
    let large_price = i128::MAX / 2;
    submit_price(&env, &source, &asset, large_price, env.ledger().timestamp());
    submit_price(&env, &source, &asset, large_price, env.ledger().timestamp());
    
    // Should not overflow
    let agg = prices::get_price(&env, &asset);
    assert!(agg.price > 0);
}
```

---

## Phase 4: DAO Governance Tests (#237) — 20 Tests

### Test 30: Set Governance Token Contract
```rust
#[test]
fn test_dao_set_governance_token_contract() {
    let env = Env::default();
    let admin = Address::random(&env);
    let token_contract = Address::random(&env);
    
    initialize_contract(&env, &admin);
    
    governance::set_governance_token_contract(&env, token_contract.clone());
    
    let stored = governance::get_governance_token_contract(&env);
    assert_eq!(stored, token_contract);
}
```

### Test 31: Create Proposal (Minimum Stake Check)
```rust
#[test]
#[should_panic(expected = "InsufficientStake")]
fn test_dao_create_proposal_insufficient_stake() {
    let env = Env::default();
    let admin = Address::random(&env);
    let token_contract = Address::random(&env);
    let proposer = Address::random(&env);
    
    initialize_contract(&env, &admin);
    governance::set_governance_token_contract(&env, token_contract.clone());
    
    // Mock: proposer has 1 token, min_proposal_stake is 1000 tokens
    // Proposer tries to create proposal → fails
    governance::create_proposal(
        &env,
        proposer,
        "Test Proposal",
        "Description",
        ProposalType::ParameterChange(OperationType::SetMinSources, ...),
    );
}
```

### Test 32: Create Proposal (Success)
```rust
#[test]
fn test_dao_create_proposal_success() {
    let env = Env::default();
    let admin = Address::random(&env);
    let token_contract = Address::random(&env);
    let proposer = Address::random(&env);
    
    initialize_contract(&env, &admin);
    governance::set_governance_token_contract(&env, token_contract.clone());
    
    // Mock: proposer has sufficient stake
    let proposal_id = governance::create_proposal(
        &env,
        proposer.clone(),
        "Test Proposal",
        "Description",
        ProposalType::ParameterChange(OperationType::SetMinSources, encode_u32(&env, 5)),
    );
    
    assert!(proposal_id > 0);
    
    let proposal = governance::get_proposal(&env, proposal_id);
    assert_eq!(proposal.proposer, proposer);
    assert_eq!(proposal.yes_votes, 0);
    assert_eq!(proposal.executed, false);
}
```

### Test 33: Vote on Proposal
```rust
#[test]
fn test_dao_vote_on_proposal() {
    let env = Env::default();
    let admin = Address::random(&env);
    let token_contract = Address::random(&env);
    let proposer = Address::random(&env);
    let voter1 = Address::random(&env);
    
    initialize_contract(&env, &admin);
    governance::set_governance_token_contract(&env, token_contract.clone());
    
    let proposal_id = governance::create_proposal(&env, proposer, "Test", "Desc", ...);
    
    // Mock: voter1 has 100 tokens at snapshot
    governance::vote(&env, voter1.clone(), proposal_id, VoteOption::Yes);
    
    let proposal = governance::get_proposal(&env, proposal_id);
    assert_eq!(proposal.yes_votes, 100);
}
```

### Test 34: Double-Vote Protection
```rust
#[test]
#[should_panic(expected = "AlreadyVoted")]
fn test_dao_double_vote_protection() {
    let env = Env::default();
    let admin = Address::random(&env);
    let token_contract = Address::random(&env);
    let proposer = Address::random(&env);
    let voter = Address::random(&env);
    
    initialize_contract(&env, &admin);
    governance::set_governance_token_contract(&env, token_contract.clone());
    
    let proposal_id = governance::create_proposal(&env, proposer, "Test", "Desc", ...);
    
    governance::vote(&env, voter.clone(), proposal_id, VoteOption::Yes);
    
    // Try to vote again
    governance::vote(&env, voter.clone(), proposal_id, VoteOption::No);
}
```

### Test 35: Voting Window Closure
```rust
#[test]
fn test_dao_voting_window_closure() {
    let env = Env::default();
    let admin = Address::random(&env);
    let token_contract = Address::random(&env);
    let proposer = Address::random(&env);
    let voter = Address::random(&env);
    
    initialize_contract(&env, &admin);
    governance::set_governance_token_contract(&env, token_contract.clone());
    
    let proposal_id = governance::create_proposal(&env, proposer, "Test", "Desc", ...);
    
    // Advance past voting window (e.g., 604800 ledgers)
    env.ledger().set_sequence(env.ledger().sequence() + 604801);
    
    // Vote after window closes → fails
    let result = governance::vote(&env, voter, proposal_id, VoteOption::Yes);
    assert_eq!(result, Err(ErrorCode::VotingWindowClosed));
}
```

### Test 36: Finalize Voting (Approved)
```rust
#[test]
fn test_dao_finalize_voting_approved() {
    let env = Env::default();
    let admin = Address::random(&env);
    let token_contract = Address::random(&env);
    let proposer = Address::random(&env);
    let voter1 = Address::random(&env);
    let voter2 = Address::random(&env);
    
    initialize_contract(&env, &admin);
    governance::set_governance_token_contract(&env, token_contract.clone());
    governance::set_voting_parameters(&env, 10, 604800);  // 10% quorum
    
    // Mock: total supply = 1000 tokens
    let proposal_id = governance::create_proposal(&env, proposer, "Test", "Desc", ...);
    
    // voter1: 600 tokens → votes Yes
    // voter2: 200 tokens → votes No
    // Yes: 600, No: 200, Abstain: 0, Total Supply: 1000
    // Yes%: 60%, No%: 20%, Participation: 80% (> 10% quorum)
    // Result: Approved
    
    governance::vote(&env, voter1, proposal_id, VoteOption::Yes);  // 600
    governance::vote(&env, voter2, proposal_id, VoteOption::No);   // 200
    
    env.ledger().set_sequence(env.ledger().sequence() + 604801);
    governance::finalize_voting(&env, proposal_id);
    
    let proposal = governance::get_proposal(&env, proposal_id);
    assert_eq!(proposal.approved, true);
}
```

### Test 37: Finalize Voting (Rejected - Below Quorum)
```rust
#[test]
fn test_dao_finalize_voting_rejected_quorum() {
    let env = Env::default();
    let admin = Address::random(&env);
    let token_contract = Address::random(&env);
    let proposer = Address::random(&env);
    let voter1 = Address::random(&env);
    
    initialize_contract(&env, &admin);
    governance::set_governance_token_contract(&env, token_contract.clone());
    governance::set_voting_parameters(&env, 50, 604800);  // 50% quorum required
    
    // Mock: total supply = 1000 tokens, only 100 vote
    let proposal_id = governance::create_proposal(&env, proposer, "Test", "Desc", ...);
    
    governance::vote(&env, voter1, proposal_id, VoteOption::Yes);  // 100
    
    env.ledger().set_sequence(env.ledger().sequence() + 604801);
    governance::finalize_voting(&env, proposal_id);
    
    let proposal = governance::get_proposal(&env, proposal_id);
    assert_eq!(proposal.approved, false);  // Quorum not met
}
```

### Test 38: Finalize Voting (Rejected - Below Threshold)
```rust
#[test]
fn test_dao_finalize_voting_rejected_threshold() {
    let env = Env::default();
    let admin = Address::random(&env);
    let token_contract = Address::random(&env);
    let proposer = Address::random(&env);
    let voter1 = Address::random(&env);
    let voter2 = Address::random(&env);
    
    initialize_contract(&env, &admin);
    governance::set_governance_token_contract(&env, token_contract.clone());
    governance::set_voting_parameters(&env, 10, 604800);  // 10% quorum
    
    // Mock: Yes: 400, No: 600, Total: 1000
    // Yes%: 40% (< 50% threshold)
    let proposal_id = governance::create_proposal(&env, proposer, "Test", "Desc", ...);
    
    governance::vote(&env, voter1, proposal_id, VoteOption::Yes);   // 400
    governance::vote(&env, voter2, proposal_id, VoteOption::No);    // 600
    
    env.ledger().set_sequence(env.ledger().sequence() + 604801);
    governance::finalize_voting(&env, proposal_id);
    
    let proposal = governance::get_proposal(&env, proposal_id);
    assert_eq!(proposal.approved, false);  // 40% < 50%
}
```

### Test 39: Execute Approved Proposal
```rust
#[test]
fn test_dao_execute_approved_proposal() {
    let env = Env::default();
    let admin = Address::random(&env);
    let token_contract = Address::random(&env);
    let proposer = Address::random(&env);
    let voter = Address::random(&env);
    let gov = Address::random(&env);
    
    initialize_contract(&env, &admin);
    governance::set_governance_token_contract(&env, token_contract.clone());
    multisig::set_governors(&env, vec![&env, gov], 1);
    timelock::set_timelock_duration(&env, 10);
    
    // Create and approve proposal
    let proposal_id = governance::create_proposal(
        &env,
        proposer,
        "Change Min Sources",
        "Set to 5",
        ProposalType::ParameterChange(OperationType::SetMinSources, encode_u32(&env, 5)),
    );
    
    governance::vote(&env, voter, proposal_id, VoteOption::Yes);  // Majority
    env.ledger().set_sequence(env.ledger().sequence() + 604801);
    governance::finalize_voting(&env, proposal_id);
    
    // Approved proposal enters multi-sig
    let gov_proposal = governance::execute_approved_proposal(&env, proposal_id);
    assert_eq!(gov_proposal.status, OperationStatus::Proposed);
}
```

### Tests 40-49: Additional Tests
- Test 40: Snapshot immutability (later token transfers don't affect voting power)
- Test 41: Change voting parameters (quorum, window) mid-process
- Test 42: Abstain votes (counted but don't affect yes/no ratio)
- Test 43: List proposals (pagination)
- Test 44: Get proposal details
- Test 45: Proposal with Upgrade operation
- Test 46: Proposal with AddSource operation
- Test 47: Proposal with SetEMAAlpha operation
- Test 48: DAO + Multi-sig + Time-based scheduling integration
- Test 49: Emergency veto (admin can veto any approved proposal)

---

## Test Execution Checklist

### Before Merging

- [ ] Run `cargo test -p price-oracle --lib` — all 125 tests pass (76 + 49)
- [ ] Run `cargo clippy -p price-oracle -- -D warnings` — zero warnings
- [ ] Run `cargo fmt --manifest-path contracts/price-oracle/Cargo.toml -- --check` — formatting OK
- [ ] All new tests have descriptive names and comments
- [ ] All panics use `panic_with_error!(env, ErrorCode::...)` pattern
- [ ] Events verified by spot-checking test output
- [ ] Edge cases covered (zero, max, negative, empty collections)
- [ ] Authorization checks verified for all admin functions
- [ ] Timelock checks verified for all sensitive operations

### Deployed Testnet

- [ ] Run full e2e test: `./scripts/e2e-testnet.sh`
- [ ] Manual 2-of-3 multi-sig vote scenario
- [ ] Manual time-scheduled operation
- [ ] Manual EMA smoothing observation
- [ ] Manual DAO proposal → execution
- [ ] Gas costs benchmarked and documented
