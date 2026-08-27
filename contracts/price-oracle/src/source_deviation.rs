//! On-chain Per-Source Deviation Report (#303)
//!
//! Tracks, per `(source, asset)` pair, the deviation of each submitted price
//! from the aggregate at the time of submission.  A ring-buffer of the most
//! recent `MAX_DEVIATION_RECORDS` records is stored under
//! [`DataKey::SourceDeviationHistory`].  At any time callers may request a
//! [`DeviationReport`] summarising the last `num_rounds` submissions.
//!
//! ## Deviation formula
//!
//! ```text
//! deviation_bps = |submitted_price - aggregate_price| * 10_000 / aggregate_price
//! ```
//!
//! When no aggregate exists for the asset yet the deviation is recorded as `0`.
//!
//! ## Outlier definition
//!
//! A submission is counted as an outlier when its deviation exceeds
//! `OUTLIER_THRESHOLD_BPS` (200 bps = 2 %).

use soroban_sdk::{Address, Env, Vec};

use crate::storage::{LEDGER_BUMP, LEDGER_THRESHOLD};
use crate::types::{AggregatePrice, DataKey, DeviationRecord, DeviationReport};

/// Maximum number of deviation records kept per (source, asset) pair.
pub const MAX_DEVIATION_RECORDS: u32 = 100;

/// Submissions deviating by more than this many basis points are counted as outliers.
pub const OUTLIER_THRESHOLD_BPS: i128 = 200;

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn history_key(source: &Address, asset: &Address) -> DataKey {
    DataKey::SourceDeviationHistory(source.clone(), asset.clone())
}

fn read_history(env: &Env, source: &Address, asset: &Address) -> Vec<DeviationRecord> {
    let key = history_key(source, asset);
    let records: Option<Vec<DeviationRecord>> = env.storage().persistent().get(&key);
    match records {
        Some(v) => {
            env.storage()
                .persistent()
                .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
            v
        }
        None => Vec::new(env),
    }
}

fn write_history(env: &Env, source: &Address, asset: &Address, records: &Vec<DeviationRecord>) {
    let key = history_key(source, asset);
    env.storage().persistent().set(&key, records);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Records the deviation of `submitted_price` from the current aggregate for
/// `(source, asset)`.  Called from within `submit_price` after the submission
/// is stored but **before** aggregation so the record captures the *previous*
/// aggregate.
///
/// When no aggregate exists yet (first submission) the deviation is stored as `0`.
pub fn record_deviation(env: &Env, source: &Address, asset: &Address, submitted_price: i128) {
    // Read the previous aggregate (may not exist yet).
    let agg_key = DataKey::Aggregate(asset.clone());
    let maybe_agg: Option<AggregatePrice> = env.storage().persistent().get(&agg_key);

    let deviation_bps: i128 = if let Some(agg) = maybe_agg {
        if agg.price > 0 {
            let diff = if submitted_price > agg.price {
                submitted_price - agg.price
            } else {
                agg.price - submitted_price
            };
            diff.saturating_mul(10_000) / agg.price
        } else {
            0
        }
    } else {
        0
    };

    let record = DeviationRecord {
        deviation_bps,
        ledger: env.ledger().sequence(),
    };

    let mut history = read_history(env, source, asset);

    // Enforce ring-buffer limit: drop the oldest entry when full.
    while history.len() >= MAX_DEVIATION_RECORDS {
        history.remove(0);
    }

    history.push_back(record);
    write_history(env, source, asset, &history);
}

/// Returns a [`DeviationReport`] for the last `num_rounds` submissions made by
/// `source` for `asset`.
///
/// If fewer than `num_rounds` records exist, all available records are used and
/// the report's `num_rounds` field reflects the actual count.
pub fn get_source_deviation_report(
    env: &Env,
    source: Address,
    asset: Address,
    num_rounds: u32,
) -> DeviationReport {
    let history = read_history(env, &source, &asset);
    let total = history.len();

    if total == 0 || num_rounds == 0 {
        return DeviationReport {
            avg_deviation_bps: 0,
            max_deviation_bps: 0,
            outlier_count: 0,
            trend: 0,
            num_rounds: 0,
        };
    }

    // Take the last `num_rounds` records (or all if fewer exist).
    let take = if num_rounds < total { num_rounds } else { total };
    let start = total - take;

    let mut sum: i128 = 0;
    let mut max_dev: i128 = 0;
    let mut outliers: u32 = 0;

    // Collect values for trend calculation.
    // Trend (slope) is computed via a simple linear regression over indices:
    //   slope = (n * Σ(i * y_i) - Σi * Σy_i) / (n * Σ(i²) - (Σi)²)
    // All values are integer-approximated; the result is in bps/round (i32).
    let mut sum_i: i128 = 0;
    let mut sum_y: i128 = 0;
    let mut sum_iy: i128 = 0;
    let mut sum_i2: i128 = 0;

    for idx in 0..take {
        let record = history.get_unchecked(start + idx);
        let dev = record.deviation_bps;

        sum += dev;
        if dev > max_dev {
            max_dev = dev;
        }
        if dev > OUTLIER_THRESHOLD_BPS {
            outliers += 1;
        }

        let i = idx as i128;
        sum_i += i;
        sum_y += dev;
        sum_iy += i * dev;
        sum_i2 += i * i;
    }

    let n = take as i128;
    let avg_dev = if n > 0 { sum / n } else { 0 };

    // Linear regression slope (integer approximation).
    let trend: i32 = if n > 1 {
        let numerator = n * sum_iy - sum_i * sum_y;
        let denominator = n * sum_i2 - sum_i * sum_i;
        if denominator != 0 {
            (numerator / denominator) as i32
        } else {
            0
        }
    } else {
        0
    };

    DeviationReport {
        avg_deviation_bps: avg_dev,
        max_deviation_bps: max_dev,
        outlier_count: outliers,
        trend,
        num_rounds: take,
    }
}
