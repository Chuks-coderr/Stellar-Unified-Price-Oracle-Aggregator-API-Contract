use soroban_sdk::{contractevent, Address, String, Symbol};

#[contractevent]
#[derive(Clone)]
pub struct PriceSubmittedEvent {
    #[topic]
    pub asset: Address,
    #[topic]
    pub source: Address,
    pub price: i128,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct PriceAggregatedEvent {
    #[topic]
    pub asset: Address,
    pub price: i128,
    pub num_sources: u32,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct SourceAddedEvent {
    #[topic]
    pub source: Address,
    pub name: String,
}

#[contractevent]
#[derive(Clone)]
pub struct SourceRemovedEvent {
    #[topic]
    pub source: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct AssetRegisteredEvent {
    #[topic]
    pub asset: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct AssetUnregisteredEvent {
    #[topic]
    pub asset: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct AdminChangedEvent {
    #[topic]
    pub new_admin: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct MinSourcesChangedEvent {
    pub value: u32,
}

#[contractevent]
#[derive(Clone)]
pub struct MaxHistoryChangedEvent {
    pub value: u32,
}

#[contractevent]
#[derive(Clone)]
pub struct ResolutionChangedEvent {
    pub value: u32,
}

#[contractevent]
#[derive(Clone)]
pub struct DecimalsChangedEvent {
    pub value: u32,
}

#[contractevent]
#[derive(Clone)]
pub struct DescriptionChangedEvent {
    pub description: String,
}

#[contractevent]
#[derive(Clone)]
pub struct ContractUpgradedEvent {
    pub new_wasm_hash: soroban_sdk::BytesN<32>,
}

// ---- Operation expiry events ----

/// Emitted when a new pending operation is enqueued.
#[contractevent]
#[derive(Clone)]
pub struct OperationQueuedEvent {
    #[topic]
    pub operation_id: u64,
    pub expires_at_ledger: u32,
}

/// Emitted when a pending operation is successfully executed.
#[contractevent]
#[derive(Clone)]
pub struct OperationExecutedEvent {
    #[topic]
    pub operation_id: u64,
}

/// Emitted when a pending operation is expired (either on-demand or via maintenance sweep).
#[contractevent]
#[derive(Clone)]
pub struct OperationExpiredEvent {
    #[topic]
    pub operation_id: u64,
    pub expired_at_ledger: u32,
}

/// Emitted when the default operation expiry window is changed.
#[contractevent]
#[derive(Clone)]
pub struct ExpiryWindowChangedEvent {
    pub ledgers: u32,
}

// ---- Template lifecycle events ----

/// Emitted when a new template is created.
#[contractevent]
#[derive(Clone)]
pub struct TemplateCreatedEvent {
    #[topic]
    pub name: Symbol,
    pub num_steps: u32,
}

/// Emitted when a template is applied (instantiated into pending operations).
#[contractevent]
#[derive(Clone)]
pub struct TemplateAppliedEvent {
    #[topic]
    pub name: Symbol,
    /// Number of pending operations created from this template application.
    pub operations_created: u32,
}

/// Emitted when a template is removed.
#[contractevent]
#[derive(Clone)]
pub struct TemplateRemovedEvent {
    #[topic]
    pub name: Symbol,
}
