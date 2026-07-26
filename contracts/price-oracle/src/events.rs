use soroban_sdk::{contractevent, symbol_short, Address, Bytes, String, Symbol};

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
pub struct AggregationCooldownChangedEvent {
    pub cooldown_ledgers: u32,
}

// --- #70: Min submission interval ---

/// Emitted when the minimum submission interval is updated.
#[contractevent]
#[derive(Clone)]
pub struct MinSubmissionIntervalChangedEvent {
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
