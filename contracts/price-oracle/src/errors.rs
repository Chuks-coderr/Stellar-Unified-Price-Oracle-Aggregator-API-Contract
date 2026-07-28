use soroban_sdk::contracterror;

/// Error codes returned by contract invocations when a precondition is violated.
///
/// Each variant maps to a `u32` discriminant that is embedded in the Soroban
/// host error returned to the caller. Clients should match on these values to
/// present meaningful error messages.
///
/// **Discriminant registry** (never reuse a number, even after a variant is removed):
///
/// | Range  | Category               |
/// |--------|------------------------|
/// | 0–15   | Core / original        |
/// | 16–19  | Rate-limit & migration |
/// | 20–29  | Source management      |
/// | 30–39  | Commit-reveal (#187)   |
/// | 40–49  | Finality gadget (#188) |
/// | 50–59  | Relayer & misc         |
#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    // ── 0–15: Core ───────────────────────────────────────────────────────────
    /// The caller is not authorized to perform the requested operation.
    NotAuthorized = 0,
    /// `initialize` was called on a contract that has already been set up.
    AlreadyInitialized = 1,
    /// The specified asset has not been registered with `register_asset`.
    AssetNotRegistered = 2,
    /// `register_asset` was called for an asset that is already registered.
    AssetAlreadyRegistered = 3,
    /// `add_source` was called for an address that is already a registered source.
    SourceAlreadyExists = 4,
    /// The referenced oracle source address is not registered.
    SourceNotFound = 5,
    /// Fewer sources have submitted prices than the configured `min_sources_required`.
    InsufficientSources = 6,
    /// A submitted price value is zero or negative.
    InvalidPrice = 7,
    /// No aggregate price data exists for the requested asset.
    NoData = 8,
    /// The submitted timestamp lies too far in the future relative to the current ledger time.
    InvalidTimestamp = 9,
    /// A configuration parameter is out of its valid range.
    InvalidConfiguration = 10,
    /// The description string exceeds the maximum allowed length of 256 characters.
    DescriptionTooLong = 11,
    /// The contract is currently paused; no price submissions or reads are allowed.
    ContractPaused = 12,
    /// A timelock operation cannot yet be executed because its delay period has not elapsed.
    TimelockNotReady = 13,
    /// No pending operation exists with the given ID.
    OperationNotFound = 14,
    /// The submitted price is below the asset's configured minimum price floor.
    PriceBelowMinimum = 15,
    /// The submitted price falls outside the asset's configured price bounds.
    PriceOutOfBounds = 54,
    /// The asset is currently paused and cannot accept new submissions.
    AssetPaused = 55,
    /// The circuit breaker tripped for the asset and rejected the update.
    CircuitBreakerTripped = 56,

    // ── 16–19: Rate-limit, subscription & migration ──────────────────────────
    /// Rate limit exceeded for an operation.
    RateLimitExceeded = 16,
    /// The requested subscription plan duration does not exist.
    InvalidDuration = 17,
    /// The consumer's subscription has expired.
    SubscriptionExpired = 18,
    /// A migration is already in progress.
    MigrationInProgress = 19,

    // ── 20–29: Source management ─────────────────────────────────────────────
    /// No migration is currently in progress.
    NoMigrationInProgress = 20,
    /// The source name is empty.
    SourceNameEmpty = 21,
    /// The source name exceeds the maximum allowed length.
    SourceNameTooLong = 22,
    /// The maximum number of sources has been reached.
    MaxSourcesReached = 23,
    /// The `op_type` discriminant passed to `propose_operation` is not valid.
    InvalidOperationType = 24,
    /// A reentrancy attempt was detected.
    Reentrant = 25,
    /// Source is not pending removal (cancel / finalize called on non-pending source).
    SourceNotPendingRemoval = 26,
    /// The removal cooldown has not elapsed yet.
    CooldownNotElapsed = 27,
    /// Maximum number of registered assets reached.
    MaxAssetsReached = 28,
    /// A reason string is too long.
    ReasonTooLong = 29,
    /// Records limit exceeded for price history query.
    RecordsLimitExceeded = 30,

    // ── 31–39: Commit-reveal (#187) ──────────────────────────────────────────
    /// The revealed hash does not match the committed hash.
    CommitHashMismatch = 31,
    /// The commit has expired (reveal window has closed for this round).
    CommitExpired = 32,
    /// No commit was found for this (source, asset, round).
    CommitNotFound = 33,
    /// The reveal window is closed — too early or too late to reveal.
    RevealWindowClosed = 34,
    /// A commit already exists for this (source, asset, round); cannot double-commit.
    AlreadyCommitted = 35,
    /// The commit round ledger is invalid (e.g., in the future).
    InvalidCommitRound = 36,

    // ── 40–49: Finality gadget (#188) ────────────────────────────────────────
    /// The price has already been finalized and cannot be changed.
    AlreadyFinalized = 40,
    /// The price was retracted due to a detected reorg.
    PriceRetracted = 41,
    /// The price is still in the finality pending window.
    FinalityPending = 42,
    /// The requested price does not meet the caller's minimum finality requirement.
    InsufficientFinality = 43,
    /// A reorg was detected via ledger hash chain inconsistency.
    ReorgDetected = 44,

    // ── 50–59: Relayer & misc ────────────────────────────────────────────────
    /// The caller is not a registered and approved relayer.
    RelayerNotAuthorized = 50,
    /// `add_relayer` was called for an address that is already an approved relayer.
    RelayerAlreadyExists = 51,
    /// Source reputation is too high to be eligible for slashing.
    ReputationTooHighToSlash = 52,
    /// Maximum number of alert subscriptions has been reached.
    MaxSubscriptionsReached = 53,
    /// Direct price submissions are rejected while BFT aggregation is enabled because
    /// commit-reveal is required for source consensus.
    CommitRevealRequired = 57,
}
