use soroban_sdk::{contracterror, contracttype, Address, Map, String, Symbol, Vec};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DataKey {
    Admin,
    Source(Address),
    AssetRegistered(Address),
    Submission(Address, Address),
    Aggregate(Address),
    PriceHistory(Address, u32),
    OracleSources,
    RegisteredAssets,
    MinSourcesRequired,
    MaxHistoryLength,
    Resolution,
    Decimals,
    Description,
    // Operation expiry
    OperationExpiry,
    PendingOperation(u64),
    PendingOperationIds,
    // Template registry
    Template(Symbol),
    TemplateNames,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PriceEntry {
    pub price: i128,
    pub timestamp: u64,
    pub source: Address,
    pub decimals: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AggregatePrice {
    pub price: i128,
    pub timestamp: u64,
    pub num_sources: u32,
    pub decimals: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PriceHistoryEntry {
    pub price: i128,
    pub timestamp: u64,
    pub ledger: u32,
    pub num_sources: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct OracleSources {
    pub sources: Vec<Address>,
    pub metadata: Map<Address, String>,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    NotAuthorized = 0,
    AlreadyInitialized = 1,
    AssetNotRegistered = 2,
    AssetAlreadyRegistered = 3,
    SourceAlreadyExists = 4,
    SourceNotFound = 5,
    InsufficientSources = 6,
    InvalidPrice = 7,
    NoData = 8,
    // Expiry errors
    OperationExpired = 9,
    OperationNotFound = 10,
    // Template errors
    TemplateNotFound = 11,
    TemplateAlreadyExists = 12,
    InvalidTemplate = 13,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum Asset {
    Stellar(Address),
    Other(Symbol),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}

// ---- Pending operation types ----

/// The kind of administrative action captured in a pending operation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum OperationKind {
    AddSource,
    RemoveSource,
    RegisterAsset,
    UnregisterAsset,
    SetMinSources,
    SetMaxHistory,
    SetDecimals,
    SetDescription,
}

/// A pending operation waiting to be executed or expired.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PendingOperation {
    /// Unique monotonic id (ledger sequence at creation).
    pub id: u64,
    pub kind: OperationKind,
    /// JSON-style serialized args stored as a String for simplicity.
    pub args: String,
    /// Ledger sequence at which this operation was created.
    pub created_at_ledger: u32,
    /// Ledger sequence after which this operation is expired and unexecutable.
    pub expires_at_ledger: u32,
    /// Whether this operation has been executed already.
    pub executed: bool,
}

// ---- Template registry types ----

/// A single parameterized step inside a template.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct TemplateStep {
    pub kind: OperationKind,
    /// Human-readable description of this step.
    pub description: String,
}

/// A named, reusable sequence of operation steps.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct OperationTemplate {
    pub name: Symbol,
    pub description: String,
    pub steps: Vec<TemplateStep>,
    pub created_at_ledger: u32,
}
