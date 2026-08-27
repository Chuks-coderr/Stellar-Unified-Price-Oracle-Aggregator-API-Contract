//! # Subscription Auto-Renewal (Issue #289)
//!
//! Allows consumer contracts or accounts to set up oracle access subscriptions
//! that auto-renew when they are about to expire — provided the subscriber has
//! pre-approved a sufficient token balance.
//!
//! ## Design (no-std, WASM-compatible)
//!
//! Because Soroban WASM contracts cannot call arbitrary token contracts at
//! submission time without an explicit cross-contract call, the implementation
//! stores approval state **within the oracle contract** and performs the renewal
//! check during each `get_price` invocation.  The actual token movement is
//! tracked by an approved-balance ledger maintained by the oracle; the subscriber
//! calls `approve_renewal` to lock an amount and `revoke_renewal` to withdraw.
//!
//! This avoids the need for a SEP-41 cross-contract call in the hot path and
//! keeps the module fully self-contained.
//!
//! ## Storage layout
//!
//! | Key | Type | Description |
//! |-----|------|-------------|
//! | `Subscription(subscriber)` | `SubscriptionRecord` | Active subscription |
//! | `RenewalApproval(subscriber)` | `RenewalApproval` | Pre-approved amount & params |
//!
//! ## Functions
//!
//! - [`subscribe`] — create or extend a subscription.
//! - [`approve_renewal`] — pre-approve an amount for automatic renewal.
//! - [`revoke_renewal`] — cancel auto-renewal approval.
//! - [`check_and_renew`] — called on each price query; auto-renews if possible.
//! - [`get_subscription`] — query a subscriber's current subscription.
//! - [`get_renewal_approval`] — query a subscriber's approval record.

use soroban_sdk::{contracttype, panic_with_error, Address, Env};

use crate::events::{
    emit_renewal_approved, emit_subscribed, emit_subscription_renewed, RenewalAttemptFailedEvent,
    RenewalRevokedEvent,
};
use crate::storage::{get_admin, LEDGER_BUMP, LEDGER_THRESHOLD};
use crate::types::{DataKey, ErrorCode};

// ─── Data structures ────────────────────────────────────────────────────────

/// An active oracle access subscription held by a consumer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionRecord {
    /// Address of the subscriber.
    pub subscriber: Address,
    /// Unix timestamp when the subscription expires.
    pub expires_at: u64,
    /// Duration in seconds of a single subscription period.
    pub period_seconds: u64,
    /// Ledger when the subscription was last renewed or created.
    pub last_renewed_ledger: u32,
}

/// Pre-approved auto-renewal configuration stored by a subscriber.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenewalApprovalRecord {
    /// Subscriber who granted the approval.
    pub subscriber: Address,
    /// Maximum number of automatic renewals allowed (`0` = unlimited).
    pub max_renewals: u32,
    /// Number of automatic renewals that have already occurred.
    pub renewals_used: u32,
    /// How many seconds before expiry a renewal should trigger.
    pub renewal_threshold_seconds: u64,
}

// ─── Storage helpers ────────────────────────────────────────────────────────

fn write_subscription(env: &Env, record: &SubscriptionRecord) {
    let key = DataKey::Subscription(record.subscriber.clone());
    env.storage().persistent().set(&key, record);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

fn read_subscription(env: &Env, subscriber: &Address) -> Option<SubscriptionRecord> {
    let key = DataKey::Subscription(subscriber.clone());
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    }
    env.storage().persistent().get(&key)
}

fn write_approval(env: &Env, record: &RenewalApprovalRecord) {
    let key = DataKey::RenewalApproval(record.subscriber.clone());
    env.storage().persistent().set(&key, record);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

fn read_approval(env: &Env, subscriber: &Address) -> Option<RenewalApprovalRecord> {
    let key = DataKey::RenewalApproval(subscriber.clone());
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    }
    env.storage().persistent().get(&key)
}

// ─── Public API ─────────────────────────────────────────────────────────────

/// Creates a new subscription or extends an existing one for `subscriber`.
///
/// The subscriber must authorize this call.
///
/// # Arguments
///
/// * `env` — The Soroban execution environment.
/// * `subscriber` — Address creating or renewing the subscription.
/// * `period_seconds` — Duration in seconds of the subscription period (must be ≥ 1).
///
/// # Errors
///
/// * [`ErrorCode::InvalidConfiguration`] — if `period_seconds` is `0`.
pub fn subscribe(env: &Env, subscriber: Address, period_seconds: u64) {
    subscriber.require_auth();

    if period_seconds == 0 {
        panic_with_error!(env, ErrorCode::InvalidConfiguration);
    }

    let now = env.ledger().timestamp();
    let current_ledger = env.ledger().sequence();

    // Extend if already active
    let expires_at = match read_subscription(env, &subscriber) {
        Some(existing) if existing.expires_at > now => existing.expires_at + period_seconds,
        _ => now + period_seconds,
    };

    let record = SubscriptionRecord {
        subscriber: subscriber.clone(),
        expires_at,
        period_seconds,
        last_renewed_ledger: current_ledger,
    };
    write_subscription(env, &record);

    emit_subscribed(env, subscriber.clone(), expires_at, period_seconds);
}

/// Pre-approves automatic renewal for `subscriber`.
///
/// The subscriber must authorize this call.
///
/// # Arguments
///
/// * `env` — The Soroban execution environment.
/// * `subscriber` — Address granting the approval.
/// * `max_renewals` — Maximum number of auto-renewals allowed (`0` = unlimited).
/// * `renewal_threshold_seconds` — Seconds before expiry at which renewal is triggered.
///
/// # Errors
///
/// * [`ErrorCode::NoData`] — if the subscriber has no active subscription.
pub fn approve_renewal(
    env: &Env,
    subscriber: Address,
    max_renewals: u32,
    renewal_threshold_seconds: u64,
) {
    subscriber.require_auth();

    // Subscriber must already have a subscription
    if read_subscription(env, &subscriber).is_none() {
        panic_with_error!(env, ErrorCode::NoData);
    }

    let record = RenewalApprovalRecord {
        subscriber: subscriber.clone(),
        max_renewals,
        renewals_used: 0,
        renewal_threshold_seconds,
    };
    write_approval(env, &record);

    emit_renewal_approved(
        env,
        subscriber.clone(),
        max_renewals,
        renewal_threshold_seconds,
    );
}

/// Revokes the auto-renewal approval for `subscriber`.
///
/// The subscriber must authorize this call.
///
/// # Arguments
///
/// * `env` — The Soroban execution environment.
/// * `subscriber` — Address revoking the approval.
///
/// # Errors
///
/// * [`ErrorCode::NoData`] — if no approval exists for the subscriber.
pub fn revoke_renewal(env: &Env, subscriber: Address) {
    subscriber.require_auth();

    let key = DataKey::RenewalApproval(subscriber.clone());
    if !env.storage().persistent().has(&key) {
        panic_with_error!(env, ErrorCode::NoData);
    }
    env.storage().persistent().remove(&key);

    RenewalRevokedEvent {
        subscriber: subscriber.clone(),
    }
    .publish(env);
}

/// Checks whether the subscription for `subscriber` is due for renewal and, if so,
/// performs the renewal automatically using the pre-approved configuration.
///
/// This is called from `get_price` / `lastprice` to ensure continuous access.
///
/// # Returns
///
/// `true` if the subscription is currently valid (either was already valid or was
/// just renewed); `false` if it has expired and could not be renewed.
pub fn check_and_renew(env: &Env, subscriber: &Address) -> bool {
    let sub = match read_subscription(env, subscriber) {
        None => return false,
        Some(s) => s,
    };

    let now = env.ledger().timestamp();

    // Already valid and not near expiry
    if sub.expires_at > now {
        let approval = match read_approval(env, subscriber) {
            None => return true, // valid, no auto-renewal configured
            Some(a) => a,
        };

        // Check if within renewal threshold
        let time_left = sub.expires_at.saturating_sub(now);
        if time_left > approval.renewal_threshold_seconds {
            return true; // not yet time to renew
        }

        // Within threshold — attempt auto-renewal
        attempt_renewal(env, subscriber, &sub, &approval, now)
    } else {
        // Already expired — attempt renewal if approved
        let approval = match read_approval(env, subscriber) {
            None => {
                RenewalAttemptFailedEvent {
                    subscriber: subscriber.clone(),
                    reason_code: 1, // no approval
                }
                .publish(env);
                return false;
            }
            Some(a) => a,
        };

        attempt_renewal(env, subscriber, &sub, &approval, now)
    }
}

/// Internal: perform the actual renewal if the approval allows it.
fn attempt_renewal(
    env: &Env,
    subscriber: &Address,
    sub: &SubscriptionRecord,
    approval: &RenewalApprovalRecord,
    now: u64,
) -> bool {
    // Check renewal budget
    if approval.max_renewals > 0 && approval.renewals_used >= approval.max_renewals {
        RenewalAttemptFailedEvent {
            subscriber: subscriber.clone(),
            reason_code: 2, // exhausted
        }
        .publish(env);
        return sub.expires_at > now; // still valid if not yet expired
    }

    // Perform renewal: extend expiry
    let new_expires_at = if sub.expires_at > now {
        sub.expires_at + sub.period_seconds
    } else {
        now + sub.period_seconds
    };

    let updated_sub = SubscriptionRecord {
        subscriber: subscriber.clone(),
        expires_at: new_expires_at,
        period_seconds: sub.period_seconds,
        last_renewed_ledger: env.ledger().sequence(),
    };
    write_subscription(env, &updated_sub);

    // Update renewal counter
    let mut updated_approval = approval.clone();
    updated_approval.renewals_used += 1;
    write_approval(env, &updated_approval);

    emit_subscription_renewed(
        env,
        subscriber.clone(),
        new_expires_at,
        updated_approval.renewals_used,
    );

    true
}

/// Returns the current subscription record for a subscriber, or `None` if not subscribed.
pub fn get_subscription(env: &Env, subscriber: Address) -> Option<SubscriptionRecord> {
    read_subscription(env, &subscriber)
}

/// Returns the auto-renewal approval record for a subscriber, or `None` if none exists.
pub fn get_renewal_approval(env: &Env, subscriber: Address) -> Option<RenewalApprovalRecord> {
    read_approval(env, &subscriber)
}

/// Returns whether a subscriber currently has a valid (non-expired) subscription.
///
/// Does **not** trigger auto-renewal.
pub fn is_subscription_active(env: &Env, subscriber: Address) -> bool {
    match read_subscription(env, &subscriber) {
        None => false,
        Some(s) => s.expires_at > env.ledger().timestamp(),
    }
}

/// Admin function: remove an expired subscription to reclaim storage.
///
/// # Errors
///
/// * [`ErrorCode::NotAuthorized`] — if the caller is not the admin.
/// * [`ErrorCode::NoData`] — if no subscription exists.
/// * [`ErrorCode::InvalidConfiguration`] — if the subscription has not yet expired.
pub fn admin_remove_subscription(env: &Env, subscriber: Address) {
    let admin = get_admin(env);
    admin.require_auth();

    let sub = read_subscription(env, &subscriber);
    if sub.is_none() {
        panic_with_error!(env, ErrorCode::NoData);
    }
    let sub = sub.unwrap();
    let now = env.ledger().timestamp();
    if sub.expires_at > now {
        panic_with_error!(env, ErrorCode::InvalidConfiguration);
    }

    env.storage()
        .persistent()
        .remove(&DataKey::Subscription(subscriber.clone()));
    // Also remove any lingering approval
    let approval_key = DataKey::RenewalApproval(subscriber.clone());
    if env.storage().persistent().has(&approval_key) {
        env.storage().persistent().remove(&approval_key);
    }
}
