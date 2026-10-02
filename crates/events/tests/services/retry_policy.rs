/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! RetryPolicy: exponential backoff with cap and jitter, retryable statuses and exhaustion.

use std::time::Duration;

use events::RetryPolicy;

fn no_jitter() -> RetryPolicy {
    RetryPolicy {
        initial_backoff_secs: 5,
        max_backoff_secs: 60,
        multiplier: 2.0,
        jitter_factor: 0.0,
        ..RetryPolicy::default()
    }
}

/// Each attempt doubles the delay until it reaches the maximum.
#[test]
fn delay_grows_exponentially_up_to_the_cap() {
    let policy = no_jitter();
    let delays: Vec<u64> = (1..=6)
        .map(|a| policy.calculate_delay(a).as_secs())
        .collect();
    assert_eq!(delays, vec![5, 10, 20, 40, 60, 60]);
}

/// Attempt zero, or a zero initial backoff, means no wait.
#[test]
fn attempt_zero_or_zero_backoff_is_immediate() {
    assert_eq!(no_jitter().calculate_delay(0), Duration::ZERO);
    let instant = RetryPolicy {
        initial_backoff_secs: 0,
        ..no_jitter()
    };
    assert_eq!(instant.calculate_delay(3), Duration::ZERO);
}

/// Jitter moves the delay at most by its factor either way.
#[test]
fn jitter_stays_within_its_factor() {
    let policy = RetryPolicy {
        jitter_factor: 0.2,
        ..no_jitter()
    };
    for _ in 0..200 {
        let secs = policy.calculate_delay(3).as_secs_f64();
        assert!((16.0..=24.0).contains(&secs), "{secs} outside 20s ± 20%");
    }
}

/// Jitter never brings the delay under a second.
#[test]
fn delay_is_never_under_a_second() {
    let policy = RetryPolicy {
        initial_backoff_secs: 1,
        jitter_factor: 0.9,
        ..no_jitter()
    };
    for _ in 0..200 {
        assert!(policy.calculate_delay(1) >= Duration::from_secs(1));
    }
}

/// Timeouts, throttling and server errors are retried; any other status is final.
#[test]
fn only_408_429_and_5xx_are_retryable() {
    for status in [408, 429, 500, 502, 503, 599] {
        assert!(RetryPolicy::is_retryable_status(status), "{status}");
    }
    for status in [200, 204, 301, 400, 401, 403, 404, 410, 422] {
        assert!(!RetryPolicy::is_retryable_status(status), "{status}");
    }
}

/// A delivery is exhausted once it reaches the attempt limit.
#[test]
fn exhausted_at_the_attempt_limit() {
    let policy = RetryPolicy {
        max_attempts: 3,
        ..no_jitter()
    };
    assert!(!policy.is_exhausted(2));
    assert!(policy.is_exhausted(3));
    assert!(policy.is_exhausted(4));
}

/// The next retry is scheduled the backoff delay after now.
#[test]
fn next_retry_is_now_plus_the_delay() {
    let before = chrono::Utc::now();
    let next = no_jitter().calculate_next_retry(2);
    let offset = (next - before).num_seconds();
    assert!(
        (9..=11).contains(&offset),
        "next retry {offset}s away, expected ~10s"
    );
}
