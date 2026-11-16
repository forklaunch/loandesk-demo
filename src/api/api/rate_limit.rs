//! Request rate limits.
//!
//! Three limits, each a token bucket that refills continuously:
//!
//! - sign-in attempts per client address (`RATE_LIMIT_LOGIN_PER_MINUTE`),
//!   beside the per-account lockout in `AuthService::login`;
//! - every signed-in request per user (`RATE_LIMIT_REQUESTS_PER_MINUTE`);
//! - the expensive operations per user (`RATE_LIMIT_EXPENSIVE_PER_MINUTE`):
//!   search, export, SSN reveal, document grants, password hashing, the
//!   access review ([`EXPENSIVE_OPERATIONS`]).
//!
//! A refused request gets 429 and is logged as the named event `rate_limited`.
//! Buckets live in the memory of one API instance, so with N instances a
//! caller can spend up to N times the limit; that keeps the API stateless and
//! is the usual trade for a limit that needs no shared store. The limit that
//! must hold exactly, five wrong passwords per account, is kept in PostgreSQL.
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

/// Route names (as declared in the contracts) that cost noticeably more than
/// a lookup: they scan, decrypt many fields, sign storage grants or hash a
/// password. `http.rs` checks that every name here is a real route.
pub const EXPENSIVE_OPERATIONS: [&str; 11] = [
    "Search Applications",
    "Export Application",
    "Reveal Owner SSN",
    "Request Document Upload",
    "Complete Document Upload",
    "Download Document",
    "Read Document Content",
    "Create User",
    "Change Password",
    "Access Review",
    "Record Access Review",
];

/// Buckets kept before idle ones are dropped.
const MAX_BUCKETS: usize = 10_000;

struct Bucket {
    tokens: f64,
    updated: Instant,
}

/// A token bucket per key: `per_minute` requests may be made at once, and the
/// allowance refills at `per_minute` per minute.
pub struct RateLimiter {
    per_minute: u32,
    buckets: Mutex<HashMap<String, Bucket>>,
}

impl RateLimiter {
    pub fn new(per_minute: u32) -> Self {
        Self { per_minute, buckets: Mutex::new(HashMap::new()) }
    }

    /// Takes one token for `key`. False when the bucket is empty.
    pub fn allow(&self, key: &str) -> bool {
        self.allow_at(key, Instant::now())
    }

    fn allow_at(&self, key: &str, now: Instant) -> bool {
        let capacity = f64::from(self.per_minute);
        let mut buckets = self.buckets.lock().expect("rate limit lock");
        if buckets.len() >= MAX_BUCKETS && !buckets.contains_key(key) {
            // Drop buckets that have been idle long enough to be full again.
            buckets.retain(|_, bucket| {
                now.saturating_duration_since(bucket.updated) < Duration::from_secs(60)
            });
            if buckets.len() >= MAX_BUCKETS {
                // Still full of active callers: refuse new ones rather than grow.
                return false;
            }
        }
        let bucket =
            buckets.entry(key.to_owned()).or_insert(Bucket { tokens: capacity, updated: now });
        let elapsed = now.saturating_duration_since(bucket.updated).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * capacity / 60.0).min(capacity);
        bucket.updated = now;
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// The address the nearest proxy saw: the last entry of `X-Forwarded-For`
/// (the web container sets the header to the peer address, replacing whatever
/// the client sent). Reduced to characters an address can contain, so nothing
/// else a client sends can reach a log line or grow the bucket table.
pub fn client_address(forwarded_for: Option<&str>) -> String {
    let last = forwarded_for.and_then(|value| value.rsplit(',').next()).unwrap_or("").trim();
    let valid = !last.is_empty()
        && last.len() <= 45
        && last.bytes().all(|b| b.is_ascii_hexdigit() || b == b'.' || b == b':');
    if valid { last.to_owned() } else { "direct".to_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_and_expensive_endpoints_are_rate_limited_per_caller() {
        let limiter = RateLimiter::new(3);
        let start = Instant::now();
        // Three at once, then refused.
        assert!((0..3).all(|_| limiter.allow_at("203.0.113.7", start)));
        assert!(!limiter.allow_at("203.0.113.7", start));
        // Another caller has their own allowance.
        assert!(limiter.allow_at("203.0.113.8", start));
        // One token comes back every 20 seconds (3 per minute), not sooner.
        assert!(!limiter.allow_at("203.0.113.7", start + Duration::from_secs(19)));
        assert!(limiter.allow_at("203.0.113.7", start + Duration::from_secs(21)));
        assert!(!limiter.allow_at("203.0.113.7", start + Duration::from_secs(22)));
        // An idle caller never banks more than the limit.
        let later = start + Duration::from_secs(3600);
        assert!((0..3).all(|_| limiter.allow_at("203.0.113.7", later)));
        assert!(!limiter.allow_at("203.0.113.7", later));
    }

    #[test]
    fn the_bucket_table_cannot_grow_without_bound() {
        let limiter = RateLimiter::new(1);
        let start = Instant::now();
        for caller in 0..MAX_BUCKETS {
            assert!(limiter.allow_at(&caller.to_string(), start));
        }
        // Full of callers seen this minute: a new one is refused.
        assert!(!limiter.allow_at("new", start + Duration::from_secs(1)));
        // A minute later the idle buckets are dropped and there is room again.
        assert!(limiter.allow_at("new", start + Duration::from_secs(120)));
        assert!(limiter.buckets.lock().unwrap().len() <= 1);
    }

    #[test]
    fn the_client_address_is_the_proxy_supplied_one_and_nothing_else() {
        assert_eq!(client_address(Some("198.51.100.4")), "198.51.100.4");
        assert_eq!(client_address(Some("10.0.0.1, 198.51.100.4")), "198.51.100.4");
        assert_eq!(client_address(Some("2001:db8::1")), "2001:db8::1");
        assert_eq!(client_address(Some("Eleanor Whitfield")), "direct");
        assert_eq!(client_address(Some("")), "direct");
        assert_eq!(client_address(None), "direct");
    }
}
