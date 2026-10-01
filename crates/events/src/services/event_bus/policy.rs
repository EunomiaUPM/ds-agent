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

use std::time::Duration;

use chrono::{DateTime, Utc};
use rand::RngExt;
use serde::{Deserialize, Serialize};

// Exponential backoff retry policy with full jitter for external webhook deliveries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_backoff_secs: u64,
    pub max_backoff_secs: u64,
    pub multiplier: f64,
    pub jitter_factor: f64,
    pub timeout_secs: u64,
    pub poll_interval_secs: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_backoff_secs: 5,
            max_backoff_secs: 3600,
            multiplier: 2.0,
            jitter_factor: 0.2,
            timeout_secs: 10,
            poll_interval_secs: 5,
        }
    }
}

impl RetryPolicy {
    // Calculate backoff duration for a given attempt with full jitter.
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 || self.initial_backoff_secs == 0 {
            return Duration::ZERO;
        }

        let base = self.initial_backoff_secs as f64 * self.multiplier.powi(attempt as i32 - 1);
        let capped = base.min(self.max_backoff_secs as f64);

        let jitter_range = capped * self.jitter_factor;
        let mut rng = rand::rng();
        let delta = rng.random_range(-jitter_range..=jitter_range);
        let final_secs = (capped + delta).max(1.0);

        Duration::from_secs_f64(final_secs)
    }

    // Calculate the absolute UTC timestamp for the next retry attempt.
    pub fn calculate_next_retry(&self, attempt: u32) -> DateTime<Utc> {
        let delay = self.calculate_delay(attempt);
        Utc::now() + chrono::Duration::from_std(delay).unwrap_or(chrono::Duration::seconds(5))
    }

    // Determine if an HTTP status code indicates a retryable failure.
    pub fn is_retryable_status(status_code: u16) -> bool {
        matches!(status_code, 408 | 429 | 500..=599)
    }

    // Check if the attempt count has reached or exceeded maximum attempts.
    pub fn is_exhausted(&self, attempts: u32) -> bool {
        attempts >= self.max_attempts
    }

    // Return the HTTP client timeout duration.
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    // Return the background poller scan interval duration.
    pub fn poll_interval(&self) -> Duration {
        Duration::from_secs(self.poll_interval_secs)
    }
}
