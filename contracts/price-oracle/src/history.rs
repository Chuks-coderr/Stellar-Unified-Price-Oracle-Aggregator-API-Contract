use soroban_sdk::{panic_with_error, Address, Env, Vec};

use crate::admin::{get_interpolation_enabled, get_max_history_length};
use crate::storage::{check_registered_asset, LEDGER_BUMP, LEDGER_THRESHOLD};
use crate::types::{DataKey, ErrorCode, PriceHistoryEntry};

pub fn get_historical_price(env: &Env, asset: Address, ledger: u32) -> PriceHistoryEntry {
    check_registered_asset(env, &asset);

    // Exact match — return as-is.
    let key = DataKey::PriceHistory(asset.clone(), ledger);
    if env.storage().temporary().has(&key) {
        env.storage()
            .temporary()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
        return env.storage().temporary().get(&key).unwrap();
    }

    // If interpolation is disabled, panic.
    if !get_interpolation_enabled(env) {
        panic_with_error!(env, ErrorCode::NoData);
    }

    // Find the nearest before/after entries via the ledger index.
    let ledgers_key = DataKey::PriceHistoryLedgers(asset.clone());
    let ledger_list: Vec<u32> = env
        .storage()
        .persistent()
        .get(&ledgers_key)
        .unwrap_or(Vec::new(env));

    let mut before: Option<PriceHistoryEntry> = None;
    let mut after: Option<PriceHistoryEntry> = None;

    for i in 0..ledger_list.len() {
        let l = ledger_list.get_unchecked(i);
        if l <= ledger {
            let k = DataKey::PriceHistory(asset.clone(), l);
            if env.storage().temporary().has(&k) {
                let entry: PriceHistoryEntry = env.storage().temporary().get(&k).unwrap();
                before = Some(entry);
            }
        } else if after.is_none() {
            let k = DataKey::PriceHistory(asset.clone(), l);
            if env.storage().temporary().has(&k) {
                let entry: PriceHistoryEntry = env.storage().temporary().get(&k).unwrap();
                after = Some(entry);
            }
        }
    }

    match (before, after) {
        (Some(b), Some(a)) => {
            // Linear interpolation: price = b.price + (a.price - b.price) * (ledger - b.ledger) / (a.ledger - b.ledger)
            let range = (a.ledger - b.ledger) as i128;
            let offset = (ledger - b.ledger) as i128;
            let interpolated_price = b.price + (a.price - b.price) * offset / range;
            let interpolated_ts = b.timestamp
                + ((a.timestamp.saturating_sub(b.timestamp) as i128) * offset / range) as u64;
            PriceHistoryEntry {
                price: interpolated_price,
                timestamp: interpolated_ts,
                ledger,
                num_sources: 0,
                is_interpolated: true,
            }
        }
        _ => panic_with_error!(env, ErrorCode::NoData),
    }
}

pub fn has_historical_price(env: &Env, asset: Address, ledger: u32) -> bool {
    if !env
        .storage()
        .persistent()
        .has(&DataKey::AssetRegistered(asset.clone()))
    {
        return false;
    }
    let key = DataKey::PriceHistory(asset, ledger);
    let exists = env.storage().temporary().has(&key);
    if exists {
        env.storage()
            .temporary()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    }
    exists
}

pub fn get_historical_prices(
    env: &Env,
    asset: Address,
    start_ledger: u32,
    end_ledger: u32,
) -> Vec<PriceHistoryEntry> {
    check_registered_asset(env, &asset);
    let max_range = get_max_history_length(env);
    if end_ledger < start_ledger || end_ledger - start_ledger > max_range {
        panic_with_error!(env, ErrorCode::NoData);
    }
    let mut entries: Vec<PriceHistoryEntry> = Vec::new(env);
    let mut ledger = start_ledger;
    while ledger <= end_ledger {
        let key = DataKey::PriceHistory(asset.clone(), ledger);
        if env.storage().temporary().has(&key) {
            let entry: PriceHistoryEntry = env.storage().temporary().get(&key).unwrap();
            env.storage()
                .temporary()
                .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
            entries.push_back(entry);
        }
        ledger += 1;
    }
    entries
}

/// Maximum page size accepted by `get_historical_prices_paginated` (#229).
pub const MAX_PAGE_SIZE: u32 = 50;

/// Returns a cursor-paginated page of historical price entries for an asset.
///
/// `cursor` is the ledger sequence number to start from, inclusive (pass `0` to
/// start at the beginning). `limit` must be in `1..=MAX_PAGE_SIZE`. Entries are
/// returned in ascending ledger order, correctly skipping over gaps where no
/// snapshot was recorded.
///
/// Returns `(entries, next_cursor)`: `next_cursor` is `Some(ledger)` to pass as
/// the next page's `cursor`, or `None` once every recorded entry has been read.
///
/// # Errors
/// * [`ErrorCode::AssetNotRegistered`] — if `asset` is not registered.
/// * [`ErrorCode::InvalidPageSize`] — if `limit` is `0` or exceeds `MAX_PAGE_SIZE`.
pub fn get_historical_prices_paginated(
    env: &Env,
    asset: Address,
    cursor: u32,
    limit: u32,
) -> (Vec<PriceHistoryEntry>, Option<u32>) {
    check_registered_asset(env, &asset);
    if limit == 0 || limit > MAX_PAGE_SIZE {
        panic_with_error!(env, ErrorCode::InvalidPageSize);
    }

    let ledgers_key = DataKey::PriceHistoryLedgers(asset.clone());
    let ledger_list: Vec<u32> = env
        .storage()
        .persistent()
        .get(&ledgers_key)
        .unwrap_or(Vec::new(env));

    let mut entries: Vec<PriceHistoryEntry> = Vec::new(env);
    let mut next_cursor: Option<u32> = None;

    for i in 0..ledger_list.len() {
        let l = ledger_list.get_unchecked(i);
        if l < cursor {
            continue;
        }
        if entries.len() >= limit {
            next_cursor = Some(l);
            break;
        }
        let key = DataKey::PriceHistory(asset.clone(), l);
        if env.storage().temporary().has(&key) {
            let entry: PriceHistoryEntry = env.storage().temporary().get(&key).unwrap();
            env.storage()
                .temporary()
                .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
            entries.push_back(entry);
        }
    }

    (entries, next_cursor)
}
