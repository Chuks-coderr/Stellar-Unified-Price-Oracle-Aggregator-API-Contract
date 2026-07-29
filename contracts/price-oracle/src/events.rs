use soroban_sdk::{contractevent, Address, Bytes, String, Symbol};

/// Publishes a generic admin-action audit event.
///
/// Used by every admin-mutating function to emit a consistent on-chain audit trail.
/// Callers pass a short `action` symbol (≤8 chars), the acting `admin` address, and
/// optional arbitrary `data` bytes (may be empty).
#[allow(deprecated)]
pub fn emit_admin_action(env: &soroban_sdk::Env, action: Symbol, admin: Address, data: Bytes) {
    env.events().publish((action, admin), (data,));
}

// ContractInitializedEvent uses manual publishing due to String field
// limitations with the macro in soroban-sdk 26.

/// Emitted when a source submits a new price for an asset.
///
/// Topics: `asset`, `source`
#[contractevent]
#[derive(Clone)]
pub struct PriceSubmittedEvent {
    /// Address of the asset whose price was submitted.
    #[topic]
    pub asset: Address,
    /// Address of the oracle source that submitted the price.
    #[topic]
    pub source: Address,
    /// Raw price value scaled by `10^decimals`.
    pub price: i128,
    /// Unix timestamp (seconds) provided by the source.
    pub timestamp: u64,
}

/// Emitted when a new optimistic price proposal is created.
///
/// Topics: `asset`, `proposer`
#[contractevent]
#[derive(Clone)]
pub struct PriceProposalCreatedEvent {
    /// Address of the asset for which the proposal was made.
    #[topic]
    pub asset: Address,
    /// Address of the proposer.
    #[topic]
    pub proposer: Address,
    /// Monotonic proposal id assigned by the contract.
    pub proposal_id: u32,
    /// Proposed price value.
    pub price: i128,
    /// Proposed timestamp.
    pub timestamp: u64,
    /// Bond amount posted for the proposal.
    pub bond_amount: i128,
    /// Ledger at which the proposal becomes final if not disputed.
    pub expires_at_ledger: u32,
}

/// Emitted when an optimistic price proposal is disputed.
///
/// Topics: `proposal_id`, `disputer`
#[contractevent]
#[derive(Clone)]
pub struct PriceProposalDisputedEvent {
    /// Proposal id being disputed.
    #[topic]
    pub proposal_id: u32,
    /// Address of the disputer.
    #[topic]
    pub disputer: Address,
    /// Bond amount posted by the disputer.
    pub bond_amount: i128,
}

/// Emitted when an optimistic price proposal is resolved.
///
/// Topics: `proposal_id`
#[contractevent]
#[derive(Clone)]
pub struct PriceProposalResolvedEvent {
    /// Proposal id being resolved.
    #[topic]
    pub proposal_id: u32,
    /// Whether the proposal was accepted by the admin.
    pub approved: bool,
    /// Whether the proposal was finalized into an aggregate price.
    pub finalized: bool,
}

/// Emitted when the aggregate price for an asset changes.
///
/// Topics: `asset`
#[allow(dead_code)]
#[contractevent]
#[derive(Clone)]
pub struct PriceUpdatedEvent {
    /// Address of the asset whose aggregate price changed.
    #[topic]
    pub asset: Address,
    /// Newly computed aggregate price.
    pub new_price: i128,
    /// Previous aggregate price before this update.
    pub old_price: i128,
    /// Unix timestamp of the new aggregate.
    pub timestamp: u64,
    /// Unix timestamp of the previous aggregate.
    pub prev_timestamp: u64,
    /// Decimal precision applied to both price values.
    pub decimals: u32,
}

/// Emitted when a new oracle source is registered by the admin.
///
/// Topics: `source`, `admin`
#[contractevent]
#[derive(Clone)]
pub struct SourceAddedEvent {
    /// Address of the newly added oracle source.
    #[topic]
    pub source: Address,
    /// Address of the admin who performed the action.
    #[topic]
    pub admin: Address,
    /// Human-readable display name assigned to the source.
    pub name: String,
}

/// Emitted when an oracle source is de-registered by the admin.
///
/// Topics: `source`, `admin`
#[contractevent]
#[derive(Clone)]
pub struct SourceRemovedEvent {
    /// Address of the removed oracle source.
    #[topic]
    pub source: Address,
    /// Address of the admin who performed the action.
    #[topic]
    pub admin: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct SourceAssetAddedEvent {
    #[topic]
    pub source: Address,
    #[topic]
    pub asset: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct SourceAssetRemovedEvent {
    #[topic]
    pub source: Address,
    #[topic]
    pub asset: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct SourceVerificationSetEvent {
    #[topic]
    pub source: Address,
    pub verified: bool,
    pub verification_method: String,
    pub verifier: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct SourceKeyRotatedEvent {
    #[topic]
    pub old_source: Address,
    #[topic]
    pub new_source: Address,
    pub ledger: u32,
}

/// Emitted when a new asset is registered for price tracking.
///
/// Topics: `asset`, `admin`
#[contractevent]
#[derive(Clone)]
pub struct AssetRegisteredEvent {
    /// Address of the newly registered asset.
    #[topic]
    pub asset: Address,
    /// Address of the admin who registered the asset.
    #[topic]
    pub admin: Address,
}

/// Emitted when a previously registered asset is removed.
///
/// Topics: `asset`, `admin`
#[contractevent]
#[derive(Clone)]
pub struct AssetUnregisteredEvent {
    /// Address of the asset that was removed.
    #[topic]
    pub asset: Address,
    /// Address of the admin who removed the asset.
    #[topic]
    pub admin: Address,
}

/// Emitted when the contract administrator is replaced.
///
/// Topics: `old_admin`, `new_admin`
#[contractevent]
#[derive(Clone)]
pub struct AdminChangedEvent {
    /// Address of the outgoing administrator.
    #[topic]
    pub old_admin: Address,
    /// Address of the incoming administrator.
    #[topic]
    pub new_admin: Address,
}

/// Emitted when the contract's WASM is upgraded to a new hash.
///
/// Topics: `new_wasm_hash`
#[contractevent]
#[derive(Clone)]
pub struct ContractUpgradedEvent {
    /// 32-byte hash of the new WASM module.
    #[topic]
    pub new_wasm_hash: soroban_sdk::BytesN<32>,
}

/// Emitted when `min_sources_required` is updated.
#[contractevent]
#[derive(Clone)]
pub struct MinSourcesChangedEvent {
    /// The new minimum-sources threshold.
    pub value: u32,
}

/// Emitted when `max_history_length` is updated.
#[contractevent]
#[derive(Clone)]
pub struct MaxHistoryChangedEvent {
    /// The new maximum history length (in entries per asset).
    pub value: u32,
}

/// Emitted when the price resolution window is updated.
#[contractevent]
#[derive(Clone)]
pub struct ResolutionChangedEvent {
    /// The new resolution value in seconds.
    pub value: u32,
}

/// Emitted when the decimal precision setting is updated.
#[contractevent]
#[derive(Clone)]
pub struct DecimalsChangedEvent {
    /// The new number of decimals.
    pub value: u32,
}

/// Emitted when the contract description is updated.
#[contractevent]
#[derive(Clone)]
pub struct DescriptionChangedEvent {
    /// The new human-readable description string.
    pub description: String,
}

/// Emitted when a price aggregation attempt fails due to too few contributing sources.
///
/// Topics: `asset`
#[contractevent]
#[derive(Clone)]
pub struct SourcesInsufficientEvent {
    /// Address of the asset for which aggregation failed.
    #[topic]
    pub asset: Address,
    /// Number of sources that had submitted prices at the time of the attempt.
    pub current_source_count: u32,
    /// Minimum number of sources required for aggregation to succeed.
    pub min_sources_required: u32,
}

/// Publishes the contract-initialized event.
///
/// Uses manual event publishing because `String` fields are not yet supported
/// by the `#[contractevent]` macro in soroban-sdk 26.
///
/// # Arguments
///
/// * `env` - The Soroban execution environment.
/// * `admin` - Address set as the initial administrator.
/// * `min_sources` - Effective minimum-sources threshold (after defaulting).
/// * `max_history` - Effective maximum-history length (after defaulting).
/// * `decimals` - Decimal precision configured at initialization.
/// * `description` - Human-readable description string.
#[allow(deprecated)]
pub fn emit_initialized(
    env: &soroban_sdk::Env,
    admin: Address,
    min_sources: u32,
    max_history: u32,
    decimals: u32,
    description: String,
) {
    let sym = soroban_sdk::symbol_short!("init");
    env.events().publish(
        (sym, admin),
        (min_sources, max_history, decimals, description),
    );
}

/// Emitted each time a successful price aggregation occurs for an asset.
///
/// Topics: `asset`
#[contractevent]
#[derive(Clone)]
pub struct PriceAggregatedEvent {
    /// Address of the asset whose price was aggregated.
    #[topic]
    pub asset: Address,
    /// Newly computed aggregate price.
    pub price: i128,
    /// Number of sources that contributed to this aggregate.
    pub num_sources: u32,
    /// Unix timestamp of the most-recent contributing submission.
    pub timestamp: u64,
}

/// Emitted when an asset's circuit breaker trips and the update is rejected.
///
/// Topics: `asset`
#[contractevent]
#[derive(Clone)]
pub struct CircuitBreakerTrippedEvent {
    /// Address of the asset that triggered the breaker.
    #[topic]
    pub asset: Address,
    /// Previous aggregate price before the rejected update.
    pub previous_price: i128,
    /// Candidate aggregate price that would have been published.
    pub candidate_price: i128,
    /// Change amount in basis points that exceeded the configured limit.
    pub change_bps: u32,
    /// Maximum allowed change in basis points for a single ledger.
    pub max_change_bps: u32,
    /// Ledger at which the breaker tripped.
    pub ledger: u32,
    /// Unix timestamp of the breaker trip.
    pub timestamp: u64,
}

/// Emitted when the circuit breaker is manually reset by the admin.
///
/// Topics: `asset`, `admin`
#[contractevent]
#[derive(Clone)]
pub struct CircuitBreakerResetEvent {
    /// Address of the asset whose breaker was reset.
    #[topic]
    pub asset: Address,
    /// Admin who reset the breaker.
    #[topic]
    pub admin: Address,
}

/// Emitted when the oldest history entry for an asset is pruned to enforce `max_history_length`.
///
/// Topics: `asset`
#[contractevent]
#[derive(Clone)]
pub struct HistoryPrunedEvent {
    /// Address of the asset whose history was pruned.
    #[topic]
    pub asset: Address,
    /// Ledger sequence number of the entry that was removed.
    pub pruned_ledger: u32,
    /// Number of history entries remaining after pruning.
    pub remaining: u32,
}

/// Publishes the timestamp-threshold-changed event.
///
/// Uses manual event publishing because `u64` values in `#[contractevent]` trigger
/// a macro limitation in soroban-sdk 26.
///
/// # Arguments
///
/// * `env` - The Soroban execution environment.
/// * `admin` - Address of the admin who made the change.
/// * `value` - New timestamp threshold in seconds.
#[allow(deprecated)]
pub fn emit_timestamp_threshold_changed(env: &soroban_sdk::Env, admin: Address, value: u64) {
    let sym = soroban_sdk::symbol_short!("tthr");
    env.events().publish((sym, admin), (value,));
}

/// Emitted when a source's submitted price deviates excessively from the current aggregate.
///
/// Topics: `asset`, `source`
#[allow(dead_code)]
#[contractevent]
#[derive(Clone)]
pub struct PriceDeviationFlaggedEvent {
    /// Address of the asset for which the deviation was detected.
    #[topic]
    pub asset: Address,
    /// Address of the source whose submission triggered the flag.
    #[topic]
    pub source: Address,
    /// Price submitted by the flagged source.
    pub price: i128,
    /// Current aggregate (median) price used as the reference.
    pub median_price: i128,
    /// Deviation magnitude expressed as a percentage (0–100).
    pub deviation_percent: u32,
}

/// Publishes the max-price-deviation-changed event.
///
/// Uses manual event publishing because the `#[contractevent]` macro does not
/// yet support all field types cleanly in soroban-sdk 26.
///
/// # Arguments
///
/// * `env` - The Soroban execution environment.
/// * `admin` - Address of the admin who made the change.
/// * `value` - New maximum deviation in basis points (100 bp = 1 %).
#[allow(deprecated)]
pub fn emit_max_price_deviation_changed(env: &soroban_sdk::Env, admin: Address, value: u32) {
    let sym = soroban_sdk::symbol_short!("devn");
    env.events().publish((sym, admin), (value,));
}

/// Emitted when an oracle source submits a liveness heartbeat.
///
/// Topics: `source`
#[contractevent]
#[derive(Clone)]
pub struct SourceHeartbeatEvent {
    /// Address of the source that submitted the heartbeat.
    #[topic]
    pub source: Address,
    /// Unix timestamp of the ledger at which the heartbeat was recorded.
    pub timestamp: u64,
}

/// Emitted when a source is detected as inactive (heartbeat overdue).
///
/// Topics: `source`
#[contractevent]
#[derive(Clone)]
pub struct SourceInactiveEvent {
    /// Address of the source that was flagged inactive.
    #[topic]
    pub source: Address,
    /// Unix timestamp of the source's last recorded heartbeat.
    pub last_heartbeat: u64,
}

/// Emitted when the heartbeat interval is updated.
#[contractevent]
#[derive(Clone)]
pub struct HeartbeatIntervalChangedEvent {
    /// New heartbeat interval in seconds.
    pub value: u64,
}

/// Emitted when a previously inactive source submits a new heartbeat and becomes active.
///
/// Topics: `source`
#[contractevent]
#[derive(Clone)]
pub struct SourceActiveAgainEvent {
    /// Address of the source that resumed activity.
    #[topic]
    pub source: Address,
    /// Unix timestamp at which the source became active again.
    pub timestamp: u64,
}

/// Emitted when the contract is paused by the admin.
///
/// Topics: `admin`
#[contractevent]
#[derive(Clone)]
pub struct ContractPausedEvent {
    /// Address of the admin who paused the contract.
    #[topic]
    pub admin: Address,
}

/// Emitted when the contract is unpaused by the admin.
///
/// Topics: `admin`
#[contractevent]
#[derive(Clone)]
pub struct ContractUnpausedEvent {
    /// Address of the admin who unpaused the contract.
    #[topic]
    pub admin: Address,
}

/// Emitted when a stale price is detected during a read operation.
///
/// Topics: `asset`
#[contractevent]
#[derive(Clone)]
pub struct PriceStaleEvent {
    /// Address of the asset whose price was considered stale.
    #[topic]
    pub asset: Address,
    /// Ledger sequence number when the aggregate was last written (0 if unavailable).
    pub last_update_ledger: u32,
    /// Current ledger sequence number at the time of detection.
    pub current_ledger: u32,
}

/// Emitted when an admin proposes a new timelock-protected operation.
///
/// Topics: `proposed_by`
#[contractevent]
#[derive(Clone)]
pub struct OperationProposedEvent {
    /// Unique ID assigned to this pending operation.
    pub operation_id: u32,
    /// Numeric discriminant of the [`OperationType`](crate::types::OperationType).
    pub op_type: u32,
    /// Address of the admin who proposed this operation.
    #[topic]
    pub proposed_by: Address,
    /// Ledger sequence number when the operation was proposed.
    pub proposed_ledger: u32,
}

/// Emitted when a timelock-protected operation is successfully executed.
///
/// Topics: `executed_by`
#[contractevent]
#[derive(Clone)]
pub struct OperationExecutedEvent {
    /// ID of the operation that was executed.
    pub operation_id: u32,
    /// Numeric discriminant of the [`OperationType`](crate::types::OperationType).
    pub op_type: u32,
    /// Address of the admin who executed the operation.
    #[topic]
    pub executed_by: Address,
}

/// Emitted when a pending timelock operation is cancelled by the admin.
///
/// Topics: `cancelled_by`
#[contractevent]
#[derive(Clone)]
pub struct OperationCancelledEvent {
    /// ID of the operation that was cancelled.
    pub operation_id: u32,
    /// Numeric discriminant of the [`OperationType`](crate::types::OperationType).
    pub op_type: u32,
    /// Address of the admin who cancelled the operation.
    #[topic]
    pub cancelled_by: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct PriceOverrideSetEvent {
    #[topic]
    pub asset: Address,
    #[topic]
    pub admin: Address,
    pub price: i128,
    pub reason: String,
    pub expiry_ledger: u32,
}

#[contractevent]
#[derive(Clone)]
pub struct PriceOverrideRemovedEvent {
    #[topic]
    pub asset: Address,
    #[topic]
    pub admin: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct PriceOverrideExpiredEvent {
    #[topic]
    pub asset: Address,
    pub expiry_ledger: u32,
    pub current_ledger: u32,
}

/// Emitted when the query rate limit is updated.
#[contractevent]
#[derive(Clone)]
pub struct QueryRateLimitChangedEvent {
    /// The new query rate limit value.
    pub value: u32,
}

/// Emitted when a rate limit is exceeded for an address.
///
/// Topics: `consumer`
#[contractevent]
#[derive(Clone)]
pub struct RateLimitExceededEvent {
    /// Address that exceeded the rate limit.
    #[topic]
    pub consumer: Address,
    /// Current count of operations.
    pub current_count: u32,
    /// The rate limit threshold.
    pub limit: u32,
}

/// Emitted when a subscription is created for a consumer.
///
/// Topics: `consumer`, `duration`
#[contractevent]
#[derive(Clone)]
pub struct SubscriptionCreatedEvent {
    /// Address of the consumer who created the subscription.
    #[topic]
    pub consumer: Address,
    /// Duration of the subscription in seconds.
    #[topic]
    pub duration: u64,
}

/// Emitted when a subscription is renewed by a consumer.
///
/// Topics: `consumer`
#[contractevent]
#[derive(Clone)]
pub struct SubscriptionRenewedEvent {
    /// Address of the consumer who renewed the subscription.
    #[topic]
    pub consumer: Address,
}

/// Emitted when a subscription expires for a consumer.
///
/// Topics: `consumer`
#[contractevent]
#[derive(Clone)]
pub struct SubscriptionExpiredEvent {
    /// Address of the consumer whose subscription expired.
    #[topic]
    pub consumer: Address,
}

// --- #67: Per-asset resolution ---

/// Emitted when the per-asset resolution is set or cleared.
#[contractevent]
#[derive(Clone)]
pub struct AssetResolutionSetEvent {
    #[topic]
    pub asset: Address,
    #[topic]
    pub admin: Address,
    /// Resolution in seconds (0 = cleared, falls back to contract-wide).
    pub resolution: u32,
}

// --- #69: Periodic aggregation trigger ---

/// Emitted when trigger_aggregation is called and aggregation succeeds.
#[contractevent]
#[derive(Clone)]
pub struct AggregationTriggeredEvent {
    #[topic]
    pub asset: Address,
    pub price: i128,
    pub num_sources: u32,
    pub triggered_at_ledger: u32,
}

/// Emitted when the aggregation cooldown is updated.
#[contractevent]
#[derive(Clone)]
pub struct AggCooldownChangedEvent {
    pub cooldown_ledgers: u32,
}

// --- #70: Min submission interval ---

/// Emitted when the minimum submission interval is updated.
#[contractevent]
#[derive(Clone)]
pub struct SubmitIntervalChangedEvent {
    pub interval_ledgers: u32,
}

/// Emitted when a source is flagged as non-compliant for an asset.
#[contractevent]
#[derive(Clone)]
pub struct SourceNonCompliantEvent {
    #[topic]
    pub source: Address,
    #[topic]
    pub asset: Address,
    pub last_submission_ledger: u32,
    pub required_interval: u32,
}

// --- #68: Batch operations ---

/// Emitted when an admin proposes a new batch of operations.
#[contractevent]
#[derive(Clone)]
pub struct BatchProposedEvent {
    pub batch_id: u32,
    pub num_operations: u32,
    #[topic]
    pub proposed_by: Address,
    pub proposed_ledger: u32,
}

/// Emitted when a batch is successfully executed.
#[contractevent]
#[derive(Clone)]
pub struct BatchExecutedEvent {
    pub batch_id: u32,
    pub num_operations: u32,
    #[topic]
    pub executed_by: Address,
}

/// Emitted when a pending batch is cancelled.
#[contractevent]
#[derive(Clone)]
pub struct BatchCancelledEvent {
    pub batch_id: u32,
    #[topic]
    pub cancelled_by: Address,
}

// #65 reputation events
#[contractevent]
#[derive(Clone)]
pub struct SourceReputationUpdatedEvent {
    #[topic]
    pub source: Address,
    pub old_score: i128,
    pub new_score: i128,
}

#[contractevent]
#[derive(Clone)]
pub struct ReputationDecayChangedEvent {
    pub value: u32,
}

// #66 phased removal events
#[contractevent]
#[derive(Clone)]
pub struct SourceMarkedForRemovalEvent {
    #[topic]
    pub source: Address,
    #[topic]
    pub admin: Address,
    pub eligible_at_ledger: u32,
}

#[contractevent]
#[derive(Clone)]
pub struct SourceRemovalCancelledEvent {
    #[topic]
    pub source: Address,
    #[topic]
    pub admin: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct RemovalCooldownChangedEvent {
    pub value: u32,
}

// =============================================================================
// #186 — Adaptive Heartbeat / Liveness Detection
// =============================================================================

/// Emitted when a source is automatically removed due to extended inactivity
/// (exceeding `max_inactive_ledgers` without a reactivating heartbeat+price).
///
/// Topics: `source`
#[contractevent]
#[derive(Clone)]
pub struct SourceAutoRemovedEvent {
    /// Address of the source that was automatically removed.
    #[topic]
    pub source: Address,
    /// Ledger at which the source first became inactive.
    pub inactive_since_ledger: u32,
    /// Current ledger when auto-removal was executed.
    pub removed_at_ledger: u32,
    /// Number of consecutive missed heartbeats at time of removal.
    pub missed_heartbeats: u32,
}

/// Emitted when a source's health status changes (e.g., Healthy → Degraded → Inactive).
///
/// Topics: `source`
#[contractevent]
#[derive(Clone)]
pub struct SourceHealthChangedEvent {
    /// Address of the source whose health changed.
    #[topic]
    pub source: Address,
    /// Old health status as a `u32` discriminant (0=Healthy,1=Degraded,2=Inactive,3=AutoRemoved).
    pub old_status: u32,
    /// New health status as a `u32` discriminant.
    pub new_status: u32,
    /// Consecutive missed-heartbeat count at time of change.
    pub missed_heartbeats: u32,
}

/// Emitted when the max_inactive_ledgers configuration is changed.
#[contractevent]
#[derive(Clone)]
pub struct InactiveLedgersChangedEvent {
    /// The new maximum inactive ledgers threshold.
    pub value: u32,
}

/// Emitted when the heartbeat window size configuration is changed.
#[contractevent]
#[derive(Clone)]
pub struct HeartbeatWindowChangedEvent {
    /// The new heartbeat window size (number of periods).
    pub value: u32,
}

// =============================================================================
// #187 — Commit-Reveal MEV Resistance
// =============================================================================

/// Emitted when a source commits a price hash for a given round.
///
/// Topics: `asset`, `source`
#[contractevent]
#[derive(Clone)]
pub struct PriceCommittedEvent {
    /// Address of the asset being committed.
    #[topic]
    pub asset: Address,
    /// Address of the committing source.
    #[topic]
    pub source: Address,
    /// Ledger round this commit belongs to.
    pub round_ledger: u32,
    /// Ledger at which the commit was made.
    pub committed_at_ledger: u32,
}

/// Emitted when a source successfully reveals a committed price.
///
/// Topics: `asset`, `source`
#[contractevent]
#[derive(Clone)]
pub struct PriceRevealedEvent {
    /// Address of the asset whose price was revealed.
    #[topic]
    pub asset: Address,
    /// Address of the source revealing the price.
    #[topic]
    pub source: Address,
    /// The revealed price value.
    pub price: i128,
    /// Round ledger this reveal belongs to.
    pub round_ledger: u32,
    /// Ledger at which the reveal was processed.
    pub revealed_at_ledger: u32,
}

/// Emitted when a commit expires without being revealed (reveal window closed).
///
/// Topics: `asset`, `source`
#[contractevent]
#[derive(Clone)]
pub struct CommitExpiredEvent {
    /// Address of the asset.
    #[topic]
    pub asset: Address,
    /// Address of the source that committed but did not reveal.
    #[topic]
    pub source: Address,
    /// The round ledger that has now expired.
    pub round_ledger: u32,
}

/// Emitted when the commit window configuration is changed.
#[contractevent]
#[derive(Clone)]
pub struct CommitWindowChangedEvent {
    /// New commit window in ledgers.
    pub value: u32,
}

/// Emitted when the reveal window configuration is changed.
#[contractevent]
#[derive(Clone)]
pub struct RevealWindowChangedEvent {
    /// New reveal window in ledgers.
    pub value: u32,
}

// =============================================================================
// #188 — Economic Finality Gadget
// =============================================================================

/// Emitted when a pending price entry transitions to finalized status.
///
/// Topics: `asset`
#[contractevent]
#[derive(Clone)]
pub struct PriceFinalizedEvent {
    /// Address of the asset whose price was finalized.
    #[topic]
    pub asset: Address,
    /// The finalized price value.
    pub price: i128,
    /// Ledger at which the price was originally aggregated.
    pub committed_ledger: u32,
    /// Ledger at which finality was confirmed.
    pub finalized_ledger: u32,
    /// Number of contributing sources.
    pub num_sources: u32,
}

/// Emitted when an admin retracts a pending price before finalization (reorg protection).
///
/// Topics: `asset`, `admin`
#[contractevent]
#[derive(Clone)]
pub struct PriceRetractedEvent {
    /// Address of the asset whose pending price was retracted.
    #[topic]
    pub asset: Address,
    /// Address of the admin who executed the retraction.
    #[topic]
    pub admin: Address,
    /// Ledger of the pending price that was retracted.
    pub committed_ledger: u32,
    /// Ledger at which the retraction occurred.
    pub retracted_at_ledger: u32,
}

/// Emitted when a reorg is detected via ledger hash chain inconsistency.
///
/// Topics: `asset`
#[contractevent]
#[derive(Clone)]
pub struct ReorgDetectedEvent {
    /// Address of the affected asset.
    #[topic]
    pub asset: Address,
    /// Ledger at which the hash chain inconsistency was detected.
    pub detected_at_ledger: u32,
    /// The committed ledger whose price is now suspect.
    pub suspect_ledger: u32,
}

/// Emitted when the finality_ledgers configuration is changed.
#[contractevent]
#[derive(Clone)]
pub struct FinalityLedgersChangedEvent {
    /// New finality ledgers count.
    pub value: u32,
}

/// Emitted when a new price is placed in the pending-finality queue.
///
/// Topics: `asset`
#[contractevent]
#[derive(Clone)]
pub struct PricePendingFinalityEvent {
    /// Address of the asset.
    #[topic]
    pub asset: Address,
    /// The price value pending finalization.
    pub price: i128,
    /// Ledger at which aggregation occurred.
    pub committed_ledger: u32,
    /// Ledger after which the price will be considered final.
    pub finality_ledger: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// #171: Source Reputation & Slashing Events
// ─────────────────────────────────────────────────────────────────────────────

/// Emitted when an oracle source stakes tokens into contract custody.
#[contractevent]
#[derive(Clone)]
pub struct SourceStakedEvent {
    /// Address of the source that staked.
    #[topic]
    pub source: Address,
    /// Amount staked in this transaction (stroops).
    pub amount: i128,
    /// New total stake after this transaction (stroops).
    pub total_stake: i128,
}

/// Emitted when a source's staked tokens are returned upon deregistration.
#[contractevent]
#[derive(Clone)]
pub struct SourceUnstakedEvent {
    /// Address of the source whose stake was returned.
    #[topic]
    pub source: Address,
    /// Amount returned (may be less than original stake if slashed).
    pub amount_returned: i128,
}

/// Emitted when an admin slashes a portion of a source's locked stake.
#[contractevent]
#[derive(Clone)]
pub struct SourceSlashedEvent {
    /// Address of the slashed source.
    #[topic]
    pub source: Address,
    /// Amount slashed (moved to treasury) in stroops.
    pub slash_amount: i128,
    /// Remaining stake after slashing.
    pub remaining_stake: i128,
    /// Configured slash percentage applied.
    pub slash_percent: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// #172: Cross-Asset Correlation Events
// ─────────────────────────────────────────────────────────────────────────────

/// Emitted when a correlation ratio band is configured or updated.
#[contractevent]
#[derive(Clone)]
pub struct CorrelationBandSetEvent {
    /// Base asset of the pair.
    #[topic]
    pub base_asset: Address,
    /// Quote asset of the pair.
    #[topic]
    pub quote_asset: Address,
    /// Minimum acceptable ratio (scaled by RATIO_PRECISION = 10^7).
    pub min_ratio: u128,
    /// Maximum acceptable ratio (scaled by RATIO_PRECISION = 10^7).
    pub max_ratio: u128,
    /// Whether the check is currently enabled.
    pub enabled: bool,
}

/// Emitted when a submitted price causes a correlation ratio violation.
#[contractevent]
#[derive(Clone)]
pub struct CorrelationViolationEvent {
    /// Base asset of the violated pair.
    #[topic]
    pub base_asset: Address,
    /// Quote asset of the violated pair.
    #[topic]
    pub quote_asset: Address,
    /// Source that submitted the out-of-band price.
    #[topic]
    pub source: Address,
    /// The price just submitted.
    pub submitted_price: i128,
    /// The current aggregate price of the counterpart asset.
    pub counterpart_price: i128,
    /// Computed ratio (scaled by RATIO_PRECISION).
    pub ratio: u128,
    /// Configured minimum ratio.
    pub min_ratio: u128,
    /// Configured maximum ratio.
    pub max_ratio: u128,
}

/// Emitted when a (source, asset) price is flagged and excluded from aggregation
/// due to a correlation violation.
#[contractevent]
#[derive(Clone)]
pub struct CorrelationPriceFlaggedEvent {
    /// The asset whose submitted price was flagged.
    #[topic]
    pub asset: Address,
    /// The source whose submission was flagged.
    #[topic]
    pub source: Address,
    /// The price value that triggered the flag.
    pub flagged_price: i128,
}

// ─────────────────────────────────────────────────────────────────────────────
// #173: Tiered Consumer Access Events
// ─────────────────────────────────────────────────────────────────────────────

/// Emitted when a new consumer registers with a tier.
#[contractevent]
#[derive(Clone)]
pub struct ConsumerRegisteredEvent {
    /// Address of the newly registered consumer.
    #[topic]
    pub consumer: Address,
    /// Tier discriminant (0=Free, 1=Basic, 2=Premium).
    pub tier: u32,
    /// Unix timestamp when the subscription expires (0 = no expiry for Free tier).
    pub subscription_expiry_ts: u64,
}

/// Emitted when a consumer changes to a different tier.
#[contractevent]
#[derive(Clone)]
pub struct ConsumerTierChangedEvent {
    /// Address of the consumer changing tiers.
    #[topic]
    pub consumer: Address,
    /// Old tier discriminant.
    pub old_tier: u32,
    /// New tier discriminant.
    pub new_tier: u32,
}

/// Emitted when a subscription fee is paid.
#[contractevent]
#[derive(Clone)]
pub struct TierFeePaidEvent {
    /// Consumer that paid the fee.
    #[topic]
    pub consumer: Address,
    /// Tier discriminant the fee was paid for.
    pub tier: u32,
    /// Amount paid in stroops.
    pub amount: i128,
}

// ─────────────────────────────────────────────────────────────────────────────
// #174: Price Deviation Alert Events
// ─────────────────────────────────────────────────────────────────────────────

/// Emitted when a consumer successfully subscribes to price deviation alerts.
#[contractevent]
#[derive(Clone)]
pub struct AlertSubscribedEvent {
    /// Address of the subscribing consumer.
    #[topic]
    pub consumer: Address,
    /// Asset being monitored.
    #[topic]
    pub asset: Address,
    /// Movement threshold in basis points.
    pub threshold_bps: u32,
    /// TTL in ledgers for this subscription.
    pub ttl_ledgers: u32,
}

/// Emitted when an alert threshold is breached and a callback is dispatched.
#[contractevent]
#[derive(Clone)]
pub struct AlertTriggeredEvent {
    /// Subscriber that was notified.
    #[topic]
    pub consumer: Address,
    /// Asset whose price moved.
    #[topic]
    pub asset: Address,
    /// Previous aggregate price.
    pub old_price: i128,
    /// New aggregate price.
    pub new_price: i128,
    /// Actual price movement in basis points.
    pub movement_bps: u32,
    /// The configured threshold that was exceeded.
    pub threshold_bps: u32,
}

/// Emitted when a consumer's callback invocation fails.
#[contractevent]
#[derive(Clone)]
pub struct AlertCallbackFailedEvent {
    /// Subscriber whose callback failed.
    #[topic]
    pub consumer: Address,
    /// Asset being monitored.
    #[topic]
    pub asset: Address,
}

/// Emitted when a subscription expires and is pruned.
#[contractevent]
#[derive(Clone)]
pub struct AlertSubscriptionExpiredEvent {
    /// Consumer whose subscription expired.
    #[topic]
    pub consumer: Address,
    /// Asset the subscription was for.
    #[topic]
    pub asset: Address,
    /// Ledger at which expiry was detected.
    pub expired_ledger: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Off-chain relayer network integration events
// ─────────────────────────────────────────────────────────────────────────────

/// Emitted when the admin approves a new relayer.
///
/// Topics: `relayer`, `admin`
#[contractevent]
#[derive(Clone)]
pub struct RelayerAddedEvent {
    /// Address of the newly approved relayer.
    #[topic]
    pub relayer: Address,
    /// Address of the admin who approved the relayer.
    #[topic]
    pub admin: Address,
    /// Human-readable display name for the relayer.
    pub name: String,
}

/// Emitted when the admin revokes a relayer's approval.
///
/// Topics: `relayer`, `admin`
#[contractevent]
#[derive(Clone)]
pub struct RelayerRemovedEvent {
    /// Address of the relayer whose approval was revoked.
    #[topic]
    pub relayer: Address,
    /// Address of the admin who performed the revocation.
    #[topic]
    pub admin: Address,
}

/// Emitted when an approved relayer successfully submits a price on behalf of a source.
///
/// Topics: `asset`, `source`, `relayer`
#[contractevent]
#[derive(Clone)]
pub struct PriceRelayedEvent {
    /// Address of the asset being priced.
    #[topic]
    pub asset: Address,
    /// Address of the oracle source whose price data was relayed.
    #[topic]
    pub source: Address,
    /// Address of the relayer that submitted the transaction.
    #[topic]
    pub relayer: Address,
    /// Raw price value scaled by `10^decimals`.
    pub price: i128,
    /// Unix timestamp (seconds) of the price observation.
    pub timestamp: u64,
}

/// Publishes the relayer-fee-changed event.
///
/// Uses manual event publishing because `i128` fields in `#[contractevent]` may
/// trigger edge cases in some tooling.
///
/// # Arguments
///
/// * `env` - The Soroban execution environment.
/// * `admin` - Address of the admin who set the new fee.
/// * `fee` - New fee per submission in stroops.
#[allow(deprecated)]
pub fn emit_relayer_fee_set(env: &soroban_sdk::Env, admin: Address, fee: i128) {
    let sym = soroban_sdk::symbol_short!("rfee");
    env.events().publish((sym, admin), (fee,));
}

// ─────────────────────────────────────────────────────────────────────────────
// Cross-reference oracle check events
// ─────────────────────────────────────────────────────────────────────────────

/// Emitted when a cross-reference check detects that our price deviates from a reference
/// oracle's price by more than the configured threshold.
///
/// Topics: `asset`, `ref_contract`
#[contractevent]
#[derive(Clone)]
pub struct CrossRefDeviationEvent {
    /// Address of the asset for which the deviation was detected.
    #[topic]
    pub asset: Address,
    /// Contract address of the reference oracle that reported the diverging price.
    #[topic]
    pub ref_contract: Address,
    /// Our current aggregated price for the asset.
    pub our_price: i128,
    /// Price reported by the reference oracle.
    pub ref_price: i128,
    /// Absolute deviation between the two prices in basis points (1 % = 100 bps).
    pub deviation_bps: u32,
    /// Configured deviation threshold (in basis points) that was exceeded.
    pub threshold_bps: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// #92/#93/#94: history cap, event spam protection, max aggregation sources
// ─────────────────────────────────────────────────────────────────────────────

/// Emitted when per-asset history is pruned beyond `max_history_per_asset` (issue #94).
///
/// Topics: `asset`
#[contractevent]
#[derive(Clone)]
pub struct HistoryPerAssetPrunedEvent {
    #[topic]
    pub asset: Address,
    /// Ledger removed from the history index.
    pub pruned_ledger: u32,
    /// Remaining entry count after pruning.
    pub remaining: u32,
}

/// Emitted when the `max_history_per_asset` limit is changed (issue #94).
#[contractevent]
#[derive(Clone)]
pub struct HistoryPerAssetChangedEvent {
    pub value: u32,
}

/// Emitted when the event-per-call cap is exceeded in a single invocation (issue #92).
/// The transaction still succeeds; this is a warning only.
///
/// Topics: `asset`
#[contractevent]
#[derive(Clone)]
pub struct EventLimitWarningEvent {
    #[topic]
    pub asset: Address,
    /// Number of events that would have been emitted.
    pub event_count: u32,
    /// Configured cap that was exceeded.
    pub max_events: u32,
}

/// Emitted when the `max_events_per_call` limit is changed (issue #92).
#[contractevent]
#[derive(Clone)]
pub struct EventsPerCallChangedEvent {
    pub value: u32,
}

/// Emitted when the `max_aggregation_sources` limit is changed (issue #93).
#[contractevent]
#[derive(Clone)]
pub struct MaxAggSourcesChangedEvent {
    pub value: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// #112: Storage migration events
// ─────────────────────────────────────────────────────────────────────────────

/// Emitted when a storage migration is resumed from a previously saved cursor.
#[contractevent]
#[derive(Clone)]
pub struct MigrationResumedEvent {
    #[topic]
    pub admin: Address,
    pub cursor: u32,
}

/// Emitted when a new storage migration begins.
#[contractevent]
#[derive(Clone)]
pub struct MigrationStartedEvent {
    #[topic]
    pub admin: Address,
    pub from_version: u32,
    pub to_version: u32,
    pub started_ledger: u32,
}

/// Emitted when a storage migration finishes processing all items.
#[contractevent]
#[derive(Clone)]
pub struct MigrationCompletedEvent {
    #[topic]
    pub admin: Address,
    pub from_version: u32,
    pub to_version: u32,
    pub items_processed: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Misc admin config events
// ─────────────────────────────────────────────────────────────────────────────

/// Emitted when historical-price interpolation is enabled or disabled.
#[contractevent]
#[derive(Clone)]
pub struct InterpolationChangedEvent {
    pub enabled: bool,
}

/// Emitted when the maximum number of registered oracle sources is changed.
#[contractevent]
#[derive(Clone)]
pub struct MaxSourcesChangedEvent {
    pub value: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// #210: Progressive Disqualification Events
// ─────────────────────────────────────────────────────────────────────────────

/// Emitted when a source accumulates enough demerits to trigger a warning.
#[contractevent]
#[derive(Clone)]
pub struct SourceWarningEvent {
    #[topic]
    pub source: Address,
    pub demerits: u32,
}

/// Emitted when a source accumulates enough demerits to be placed on probation.
#[contractevent]
#[derive(Clone)]
pub struct SourceProbationEvent {
    #[topic]
    pub source: Address,
    pub demerits: u32,
}

/// Emitted when a source accumulates enough demerits to be disqualified.
#[contractevent]
#[derive(Clone)]
pub struct SourceDisqualifiedEvent {
    #[topic]
    pub source: Address,
    pub demerits: u32,
    pub status_updated_ledger: u32,
}

/// Emitted when a source's demerits and disqualification status are reset by the admin.
#[contractevent]
#[derive(Clone)]
pub struct SourceDemeritsResetEvent {
    #[topic]
    pub source: Address,
    #[topic]
    pub admin: Address,
}

/// Emitted when the global demerit configuration is changed.
#[contractevent]
#[derive(Clone)]
pub struct DemeritConfigChangedEvent {
    #[topic]
    pub admin: Address,
    pub warning_threshold: u32,
    pub probation_threshold: u32,
    pub disqualified_threshold: u32,
    pub cooldown_ledgers: u32,
}

/// Emitted when an invalid price submission is recorded against a source.
#[contractevent]
#[derive(Clone)]
pub struct InvalidSubmissionRecordedEvent {
    #[topic]
    pub source: Address,
    pub demerits: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// #207: Multi-sig Source Governance Events
// ─────────────────────────────────────────────────────────────────────────────

/// Emitted when source governance config is updated.
#[contractevent]
#[derive(Clone)]
pub struct SourceGovConfigChangedEvent {
    #[topic]
    pub admin: Address,
    pub threshold: u32,
    pub approvers_count: u32,
}

/// Emitted when a new source proposal is proposed.
#[contractevent]
#[derive(Clone)]
pub struct SourceProposalCreatedEvent {
    #[topic]
    pub proposal_id: u32,
    #[topic]
    pub proposer: Address,
    #[topic]
    pub source: Address,
    pub name: String,
}

/// Emitted when an approver approves a source proposal.
#[contractevent]
#[derive(Clone)]
pub struct SourceProposalApprovedEvent {
    #[topic]
    pub proposal_id: u32,
    #[topic]
    pub approver: Address,
}

/// Emitted when a source proposal is executed (threshold met).
#[contractevent]
#[derive(Clone)]
pub struct SourceProposalExecutedEvent {
    #[topic]
    pub proposal_id: u32,
    #[topic]
    pub source: Address,
}

// ─────────────────────────────────────────────────────────────────────────────
// #208: Source Geolocation Events
// ─────────────────────────────────────────────────────────────────────────────

/// Emitted when geolocation metadata for a source is updated.
#[contractevent]
#[derive(Clone)]
pub struct SourceGeoUpdatedEvent {
    #[topic]
    pub source: Address,
    pub region: String,
    pub provider: String,
    pub jurisdiction: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// #209: Source Heartbeat Liveness Bond Events
// ─────────────────────────────────────────────────────────────────────────────

/// Emitted when the required source bond amount is changed.
#[contractevent]
#[derive(Clone)]
pub struct SourceBondConfigChangedEvent {
    #[topic]
    pub admin: Address,
    pub amount: i128,
}

/// Emitted when a source deposits its liveness bond.
#[contractevent]
#[derive(Clone)]
pub struct SourceBondDepositedEvent {
    #[topic]
    pub source: Address,
    pub amount: i128,
}

/// Emitted when a source bond is forfeited.
#[contractevent]
#[derive(Clone)]
pub struct SourceBondForfeitedEvent {
    #[topic]
    pub source: Address,
    pub amount: i128,
}

/// Emitted when a source bond is returned.
#[contractevent]
#[derive(Clone)]
pub struct SourceBondReturnedEvent {
    #[topic]
    pub source: Address,
    pub amount: i128,
}





// =============================================================================
// Missing events for feature modules
// =============================================================================

/// Emitted when asset metadata is updated.
#[contractevent]
#[derive(Clone)]
pub struct AssetMetadataUpdatedEvent {
    #[topic]
    pub asset: Address,
    #[topic]
    pub admin: Address,
}

/// Circuit breaker event entry (used as a struct in some older modules).
/// NOTE: This is a struct, not an event, kept here for backward compatibility.
#[derive(Clone)]
#[soroban_sdk::contracttype]
pub struct CircuitBreakerEventEntry {
    pub asset: Address,
    pub previous_price: i128,
    pub candidate_price: i128,
    pub change_bps: u32,
    pub max_change_bps: u32,
    pub ledger: u32,
    pub timestamp: u64,
}

/// Emitted when a price is submitted with a deadline (#202).
#[contractevent]
#[derive(Clone)]
pub struct PriceSubmittedWithDeadlineEvent {
    #[topic]
    pub asset: Address,
    #[topic]
    pub source: Address,
    pub price: i128,
    pub timestamp: u64,
    pub deadline_ledger: u32,
}

/// Emitted when a submission rebate is distributed (#202).
#[contractevent]
#[derive(Clone)]
pub struct RebateDistributedEvent {
    #[topic]
    pub source: Address,
    #[topic]
    pub asset: Address,
    pub amount: i128,
}

/// Emitted when an exotic asset pricing config is set (#177).
#[contractevent]
#[derive(Clone)]
pub struct ExoticAssetConfigSetEvent {
    #[topic]
    pub asset: Address,
    #[topic]
    pub admin: Address,
}

/// Emitted when the fee market minimum priority fee is changed (#176).
#[contractevent]
#[derive(Clone)]
pub struct FmMinPriorityFeeChangedEvent {
    pub value: u128,
}

/// Emitted when the fee distribution ratio is changed (#176).
#[contractevent]
#[derive(Clone)]
pub struct FmFeeDistributionRatioChangedEvent {
    pub ratio_bps: u32,
}

/// Emitted when a fee market submission is enqueued (#176).
#[contractevent]
#[derive(Clone)]
pub struct FmSubmissionEnqueuedEvent {
    #[topic]
    pub source: Address,
    #[topic]
    pub asset: Address,
    pub priority_fee: u128,
    pub queue_position: u32,
}

/// Emitted when a fee market submission is processed (#176).
#[contractevent]
#[derive(Clone)]
pub struct FmSubmissionProcessedEvent {
    #[topic]
    pub source: Address,
    #[topic]
    pub asset: Address,
    pub price: i128,
}

/// Emitted when multi-sig governors list is updated (#178).
#[contractevent]
#[derive(Clone)]
pub struct MsGovernorsUpdatedEvent {
    #[topic]
    pub admin: Address,
    pub governor_count: u32,
    pub required_approvals: u32,
}

/// Emitted when a multi-sig operation is proposed (#178).
#[contractevent]
#[derive(Clone)]
pub struct MsOperationProposedEvent {
    pub op_id: u32,
    pub op_type: u32,
    #[topic]
    pub proposed_by: Address,
    pub proposed_ledger: u32,
}

/// Emitted when multi-sig quorum is reached (#178).
#[contractevent]
#[derive(Clone)]
pub struct MsQuorumReachedEvent {
    pub op_id: u32,
    pub approval_count: u32,
}

/// Emitted when a governor approves a multi-sig operation (#178).
#[contractevent]
#[derive(Clone)]
pub struct MsOperationApprovedEvent {
    pub op_id: u32,
    #[topic]
    pub approver: Address,
}

/// Emitted when a multi-sig operation is retracted before execution (#178).
#[contractevent]
#[derive(Clone)]
pub struct MsOperationRetractedEvent {
    pub op_id: u32,
    #[topic]
    pub retracted_by: Address,
}

/// Emitted when a multi-sig operation is executed (#178).
#[contractevent]
#[derive(Clone)]
pub struct MsOperationExecutedEvent {
    pub op_id: u32,
    pub op_type: u32,
    #[topic]
    pub executed_by: Address,
}

/// Emitted when a multi-sig operation is cancelled (#178).
#[contractevent]
#[derive(Clone)]
pub struct MsOperationCancelledEvent {
    pub op_id: u32,
    #[topic]
    pub cancelled_by: Address,
}

/// Emitted when a source fee credit is recorded.
#[contractevent]
#[derive(Clone)]
pub struct SourceFeeCreditedEvent {
    #[topic]
    pub source: Address,
    pub amount: i128,
    pub total_balance: i128,
}

/// Emitted when a source withdraws accumulated fees.
#[contractevent]
#[derive(Clone)]
pub struct SourceFeesWithdrawnEvent {
    #[topic]
    pub source: Address,
    pub amount: i128,
}

/// Emitted when a ZK verifying key is set (#175).
#[contractevent]
#[derive(Clone)]
pub struct ZkVerifyingKeySetEvent {
    #[topic]
    pub admin: Address,
}

/// Emitted when a ZK-verified price is submitted (#175).
#[contractevent]
#[derive(Clone)]
pub struct ZkPriceSubmittedEvent {
    #[topic]
    pub asset: Address,
    #[topic]
    pub source: Address,
    pub price: i128,
    pub timestamp: u64,
}

/// Emitted when a challenge is submitted (#235).
#[contractevent]
#[derive(Clone)]
pub struct ChallengePricedEvent {
    #[topic]
    pub asset: Address,
    #[topic]
    pub challenger: Address,
    pub challenge_id: u32,
    pub expected_price: i128,
}

/// Emitted when a challenge is resolved (#235).
#[contractevent]
#[derive(Clone)]
pub struct ChallengeResolvedEvent {
    pub challenge_id: u32,
    pub valid: bool,
    pub reward: i128,
}

/// Emitted when challenger rewards are claimed (#235).
#[contractevent]
#[derive(Clone)]
pub struct RewardsClaimedEvent {
    #[topic]
    pub challenger: Address,
    pub amount: i128,
}

/// Emitted when a source rotation schedule is set (#206).
#[contractevent]
#[derive(Clone)]
pub struct SourceRotationScheduleSetEvent {
    #[topic]
    pub asset: Address,
    #[topic]
    pub admin: Address,
    pub rotation_interval: u32,
}

/// Emitted when sources are rotated for an asset (#206).
#[contractevent]
#[derive(Clone)]
pub struct SourcesRotatedEvent {
    #[topic]
    pub asset: Address,
    pub rotated_at_ledger: u32,
}

/// Emitted when an admin audit entry is appended (#239).
#[contractevent]
#[derive(Clone)]
pub struct AdminAuditEntryAppendedEvent {
    #[topic]
    pub admin: Address,
    pub entry_id: u32,
}

/// Emitted when a role is delegated (#241).
#[contractevent]
#[derive(Clone)]
pub struct RoleDelegatedEvent {
    #[topic]
    pub delegator: Address,
    #[topic]
    pub delegatee: Address,
    pub role: u32,
}

/// Emitted when a role is revoked (#241).
#[contractevent]
#[derive(Clone)]
pub struct RoleRevokedEvent {
    #[topic]
    pub revoker: Address,
    #[topic]
    pub holder: Address,
    pub role: u32,
}

/// Emitted when an emergency pause is triggered (#240).
#[contractevent]
#[derive(Clone)]
pub struct EmergencyPausedEvent {
    #[topic]
    pub admin: Address,
    pub auto_unpause_ledger: u32,
}

/// Emitted when an emergency pause is lifted (#240).
#[contractevent]
#[derive(Clone)]
pub struct EmergencyUnpausedEvent {
    #[topic]
    pub admin: Address,
}

/// Emitted when an emergency pause duration is extended (#240).
#[contractevent]
#[derive(Clone)]
pub struct EmergencyPauseExtendedEvent {
    #[topic]
    pub admin: Address,
    pub new_unpause_ledger: u32,
}

/// Emitted when an asset TTL extension is performed (#203).
#[contractevent]
#[derive(Clone)]
pub struct AssetTtlExtendedEvent {
    #[topic]
    pub asset: Address,
    pub num_extended: u32,
    pub current_ledger: u32,
}

/// Emitted when the rate limit tier is changed.
#[contractevent]
#[derive(Clone)]
pub struct RateLimitTierChangedEvent {
    pub tier: u32,
    pub limit: u32,
}

// Emitted when an invalid submission is recorded against a source (re-export from events).
// Already defined elsewhere, but needed here as well.
// Note: InvalidSubmissionRecordedEvent is already defined above; this is the canonical copy.

/// Emitted when an admin freezes an asset's price during a market emergency (#223).
#[contractevent]
#[derive(Clone)]
pub struct PriceFrozenEvent {
    #[topic]
    pub asset: Address,
    pub reason: String,
    pub price: i128,
    pub frozen_at_ledger: u32,
}

/// Emitted when an admin unfreezes a previously frozen asset (#223).
#[contractevent]
#[derive(Clone)]
pub struct PriceUnfrozenEvent {
    #[topic]
    pub asset: Address,
    pub unfrozen_at_ledger: u32,
}

/// Emitted when an admin registers a notification preference for an event type (#243).
#[contractevent]
#[derive(Clone)]
pub struct NotifPrefSetEvent {
    #[topic]
    pub event_type: u32,
    pub channel: String,
    pub target: String,
}

/// Emitted when an admin clears all notification preferences for an event type (#243).
#[contractevent]
#[derive(Clone)]
pub struct NotifPrefsClearedEvent {
    #[topic]
    pub event_type: u32,
}
