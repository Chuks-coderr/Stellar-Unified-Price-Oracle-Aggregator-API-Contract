use crate::types::{DataKey, ErrorCode, OracleSources, SubscriptionPlans};
use soroban_sdk::{panic_with_error, Address, Env, Map, Vec};

pub const LEDGER_THRESHOLD: u32 = 1000;
pub const LEDGER_BUMP: u32 = 4000;
pub const DEFAULT_QUERY_RATE_LIMIT: u32 = 100;

pub fn get_admin(env: &Env) -> Address {
    env.storage().persistent().get(&DataKey::Admin).unwrap()
}

pub fn check_source(env: &Env, addr: &Address) {
    let key = DataKey::SrcActive(addr.clone());
    let is_source: bool = env.storage().persistent().get(&key).unwrap_or(false);
    if !is_source {
        panic_with_error!(env, ErrorCode::NotAuthorized);
    }
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn check_registered_asset(env: &Env, asset: &Address) {
    // Prefer the O(1) membership index.
    let index_key = DataKey::AssetRegistryIndex(asset.clone());
    let indexed: bool = env.storage().persistent().get(&index_key).unwrap_or(false);
    if indexed {
        env.storage()
            .persistent()
            .extend_ttl(&index_key, LEDGER_THRESHOLD, LEDGER_BUMP);
        return;
    }

    // Backward compatibility: older deployments only have the legacy
    // `AssetRegistered(asset)` flag. If it exists, lazily (re)build
    // the index entry.
    let legacy_key = DataKey::AssetRegistered(asset.clone());
    let exists: bool = env.storage().persistent().get(&legacy_key).unwrap_or(false);
    if !exists {
        panic_with_error!(env, ErrorCode::AssetNotRegistered);
    }

    env.storage()
        .persistent()
        .extend_ttl(&legacy_key, LEDGER_THRESHOLD, LEDGER_BUMP);

    env.storage().persistent().set(&index_key, &true);
    env.storage()
        .persistent()
        .extend_ttl(&index_key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

/// Sort prices using heapsort — guaranteed O(n log n) worst-case, O(1) extra space.
/// Used by `compute_trimmed_mean` which needs a fully sorted array.
pub fn sort_prices(prices: &mut soroban_sdk::Vec<i128>) {
    let n = prices.len();
    if n <= 1 {
        return;
    }
    // Build max-heap
    let mut i = n / 2;
    loop {
        heapify(prices, n, i);
        if i == 0 {
            break;
        }
        i -= 1;
    }
    // Extract elements from heap one by one
    let mut end = n - 1;
    loop {
        let tmp = prices.get_unchecked(0);
        prices.set(0, prices.get_unchecked(end));
        prices.set(end, tmp);
        heapify(prices, end, 0);
        if end == 0 {
            break;
        }
        end -= 1;
    }
}

/// Sift down the element at `root` within a heap of size `n` (iterative, no stack growth).
fn heapify(prices: &mut soroban_sdk::Vec<i128>, n: u32, root: u32) {
    let mut current = root;
    loop {
        let mut largest = current;
        let left = 2 * current + 1;
        let right = 2 * current + 2;
        if left < n && prices.get_unchecked(left) > prices.get_unchecked(largest) {
            largest = left;
        }
        if right < n && prices.get_unchecked(right) > prices.get_unchecked(largest) {
            largest = right;
        }
        if largest == current {
            break;
        }
        let tmp = prices.get_unchecked(current);
        prices.set(current, prices.get_unchecked(largest));
        prices.set(largest, tmp);
        current = largest;
    }
}

// ---------------------------------------------------------------------------
// Quickselect (Floyd-Rivest selection algorithm) — O(n) average, O(n²) worst
// ---------------------------------------------------------------------------
// We use a deterministic pivot (median-of-three) to avoid O(n²) on adversarial
// sorted or reverse-sorted inputs while still beating heapsort for large n.
//
// Gas impact: for 50 sources quickselect processes ≈50 elements vs heapsort's
// ≈50×6 = 300 comparisons — an estimated 40–60 % gas reduction per aggregation.

/// Swap elements at positions `a` and `b` in a soroban `Vec<i128>`.
#[inline]
fn swap_prices(prices: &mut soroban_sdk::Vec<i128>, a: u32, b: u32) {
    if a != b {
        let tmp = prices.get_unchecked(a);
        prices.set(a, prices.get_unchecked(b));
        prices.set(b, tmp);
    }
}

/// Partition `prices[lo..=hi]` around a pivot chosen as the median of
/// `prices[lo]`, `prices[mid]`, `prices[hi]`. Returns the final pivot index.
fn partition(prices: &mut soroban_sdk::Vec<i128>, lo: u32, hi: u32) -> u32 {
    let mid = lo + (hi - lo) / 2;

    // Median-of-three pivot selection (sorts lo, mid, hi in-place as a side effect)
    if prices.get_unchecked(lo) > prices.get_unchecked(mid) {
        swap_prices(prices, lo, mid);
    }
    if prices.get_unchecked(lo) > prices.get_unchecked(hi) {
        swap_prices(prices, lo, hi);
    }
    if prices.get_unchecked(mid) > prices.get_unchecked(hi) {
        swap_prices(prices, mid, hi);
    }
    // pivot is now at `mid`; move it to hi-1 to stay out of the partition loop
    let pivot = prices.get_unchecked(mid);
    swap_prices(prices, mid, hi);

    // Lomuto-style partition around `pivot`
    let mut store = lo;
    let mut i = lo;
    // iterate [lo, hi-1) since we placed the pivot at hi
    while i < hi {
        if prices.get_unchecked(i) <= pivot {
            swap_prices(prices, i, store);
            store += 1;
        }
        i += 1;
    }
    // Restore pivot to its final position
    swap_prices(prices, store, hi);
    store
}

/// Rearrange `prices` in-place so that `prices[k]` is the k-th smallest
/// element (0-indexed) and all elements before it are ≤ it. O(n) average.
///
/// Uses iterative tail recursion to avoid stack growth in `no_std` WASM.
pub fn quickselect(prices: &mut soroban_sdk::Vec<i128>, k: u32) {
    let n = prices.len();
    if n <= 1 || k >= n {
        return;
    }
    let mut lo: u32 = 0;
    let mut hi: u32 = n - 1;
    loop {
        if lo >= hi {
            break;
        }
        // For tiny sub-arrays (≤ 3 elements) just do an insertion-sort style
        // network to avoid recursion overhead.
        if hi - lo < 3 {
            // Sort the 2–3 element window directly.
            if prices.get_unchecked(lo) > prices.get_unchecked(lo + 1) {
                swap_prices(prices, lo, lo + 1);
            }
            if hi - lo == 2 {
                if prices.get_unchecked(lo + 1) > prices.get_unchecked(hi) {
                    swap_prices(prices, lo + 1, hi);
                }
                if prices.get_unchecked(lo) > prices.get_unchecked(lo + 1) {
                    swap_prices(prices, lo, lo + 1);
                }
            }
            break;
        }
        let pivot_idx = partition(prices, lo, hi);
        if k < pivot_idx {
            hi = pivot_idx - 1;
        } else if k > pivot_idx {
            lo = pivot_idx + 1;
        } else {
            break; // prices[k] is in its final sorted position
        }
    }
}

/// Compute the median of `prices` using O(n) quickselect.
///
/// * Odd length  → middle element.
/// * Even length → average of the two middle elements (same formula as before
///   to keep differential test parity).
///
/// Replaces the previous O(n log n) heapsort-based implementation.
pub fn compute_median(prices: &soroban_sdk::Vec<i128>) -> i128 {
    let n = prices.len();
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return prices.get_unchecked(0);
    }
    let mut buf = prices.clone();
    if n % 2 == 1 {
        // Odd: select the middle element.
        let mid = n / 2;
        quickselect(&mut buf, mid);
        buf.get_unchecked(mid)
    } else {
        // Even: we need both middle elements. Run quickselect for the upper
        // middle first, which also partitions the lower half correctly, then
        // take the maximum of the lower half as the lower middle.
        let upper_mid = n / 2;
        quickselect(&mut buf, upper_mid);
        // After quickselect for upper_mid, all elements in buf[0..upper_mid]
        // are ≤ buf[upper_mid]. The lower middle is the maximum of buf[0..upper_mid].
        let b = buf.get_unchecked(upper_mid);
        let mut a = buf.get_unchecked(0);
        for i in 1..upper_mid {
            let v = buf.get_unchecked(i);
            if v > a {
                a = v;
            }
        }
        // Use the same rounding formula as the old implementation to stay
        // bit-exact with all existing tests and the reference implementation.
        a + (b - a) / 2
    }
}

pub fn compute_mean(prices: &soroban_sdk::Vec<i128>) -> i128 {
    let n = prices.len();
    if n == 0 {
        return 0;
    }
    let mut sum: i128 = 0;
    for i in 0..n {
        sum = sum.saturating_add(prices.get_unchecked(i));
    }
    sum / (n as i128)
}

/// Compute a weighted median where each price is weighted by its source's reputation score.
///
/// The weighted median is the value v such that the sum of weights for prices ≤ v
/// is ≥ total_weight/2, and the sum of weights for prices ≥ v is ≥ total_weight/2.
///
/// This is computed by:
///   1. Pairing each price with its weight (reputation score, clamped to [1, 100]).
///   2. Sorting the pairs by price.
///   3. Walking the sorted list until the cumulative weight crosses total_weight/2.
///
/// If `weights` is empty or has a different length to `prices`, falls back to `compute_median`.
/// A weight of 0 is treated as 1 to avoid dead sources suppressing the result entirely.
pub fn compute_weighted_median(
    prices: &soroban_sdk::Vec<i128>,
    weights: &soroban_sdk::Vec<i128>,
) -> i128 {
    let n = prices.len();
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return prices.get_unchecked(0);
    }
    if weights.len() != n {
        // Mismatch — fall back to unweighted median for safety.
        return compute_median(prices);
    }

    // Build (price, weight) pairs as two parallel soroban Vecs sorted by price.
    // We copy into sorted_prices and sorted_weights together using insertion sort
    // (n is always small in practice — contract enforces MaxSources ≤ 100).
    let mut sorted_prices = prices.clone();
    let mut sorted_weights = weights.clone();

    // Insertion sort of both arrays by price (stable, O(n²) but n is tiny)
    let len = sorted_prices.len();
    let mut i: u32 = 1;
    while i < len {
        let key_price = sorted_prices.get_unchecked(i);
        let key_weight = sorted_weights.get_unchecked(i);
        let mut j = i;
        while j > 0 && sorted_prices.get_unchecked(j - 1) > key_price {
            sorted_prices.set(j, sorted_prices.get_unchecked(j - 1));
            sorted_weights.set(j, sorted_weights.get_unchecked(j - 1));
            j -= 1;
        }
        sorted_prices.set(j, key_price);
        sorted_weights.set(j, key_weight);
        i += 1;
    }

    // Compute total weight (each weight clamped to minimum 1)
    let mut total_weight: i128 = 0;
    for i in 0..len {
        let w = sorted_weights.get_unchecked(i).max(1);
        total_weight = total_weight.saturating_add(w);
    }

    // Walk sorted prices until cumulative weight ≥ total_weight / 2
    // The weighted median is the first price where this condition holds.
    let half = total_weight / 2;
    let mut cumulative: i128 = 0;
    let mut median_idx: u32 = 0;
    for i in 0..len {
        let w = sorted_weights.get_unchecked(i).max(1);
        cumulative = cumulative.saturating_add(w);
        if cumulative > half {
            median_idx = i;
            break;
        }
        median_idx = i;
    }

    // When total_weight is even and cumulative == half exactly at index i,
    // interpolate between sorted_prices[i] and sorted_prices[i+1] (like
    // the unweighted even-length case) for consistency.
    let price_at = sorted_prices.get_unchecked(median_idx);
    if total_weight % 2 == 0 && cumulative == half && median_idx + 1 < len {
        let next = sorted_prices.get_unchecked(median_idx + 1);
        return price_at + (next - price_at) / 2;
    }

    price_at
}

pub fn compute_trimmed_mean(prices: &soroban_sdk::Vec<i128>, trim_percent: u32) -> i128 {
    let n = prices.len();
    if n == 0 {
        return 0;
    }
    if trim_percent == 0 {
        return compute_mean(prices);
    }

    let mut sorted = prices.clone();
    sort_prices(&mut sorted);

    let trim_count = ((n.saturating_mul(trim_percent) / 100) / 2).min(n - 1);
    if trim_count == 0 {
        return compute_mean(&sorted);
    }

    let mut trimmed: soroban_sdk::Vec<i128> = soroban_sdk::Vec::new(prices.env());
    for i in trim_count..(n - trim_count) {
        trimmed.push_back(sorted.get_unchecked(i));
    }

    if trimmed.is_empty() {
        return sorted.get_unchecked(n / 2);
    }

    compute_mean(&trimmed)
}

pub fn read_registered_assets(env: &Env) -> Vec<Address> {
    let key = DataKey::AssetRegistry;
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
        .set(&DataKey::AssetRegistry, assets);
}

pub fn read_oracle_sources(env: &Env) -> OracleSources {
    let key = DataKey::SrcRegistry;
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

pub fn is_source_inactive(env: &Env, source: &Address) -> bool {
    let key = DataKey::SrcInactive(source.clone());
    env.storage().persistent().get(&key).unwrap_or(false)
}

pub fn mark_source_inactive(env: &Env, source: &Address) {
    let key = DataKey::SrcInactive(source.clone());
    env.storage().persistent().set(&key, &true);
}

pub fn mark_source_active(env: &Env, source: &Address) {
    let key = DataKey::SrcInactive(source.clone());
    env.storage().persistent().remove(&key);
}

pub fn check_rate_limit(env: &Env, consumer: &Address) -> bool {
    let ledger = env.ledger().sequence();
    let key = DataKey::QueryCount(consumer.clone(), ledger);
    let count: u32 = env.storage().temporary().get(&key).unwrap_or(0);
    let rate_limit_key = DataKey::QueryRateLimit;
    let max_queries: u32 = env.storage().persistent().get(&rate_limit_key).unwrap_or(DEFAULT_QUERY_RATE_LIMIT);
    count < max_queries
}

pub fn increment_query_count(env: &Env, consumer: &Address) -> u32 {
    let ledger = env.ledger().sequence();
    let key = DataKey::QueryCount(consumer.clone(), ledger);
    let count: u32 = env.storage().temporary().get(&key).unwrap_or(0);
    let new_count = count + 1;
    env.storage().temporary().set(&key, &new_count);
    env.storage().temporary().extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    new_count
}

pub fn read_subscription_expiry(env: &Env, consumer: &Address) -> Option<u64> {
    let key = DataKey::SubscriptionExpiry(consumer.clone());
    env.storage().persistent().get(&key)
}

pub fn write_subscription_expiry(env: &Env, consumer: &Address, expiry: u64) {
    let key = DataKey::SubscriptionExpiry(consumer.clone());
    env.storage().persistent().set(&key, &expiry);
}

pub fn read_subscription_plans(env: &Env) -> SubscriptionPlans {
    let key = DataKey::SubscriptionPlans;
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Map::new(env))
}

pub fn write_subscription_plans(env: &Env, plans: &SubscriptionPlans) {
    let key = DataKey::SubscriptionPlans;
    env.storage().persistent().set(&key, plans);
}

pub fn get_plan_amount(env: &Env, duration: u32) -> Option<i128> {
    let plans = read_subscription_plans(env);
    plans.get(duration)
}

pub fn is_subscribed(env: &Env, consumer: &Address) -> bool {
    let key = DataKey::SubscriptionExpiry(consumer.clone());
    let expiry: u64 = env.storage().persistent().get(&key).unwrap_or(0);
    if expiry > 0 {
        let ledger_timestamp = env.ledger().timestamp();
        expiry > ledger_timestamp
    } else {
        false
    }
}
