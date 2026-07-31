use crate::types::{DataKey, ErrorCode, OperationTemplate, OracleSources, PendingOperation};
use soroban_sdk::{panic_with_error, Address, Env, Symbol, Vec};

pub const LEDGER_THRESHOLD: u32 = 1000;
pub const LEDGER_BUMP: u32 = 4000;

/// Default number of ledgers after creation before a pending operation expires (~24 h at 5 s/ledger).
pub const DEFAULT_EXPIRY_LEDGERS: u32 = 17_280;

pub fn get_admin(env: &Env) -> Address {
    env.storage().persistent().get(&DataKey::Admin).unwrap()
}

pub fn check_source(env: &Env, addr: &Address) {
    let key = DataKey::Source(addr.clone());
    let is_source: bool = env.storage().persistent().get(&key).unwrap_or(false);
    if !is_source {
        panic_with_error!(env, ErrorCode::NotAuthorized);
    }
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn check_registered_asset(env: &Env, asset: &Address) {
    let key = DataKey::AssetRegistered(asset.clone());
    let is_registered: bool = env.storage().persistent().get(&key).unwrap_or(false);
    if !is_registered {
        panic_with_error!(env, ErrorCode::AssetNotRegistered);
    }
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn sort_prices(prices: &mut soroban_sdk::Vec<i128>) {
    let n = prices.len();
    if n <= 1 {
        return;
    }
    quicksort(prices, 0, n - 1);
}

fn quicksort(prices: &mut soroban_sdk::Vec<i128>, low: u32, high: u32) {
    if low < high {
        let pi = partition(prices, low, high);
        if pi > 0 {
            quicksort(prices, low, pi - 1);
        }
        quicksort(prices, pi + 1, high);
    }
}

fn partition(prices: &mut soroban_sdk::Vec<i128>, low: u32, high: u32) -> u32 {
    let pivot = prices.get_unchecked(high);
    let mut i = low;
    let mut j = low;
    while j < high {
        if prices.get_unchecked(j) <= pivot {
            let tmp = prices.get_unchecked(i);
            prices.set(i, prices.get_unchecked(j));
            prices.set(j, tmp);
            i += 1;
        }
        j += 1;
    }
    let tmp = prices.get_unchecked(i);
    prices.set(i, prices.get_unchecked(high));
    prices.set(high, tmp);
    i
}

pub fn compute_median(prices: &soroban_sdk::Vec<i128>) -> i128 {
    let n = prices.len();
    if n == 0 {
        return 0;
    }
    let mut sorted = prices.clone();
    sort_prices(&mut sorted);
    if n.is_multiple_of(2) {
        let mid = n / 2;
        let a = sorted.get_unchecked(mid - 1);
        let b = sorted.get_unchecked(mid);
        a + (b - a) / 2
    } else {
        sorted.get_unchecked(n / 2)
    }
}

pub fn read_registered_assets(env: &Env) -> Vec<Address> {
    let key = DataKey::RegisteredAssets;
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    }
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(env))
}

pub fn write_registered_assets(env: &Env, assets: &Vec<Address>) {
    env.storage()
        .persistent()
        .set(&DataKey::RegisteredAssets, assets);
}

pub fn read_oracle_sources(env: &Env) -> OracleSources {
    let key = DataKey::OracleSources;
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    }
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(OracleSources {
            sources: soroban_sdk::Vec::new(env),
            metadata: soroban_sdk::Map::new(env),
        })
}

// ---- Expiry window helpers ----

/// Read the configured expiry window in ledgers (defaults to DEFAULT_EXPIRY_LEDGERS).
pub fn read_expiry_ledgers(env: &Env) -> u32 {
    let key = DataKey::OperationExpiry;
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
        env.storage().persistent().get(&key).unwrap()
    } else {
        DEFAULT_EXPIRY_LEDGERS
    }
}

pub fn write_expiry_ledgers(env: &Env, ledgers: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::OperationExpiry, &ledgers);
}

// ---- Pending operation helpers ----

pub fn read_pending_ids(env: &Env) -> Vec<u64> {
    let key = DataKey::PendingOperationIds;
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
        env.storage().persistent().get(&key).unwrap()
    } else {
        Vec::new(env)
    }
}

pub fn write_pending_ids(env: &Env, ids: &Vec<u64>) {
    env.storage()
        .persistent()
        .set(&DataKey::PendingOperationIds, ids);
}

pub fn read_pending_operation(env: &Env, id: u64) -> Option<PendingOperation> {
    let key = DataKey::PendingOperation(id);
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
        env.storage().persistent().get(&key)
    } else {
        None
    }
}

pub fn write_pending_operation(env: &Env, op: &PendingOperation) {
    let key = DataKey::PendingOperation(op.id);
    env.storage().persistent().set(&key, op);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn remove_pending_operation(env: &Env, id: u64) {
    let key = DataKey::PendingOperation(id);
    if env.storage().persistent().has(&key) {
        env.storage().persistent().remove(&key);
    }
    let ids = read_pending_ids(env);
    let mut new_ids: Vec<u64> = Vec::new(env);
    for i in 0..ids.len() {
        let existing = ids.get_unchecked(i);
        if existing != id {
            new_ids.push_back(existing);
        }
    }
    write_pending_ids(env, &new_ids);
}

// ---- Template registry helpers ----

pub fn read_template_names(env: &Env) -> Vec<Symbol> {
    let key = DataKey::TemplateNames;
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
        env.storage().persistent().get(&key).unwrap()
    } else {
        Vec::new(env)
    }
}

pub fn write_template_names(env: &Env, names: &Vec<Symbol>) {
    env.storage()
        .persistent()
        .set(&DataKey::TemplateNames, names);
}

pub fn read_template(env: &Env, name: &Symbol) -> Option<OperationTemplate> {
    let key = DataKey::Template(name.clone());
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
        env.storage().persistent().get(&key)
    } else {
        None
    }
}

pub fn write_template(env: &Env, template: &OperationTemplate) {
    let key = DataKey::Template(template.name.clone());
    env.storage().persistent().set(&key, template);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn remove_template(env: &Env, name: &Symbol) {
    let key = DataKey::Template(name.clone());
    if env.storage().persistent().has(&key) {
        env.storage().persistent().remove(&key);
    }
    let names = read_template_names(env);
    let mut new_names: Vec<Symbol> = Vec::new(env);
    for i in 0..names.len() {
        let n = names.get_unchecked(i);
        if n != *name {
            new_names.push_back(n);
        }
    }
    write_template_names(env, &new_names);
}
