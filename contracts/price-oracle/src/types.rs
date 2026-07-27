use soroban_sdk::{contracttype, Address, Bytes, BytesN, Map, String, Symbol, Vec};

pub use crate::errors::ErrorCode;

pub type SubscriptionPlans = Map<u32, i128>;

/// Storage keys used to address contract state in persistent, temporary, and instance storage.
///
/// ## Key Schema (namespace → variants)
///
/// | Namespace | Prefix | Variants |
/// |-----------|--------|----------|
/// | Admin identity | (none) | `Admin` |
/// | Global config | `Cfg` | `CfgMinSources`, `CfgMaxHistory`, `CfgResolution`, `CfgDecimals`, `CfgDescription`, `CfgTimestampThreshold`, `CfgMaxDeviation`, `CfgHeartbeatInterval`, `CfgMaxInvalidSubs`, `CfgAggregationMethod`, `CfgPauseFlag`, `CfgTimelockDuration` |
/// | Source registry | `Src` | `SrcActive(addr)`, `SrcRegistry`, `SrcHeartbeat(addr)`, `SrcInactive(addr)` |
/// | Asset registry | `Asset` | `AssetRegistered(addr)`, `AssetRegistry`, `AssetMetadata(addr)`, `AssetMinPrice(addr)` |
/// | Price data | `Price` | `Submission(asset, src)`, `PriceSubmissionLedger(asset, src)`, `Aggregate(asset)`, `PriceOverride(asset)`, `PriceDeviant(asset, src)` |
/// | History | `Hist` | `PriceHistory(asset, ledger)`, `PriceHistoryLedgers(asset)` |
/// | Timelock ops | `Tl` | `TlPendingOpCount`, `TlPendingOp(id)` |
///
/// Soroban encodes each variant name as an XDR `Symbol` discriminant, so variants are
/// inherently collision-free. The namespace prefixes make the category explicit at the
/// call site and prevent accidental re-use of a name across categories in future additions.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DataKey {
    // --- Admin ---
    /// The contract administrator's address.
    Admin,
    ReentrancyGuard,
    /// Existence flag for a registered oracle source (`true` when present).
    Source(Address),
    /// Latest [`PriceEntry`] submitted by a specific source for a specific asset.
    Submission(Address, Address),
    /// Ledger sequence number of the last submission by a source for an asset.
    SubmissionLedger(Address, Address),
    /// Latest [`AggregatePrice`] computed across all contributing sources for an asset.
    Aggregate(Address),
    /// [`PriceHistoryEntry`] recorded at a specific ledger for an asset (temporary storage).
    PriceHistory(Address, u32),
    /// Ordered list of ledger numbers for which history exists for an asset.
    PriceHistoryLedgers(Address),
    /// The [`OracleSources`] registry (list of sources and their metadata).
    OracleSources,
    /// Ordered list of all registered asset addresses.
    RegisteredAssets,
    /// Minimum number of contributing sources required to publish an aggregate price.
    CfgMinSources,
    /// Maximum number of history entries retained per asset before pruning.
    CfgMaxHistory,
    /// Price resolution window in seconds (SEP-40 `resolution` field).
    CfgResolution,
    /// Decimal precision applied to all prices stored by this contract.
    CfgDecimals,
    /// Human-readable description of this oracle instance.
    CfgDescription,
    /// Maximum allowed difference (in seconds) between a submitted timestamp and ledger time.
    CfgTimestampThreshold,
    /// Maximum allowed price deviation in basis points before flagging a submission.
    CfgMaxDeviation,
    /// Interval in seconds after which a source with no heartbeat is considered inactive.
    CfgHeartbeatInterval,
    /// Maximum number of invalid submissions allowed before a source is suspended.
    CfgMaxInvalidSubs,
    /// Currently active [`AggregationMethod`] stored as a `u32` discriminant.
    CfgAggregationMethod,
    /// Boolean flag indicating whether the contract is paused.
    CfgPauseFlag,
    /// Number of ledgers that must pass between proposing and executing a timelock operation.
    CfgTimelockDuration,

    // --- Source registry (prefix: Src) ---
    /// Existence flag for a registered oracle source (`true` when present).
    SrcActive(Address),
    /// The [`OracleSources`] registry (list of sources and their metadata).
    SrcRegistry,
    /// Unix timestamp of the last heartbeat submitted by a source.
    SrcHeartbeat(Address),
    /// Inactive flag for a source.
    SrcInactive(Address),

    // --- Asset registry (prefix: Asset) ---
    /// Existence flag for a registered asset (`true` when present).
    AssetRegistered(Address),
    /// Ordered list of all registered asset addresses (used for enumeration).
    AssetRegistry,

    /// O(1) membership index: `true` when an asset address is registered.
    ///
    /// Kept separate from `AssetRegistry` so we can provide efficient
    /// `is_asset_registered` / `check_registered_asset` lookups while
    /// preserving the historical ordering exposed by `assets()`.
    AssetRegistryIndex(Address),

    /// Optional [`AssetMetadata`] attached to a registered asset.
    AssetMetadata(Address),
    /// Optional minimum accepted price (`i128`) for a registered asset.
    AssetMinPrice(Address),
    /// Configurable maximum number of assets that can be registered.
    MaxAssets,

    /// Boolean flag indicating whether the contract is paused.
    PauseFlag,
    /// Monotonically incrementing counter used to assign IDs to pending operations.
    TlPendingOpCount,
    /// A [`PendingOperation`] awaiting timelock expiry before execution.
    TlPendingOp(u32),
    /// Number of ledgers that must pass between proposing and executing a timelock operation.
    TimelockDuration,
    /// Tracks the number of queries made by a consumer for a specific ledger.
    QueryCount(Address, u32),
    /// Maximum number of queries allowed per ledger for rate limiting.
    QueryRateLimit,
    /// Expiry timestamp for a consumer's subscription.
    SubscriptionExpiry(Address),
    /// Available subscription plans mapped by duration (seconds) to amount (stroops).
    SubscriptionPlans,
    PriceOverride(Address),
    /// Per-asset resolution override in seconds. When set, overrides the contract-wide resolution.
    AssetResolution(Address),
    /// Cooldown (in ledgers) between trigger_aggregation calls per asset.
    AggregationCooldown,
    /// Ledger of the last trigger_aggregation call per asset.
    LastAggregationTrigger(Address),
    /// Minimum submission interval enforcement (in ledgers) for sources.
    MinSubmissionInterval,
    /// Last submission ledger per (source, asset) pair — for compliance tracking.
    LastSubmissionLedger(Address, Address),
    /// Flag marking a source as non-compliant for a given asset.
    SourceNonCompliant(Address, Address),
    /// Counter and storage for pending batch operations.
    PendingBatchCount,
    /// A pending batch operation.
    PendingBatch(u32),
    /// Current storage schema version (u32). Absent means version 1.
    StorageVersion,
    /// Active migration state, if a migration is in progress.
    MigrationState,

    // --- #66: Phased removal (used in sources.rs but missing from original enum) ---
    /// Ledger at which a source becomes eligible for finalization after mark_source_for_removal.
    SourcePendingRemoval(Address),
    /// Decay factor for source reputation scores (u32, out of 100).
    ReputationDecayFactor,
    /// Reputation score for a source (i128, 0–100).
    SourceReputation(Address),
    /// Cooldown in ledgers between mark_source_for_removal and finalize_source_removal.
    RemovalCooldown,
    /// Maximum price deviation key used in admin.rs (mirrors CfgMaxDeviation for set path).
    MaxPriceDeviation,
    /// Timestamp threshold key used in admin.rs set path (mirrors CfgTimestampThreshold).
    TimestampThreshold,
    /// Minimum sources key used in timelock.rs batch execution (mirrors CfgMinSources).
    MinSourcesRequired,
    /// Max history length key used in timelock.rs batch execution (mirrors CfgMaxHistory).
    MaxHistoryLength,
    /// Resolution key used in timelock.rs batch execution (mirrors CfgResolution).
    Resolution,
    /// Decimals key used in timelock.rs batch execution (mirrors CfgDecimals).
    Decimals,

    // -------------------------------------------------------------------------
    // #92/#93/#94: event spam protection, max aggregation sources, per-asset history cap
    // -------------------------------------------------------------------------
    /// Per-asset maximum number of price entries before the oldest is pruned (issue #94).
    MaxHistoryPerAsset,
    /// Maximum number of events that may be emitted in a single call (issue #92).
    MaxEventsPerCall,
    /// Maximum number of sources used for aggregation; excess sources are randomly
    /// sub-sampled using the ledger hash (issue #93).
    MaxAggregationSources,
    /// Maximum number of oracle sources that may be registered. `0` = unlimited.
    MaxSources,
    /// Whether linear interpolation is used for historical price lookups that miss
    /// an exact ledger match.
    InterpolationEnabled,

    // -------------------------------------------------------------------------
    // #186: Adaptive heartbeat / liveness detection
    // -------------------------------------------------------------------------
    /// Number of consecutive missed heartbeats for a source (u32).
    SrcMissedHeartbeats(Address),
    /// Ledger sequence of the last price submission from a source (u32).
    SrcLastPriceLedger(Address),
    /// Flag: source has submitted a price since its last heartbeat reactivation (bool).
    SrcPriceSubmitAfterReactivation(Address),
    /// Ledger sequence at which a source first became inactive — for auto-removal timer (u32).
    SrcInactiveSinceLedger(Address),
    /// Maximum ledgers a source may remain inactive before automatic removal (u32).
    CfgMaxInactiveLedgers,
    /// Window size (number of heartbeat periods) used when computing the adaptive interval (u32).
    CfgHeartbeatWindow,

    // -------------------------------------------------------------------------
    // #187: Commit-reveal MEV resistance
    // -------------------------------------------------------------------------
    /// A pending price commit: stores PriceCommit under (asset, source, round_ledger).
    PriceCommit(Address, Address, u32),
    /// Number of ledgers that the commit phase lasts (sources must commit within this window).
    CfgCommitWindow,
    /// Number of ledgers after the commit deadline during which sources may reveal.
    CfgRevealWindow,

    // -------------------------------------------------------------------------
    // #188: Economic finality gadget
    // -------------------------------------------------------------------------
    /// A pending finality entry for an asset at a given ledger (PendingFinalityEntry).
    PendingFinality(Address, u32),
    /// Number of ledgers to wait before an aggregated price is considered finalized (u32).
    CfgFinalityLedgers,
    /// Recorded ledger hash for reorg detection, keyed by ledger sequence number (BytesN<32>).
    LedgerHashChain(u32),
    /// The finalized aggregate price for an asset (FinalizedPrice).
    FinalizedPrice(Address),

    // -------------------------------------------------------------------------
    // #171: Source Reputation & Slashing
    // -------------------------------------------------------------------------
    /// Staking record for an oracle source (SourceStakeRecord).
    SourceStake(Address),
    /// Address of the XLM/oracle token contract used for staking and fees.
    StakeTokenContract,
    /// Percentage of stake to slash (u32, 0–100) per slash event.
    SlashPercent,
    /// Threshold below which a source's reputation triggers slash eligibility (u32, 0–100).
    SlashReputationThreshold,
    /// Total slashed funds held in contract treasury (i128 stroops).
    TreasuryBalance,

    // -------------------------------------------------------------------------
    // #172: Cross-Asset Correlation
    // -------------------------------------------------------------------------
    /// Min/max ratio band for a (base_asset, quote_asset) pair.
    CorrelationBand(Address, Address),
    /// Ordered list of (base, quote) correlation pairs registered.
    CorrelationPairList,

    // -------------------------------------------------------------------------
    // #173: Tiered Consumer Whitelisting
    // -------------------------------------------------------------------------
    /// Consumer tier and quota info for an address.
    ConsumerInfo(Address),
    /// Pricing for each tier (ConsumerTier discriminant → price in stroops per ledger cycle).
    TierPricing(u32),
    /// Per-ledger query counter for a tiered consumer.
    TierQueryCount(Address, u32),
    /// Treasury address for XLM fee sweeps.
    WhitelistTreasury,
    /// Address of the XLM token contract used for subscription fee collection.
    XlmTokenContract,

    // -------------------------------------------------------------------------
    // #174: Price Deviation Alerts
    // -------------------------------------------------------------------------
    /// Alert subscription record for a (consumer, asset) pair.
    AlertSubscription(Address, Address),
    /// Ordered list of (consumer, asset) pairs that have active alert subscriptions.
    AlertSubscriptionList,
    /// Maximum number of alert subscriptions allowed globally.
    MaxAlertSubscriptions,
    /// TTL in ledgers for alert subscriptions before auto-expiry.
    AlertSubscriptionTtl,
    /// Last aggregate price recorded per asset for deviation comparison.
    AlertLastPrice(Address),

    // -------------------------------------------------------------------------
    // Off-chain relayer network integration
    // -------------------------------------------------------------------------
    /// Metadata for an approved relayer address.
    ApprovedRelayer(Address),
    /// Fee (in stroops) credited to a relayer per successful relayed price submission.
    RelayerFeePerSubmission,
    /// Accumulated fee balance (in stroops) owed to a relayer.
    RelayerFeeBalance(Address),
    /// Running count of successful price submissions made by a relayer.
    RelayerSubmissionCount(Address),

    // -------------------------------------------------------------------------
    // Cross-reference oracle checks
    // -------------------------------------------------------------------------
    /// Stored [`ReferenceOracleEntry`] for a registered external reference oracle.
    ReferenceOracle(Address),
    /// Ordered list of registered reference oracle contract addresses.
    ReferenceOracleList,
    /// Allowed deviation in basis points before a cross-reference alert is emitted.
    CrossRefDeviationThreshold,

    // -------------------------------------------------------------------------
    // #199: Off-Chain Price Deviation Alerting
    // -------------------------------------------------------------------------
    /// Last recorded price for deviation alert (i128).
    AlertLastPrice(Address),

    // -------------------------------------------------------------------------
    // #201: Event Indexing
    // -------------------------------------------------------------------------
    /// Registry of event type discriminants.
    EventTypeRegistry,
    /// Reverse index: address to event IDs.
    EventsByParticipant(Address),
    /// Reverse index: event type to event IDs.
    EventsByType(u32),

    // -------------------------------------------------------------------------
    // #200: Tiered Rate Limiting
    // -------------------------------------------------------------------------
    /// Rate limit tier assignment (stored in ConsumerInfo).
    // (Uses existing ConsumerInfo structure)

    // -------------------------------------------------------------------------
    // #202: Deadline-Aware Submission with Rebate
    // -------------------------------------------------------------------------
    /// Deadline ledger for a source's submission (Address, Address) -> u32.
    SubmissionDeadline(Address, Address),
    /// Rebate amount for a deadline submission (Address, Address) -> i128.
    SubmissionRebate(Address, Address),
    /// Accumulated rebate balance for a source.
    RebateBalance(Address),
}

/// A price submission from a single oracle source for a specific asset.
///
/// Stored under [`DataKey::Submission`] keyed by `(asset, source)`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PriceEntry {
    /// Raw price value scaled by `10^decimals`.
    pub price: i128,
    /// Unix timestamp (seconds) provided by the source at submission time.
    pub timestamp: u64,
    /// Address of the oracle source that submitted this entry.
    pub source: Address,
    /// Decimal precision in effect when this entry was stored.
    pub decimals: u32,
    /// Ledger sequence number when this entry was last written.
    pub last_updated: u32,
    pub ledger_timestamp: u64,
}

/// An aggregated price computed from multiple oracle sources for a specific asset.
///
/// Stored under [`DataKey::Aggregate`] and updated on every [`PriceEntry`] submission
/// that results in enough contributing sources to meet the minimum threshold.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AggregatePrice {
    /// Aggregated price value scaled by `10^decimals`.
    pub price: i128,
    /// Unix timestamp of the most-recent contributing submission.
    pub timestamp: u64,
    /// Number of sources that contributed to this aggregate.
    pub num_sources: u32,
    /// Decimal precision applied to `price`.
    pub decimals: u32,
    pub is_override: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PriceOverrideEntry {
    pub price: i128,
    pub reason: String,
    pub expiry_ledger: u32,
    pub set_ledger: u32,
}

/// Subscription plan configuration.
///
/// Stored under [`DataKey::SubscriptionPlan`] or similar key.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SubscriptionPlan {
    /// Duration of the subscription plan in seconds.
    pub duration: u32,
    /// Cost amount in stroops (i128 for precision).
    pub amount: i128,
}

/// Subscription expiry record for a consumer address.
///
/// Stored under [`DataKey::SubscriptionExpiry`] keyed by consumer address.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SubscriptionExpiry {
    /// Unix timestamp (seconds) at which the subscription expires.
    pub expiry_timestamp: u64,
}

/// A snapshot of the aggregate price recorded at a particular ledger.
///
/// Stored in temporary storage under [`DataKey::PriceHistory`] keyed by `(asset, ledger)`.
/// Entries are pruned to the configured `max_history_length` on each new aggregation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PriceHistoryEntry {
    /// Aggregated price value scaled by the contract's decimal precision.
    pub price: i128,
    /// Unix timestamp of the most-recent contributing submission at snapshot time.
    pub timestamp: u64,
    /// Ledger sequence number when this snapshot was recorded.
    pub ledger: u32,
    /// Number of sources that contributed to this price.
    pub num_sources: u32,
    /// `true` when this entry was produced by linear interpolation rather than a
    /// real submission. Consumers should treat interpolated values as estimates.
    pub is_interpolated: bool,
}

/// Registry of all authorized oracle sources and their display names.
///
/// Stored under [`DataKey::OracleSources`] and updated by [`add_source`] /
/// [`remove_source`] operations.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct OracleSources {
    /// Ordered list of authorized source addresses.
    pub sources: Vec<Address>,
    /// Human-readable display name for each source, keyed by address.
    pub metadata: Map<Address, String>,
}

/// Represents a priced asset, following the SEP-40 oracle interface convention.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum Asset {
    /// A Stellar token identified by its contract address.
    Stellar(Address),
    /// A non-Stellar asset identified by a short symbol (e.g. `"USD"`, `"BTC"`).
    Other(Symbol),
}

/// Strategy used when combining multiple source prices into a single aggregate.
///
/// Stored as a `u32` discriminant under [`DataKey::AggregationMethod`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AggregationMethod {
    /// Select the middle value after sorting; resistant to outliers. (default)
    Median = 0,
    /// Arithmetic mean of all submitted prices.
    Mean = 1,
    /// Arithmetic mean after removing the top and bottom 10 % of values.
    TrimmedMean = 2,
}

/// SEP-40 compatible price data returned by the standard oracle interface methods.
///
/// Used as the return type of [`lastprice`], [`price`], and [`prices`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PriceData {
    /// Aggregated price value scaled by `10^decimals`.
    pub price: i128,
    /// Unix timestamp (seconds) of the price observation.
    pub timestamp: u64,
    /// Ledger sequence number when this data was last updated.
    pub last_updated: u32,
}

/// Discriminant for operations that require timelock protection before execution.
///
/// Used in [`PendingOperation`] and mapped to/from `u32` in the public API.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum OperationType {
    /// Upgrade the contract WASM hash.
    Upgrade = 0,
    /// Replace the administrator address.
    SetAdmin = 1,
    /// Change the minimum number of required sources.
    SetMinSources = 2,
    /// Change the maximum retained history length.
    SetMaxHistory = 3,
    /// Change the price resolution window.
    SetResolution = 4,
    /// Change the decimal precision.
    SetDecimals = 5,
    /// Update the contract description string.
    SetDescription = 6,
    /// Adjust the timestamp validity threshold.
    SetTimestampThreshold = 7,
}

/// A governance operation that has been proposed and is waiting for its timelock to expire.
///
/// Stored under [`DataKey::PendingOp`] keyed by `id`. Removed once executed or cancelled.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PendingOperation {
    /// Unique sequential identifier assigned at proposal time.
    pub id: u32,
    /// Kind of administrative change being proposed.
    pub op_type: OperationType,
    /// Address of the admin who proposed this operation.
    pub proposed_by: Address,
    /// Ledger sequence number when this operation was proposed.
    pub proposed_ledger: u32,
    /// Arbitrary encoded payload whose interpretation depends on `op_type`.
    pub data: Bytes,
}

/// A snapshot of the oracle's overall health, returned by `health_check()`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct HealthReport {
    /// Total number of registered oracle sources.
    pub total_sources: u32,
    /// Number of sources that are currently active (not inactive/suspended).
    pub active_sources: u32,
    /// Total number of registered assets.
    pub total_assets: u32,
    /// Number of assets that have at least one aggregate price recorded.
    pub assets_with_prices: u32,
    /// Whether the contract is currently paused (aggregation suspended).
    pub is_aggregation_paused: bool,
    /// Ledger sequence number of the most recent price aggregation, or 0 if none.
    pub last_aggregation_ledger: u32,
    /// Number of assets whose latest price is stale (older than `resolution` seconds).
    /// Always 0 when `resolution` is 0 (staleness checking disabled).
    pub stale_price_count: u32,
    /// Number of sources currently marked as suspended or inactive.
    pub suspended_source_count: u32,
}

/// Optional metadata that can be attached to a registered asset.
///
/// Stored under [`DataKey::AssetMetadata`] and managed via `set_asset_metadata`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AssetMetadata {
    /// Human-readable name of the asset (e.g. `"Wrapped Bitcoin"`).
    pub name: String,
    /// Trading symbol of the asset (e.g. `"WBTC"`).
    pub symbol: String,
    /// Optional override for the number of decimals used by this asset's token contract.
    /// When `None`, the contract-wide decimal setting applies.
    pub decimals: Option<u32>,
}

/// A single admin operation within a batch, identified by type and its encoded payload.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BatchOperation {
    /// Numeric discriminant matching [`OperationType`] (0–7).
    pub op_type: u32,
    /// Encoded payload for the operation (same encoding as single [`PendingOperation`]).
    pub data: Bytes,
}

/// A pending batch of admin operations waiting for its timelock to expire.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PendingBatch {
    /// Unique sequential identifier assigned at proposal time.
    pub id: u32,
    /// Address of the admin who proposed the batch.
    pub proposed_by: Address,
    /// Ledger when the batch was proposed.
    pub proposed_ledger: u32,
    /// Ordered list of operations to execute atomically.
    pub operations: Vec<BatchOperation>,
}

/// Status of an in-progress storage migration.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum MigrationStatus {
    /// Migration has been started but not yet completed.
    InProgress = 0,
    /// Migration completed successfully.
    Completed = 1,
}

/// Tracks progress of a storage migration from one version to the next.
///
/// Stored under [`DataKey::MigrationState`] while a migration is running.
/// Removed once the migration completes.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct MigrationState {
    /// Storage version being migrated from.
    pub from_version: u32,
    /// Storage version being migrated to.
    pub to_version: u32,
    /// Index of the next item to process (supports pause/resume).
    pub cursor: u32,
    /// Ledger when the migration was started.
    pub started_ledger: u32,
    /// Current migration status.
    pub status: MigrationStatus,
}

// =============================================================================
// #186 — Adaptive Heartbeat / Liveness Detection
// =============================================================================

/// Liveness health status of an oracle source.
///
/// Returned by `get_source_health(source)`. Each variant encodes progressively
/// worse liveness signals.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum SourceHealthStatus {
    /// Source is submitting heartbeats and prices within the adaptive interval.
    Healthy = 0,
    /// Source has missed one or more heartbeats but is still within the allowed window.
    /// The inner `u32` is the consecutive-miss count.
    Degraded = 1,
    /// Source has exceeded the consecutive-miss threshold and is marked inactive.
    Inactive = 2,
    /// Source was automatically removed after exceeding `max_inactive_ledgers`.
    AutoRemoved = 3,
}

// =============================================================================
// #187 — Commit-Reveal MEV Resistance
// =============================================================================

/// A committed (but not yet revealed) price submission.
///
/// Stored in temporary storage under [`DataKey::PriceCommit`] keyed by
/// `(asset, source, round_ledger)`. Expires after `commit_window + reveal_window`
/// ledgers to bound storage growth.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PriceCommit {
    /// sha256(price_bytes || salt || round_ledger_bytes) as a 32-byte hash.
    pub hash: BytesN<32>,
    /// Ledger sequence number when this commit was submitted (= the round's start ledger).
    pub committed_ledger: u32,
    /// Address of the source that made this commit.
    pub source: Address,
    /// Address of the asset this commit is for.
    pub asset: Address,
    /// Whether this commit has been revealed (prevents double-reveal).
    pub revealed: bool,
}

// =============================================================================
// #188 — Economic Finality Gadget
// =============================================================================

/// Finality status of an aggregated price.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum FinalityStatus {
    /// Price is awaiting the finality window. Inner `u32` = ledger it becomes final.
    Pending = 0,
    /// Price has passed the finality window without retraction — immutable.
    Finalized = 1,
    /// Price was retracted by admin before finalization (reorg protection).
    Retracted = 2,
}

/// A price aggregation result that is in the pending-finality window.
///
/// Stored under [`DataKey::PendingFinality`] keyed by `(asset, committed_ledger)`.
/// After `finality_ledgers` pass, it transitions to [`DataKey::FinalizedPrice`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PendingFinalityEntry {
    /// The aggregated price value (scaled by `10^decimals`).
    pub price: i128,
    /// Unix timestamp of the aggregation.
    pub timestamp: u64,
    /// Number of sources that contributed.
    pub num_sources: u32,
    /// Decimal precision applied to `price`.
    pub decimals: u32,
    /// Ledger at which this price was first aggregated.
    pub committed_ledger: u32,
    /// Ledger after which this price is considered finalized (= committed_ledger + finality_ledgers).
    pub finality_ledger: u32,
    /// Current status.
    pub status: FinalityStatus,
    /// Ledger hash recorded at `committed_ledger` — used for reorg detection.
    pub ledger_hash: BytesN<32>,
}

/// An immutable, finalized price record.
///
/// Written to [`DataKey::FinalizedPrice`] when a [`PendingFinalityEntry`] passes
/// `finality_ledger` without retraction.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct FinalizedPrice {
    /// The finalized aggregated price.
    pub price: i128,
    /// Unix timestamp of the finalized aggregation.
    pub timestamp: u64,
    /// Number of contributing sources.
    pub num_sources: u32,
    /// Decimal precision.
    pub decimals: u32,
    /// Ledger at which this price was originally aggregated.
    pub committed_ledger: u32,
    /// Ledger at which finality was confirmed.
    pub finalized_ledger: u32,
}

// =============================================================================
// #171 — Source Reputation & Slashing
// =============================================================================

/// Staking record for an oracle source.
///
/// Stored under [`DataKey::SourceStake`] keyed by source address.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SourceStakeRecord {
    /// Locked stake amount in stroops.
    pub amount: i128,
    /// Ledger at which the stake was last updated.
    pub last_updated_ledger: u32,
}

// =============================================================================
// #172 — Cross-Asset Correlation
// =============================================================================

/// Acceptable ratio band between two correlated assets.
///
/// The ratio is computed as `price_base * RATIO_PRECISION / price_quote` and must
/// fall within `[min_ratio, max_ratio]`. All values are scaled by `RATIO_PRECISION`
/// (10^7) to avoid floating-point arithmetic.
///
/// Stored under [`DataKey::CorrelationBand`] keyed by `(base_asset, quote_asset)`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CorrelationBand {
    /// Minimum acceptable ratio (scaled by 10^7).
    pub min_ratio: u128,
    /// Maximum acceptable ratio (scaled by 10^7).
    pub max_ratio: u128,
    /// Whether this correlation check is currently active.
    pub enabled: bool,
}

/// A registered correlation pair for enumeration purposes.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CorrelationPair {
    pub base_asset: Address,
    pub quote_asset: Address,
}

// =============================================================================
// #173 — Tiered Consumer Access
// =============================================================================

/// Consumer access tier.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ConsumerTier {
    /// Free tier: 10 queries/ledger, data up to 1-hour stale.
    Free = 0,
    /// Basic tier: 100 queries/ledger, max 30-second fresh data.
    Basic = 1,
    /// Premium tier: unlimited queries/ledger, real-time data.
    Premium = 2,
}

/// Per-consumer tier, quota, and subscription state.
///
/// Stored under [`DataKey::ConsumerInfo`] keyed by consumer address.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ConsumerInfo {
    /// Current access tier.
    pub tier: ConsumerTier,
    /// Ledger-based subscription expiration. 0 = no active subscription.
    pub subscription_expiry_ledger: u32,
    /// Unix timestamp-based subscription expiration. 0 = no active subscription.
    pub subscription_expiry_ts: u64,
    /// Number of queries consumed in the current ledger.
    pub queries_this_ledger: u32,
    /// Ledger sequence for which `queries_this_ledger` was last reset.
    pub quota_reset_ledger: u32,
}

// =============================================================================
// #174 — On-Chain Alert Subscriptions
// =============================================================================

/// An alert subscription record.
///
/// Stored under [`DataKey::AlertSubscription`] keyed by `(consumer, asset)`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AlertSubscription {
    /// Address of the subscriber (consumer contract or account).
    pub consumer: Address,
    /// Asset being monitored.
    pub asset: Address,
    /// Price movement threshold in basis points (100 bps = 1%) that triggers an alert.
    pub threshold_bps: u32,
    /// Contract address to invoke when the threshold is breached.
    pub callback_contract: Address,
    /// Function selector (Symbol) on the callback contract to call.
    pub callback_fn: Symbol,
    /// Ledger at which this subscription was created or last renewed.
    pub created_ledger: u32,
    /// TTL in ledgers; subscription expires at `created_ledger + ttl`.
    pub ttl_ledgers: u32,
}

// =============================================================================
// Off-chain relayer network integration
// =============================================================================

/// Metadata stored for each admin-approved relayer.
///
/// Stored under [`DataKey::ApprovedRelayer`] keyed by relayer address.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RelayerInfo {
    /// Human-readable display name for this relayer.
    pub name: String,
    /// Ledger sequence number when the relayer was approved by the admin.
    pub approved_at_ledger: u32,
}

// =============================================================================
// Cross-reference oracle checks
// =============================================================================

/// A registered external oracle contract used for cross-reference price checks.
///
/// Stored under [`DataKey::ReferenceOracle`] keyed by the oracle's contract address.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ReferenceOracleEntry {
    /// Contract address of the external oracle.
    pub contract_id: Address,
    /// Maps our asset addresses to the corresponding asset addresses used by the reference oracle.
    pub asset_mapping: Map<Address, Address>,
}

/// The result of a cross-reference price comparison for a single asset.
///
/// Returned by `get_cross_reference`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CrossReferenceResult {
    /// Our current aggregated price for the asset.
    pub our_price: i128,
    /// Price reported by the reference oracle for the same asset.
    pub ref_price: i128,
    /// Absolute deviation between the two prices expressed in basis points (1 % = 100 bps).
    pub deviation_bps: u32,
    /// Contract address of the reference oracle that provided `ref_price`.
    pub ref_contract: Address,
}
