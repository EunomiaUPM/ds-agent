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

//! Deserialization defaults, roundtrips, and validation of `InteractionConfig`
//! (`lifetime: INFINITE | FINITE`, `maxRetries`).

use connector::entities::interaction::{
    InteractionConfig, TransferLifetime, MAX_INTERACTION_RETRIES,
};
use serde_json::json;

#[test]
fn pull_without_lifetime_or_max_retries_defaults_to_infinite_and_none() {
    let raw = json!({
        "mode": "PULL",
        "dataAccess": {
            "protocol": "HTTP",
            "urlTemplate": "https://api.example.com/data",
            "method": ["GET"]
        }
    });

    let cfg: InteractionConfig = serde_json::from_value(raw).expect("valid pull interaction");
    assert_eq!(cfg.lifetime(), TransferLifetime::Infinite);
    assert_eq!(cfg.max_retries(), None);
    assert!(cfg.validate().is_ok());

    let serialized = serde_json::to_value(&cfg).unwrap();
    assert_eq!(serialized["lifetime"], "INFINITE");
    assert!(serialized.get("maxRetries").is_none());
}

#[test]
fn pull_finite_with_max_retries_round_trips() {
    let raw = json!({
        "mode": "PULL",
        "lifetime": "FINITE",
        "maxRetries": 2,
        "dataAccess": {
            "protocol": "HTTP",
            "urlTemplate": "https://api.example.com/snapshot",
            "method": ["GET"]
        }
    });

    let cfg: InteractionConfig = serde_json::from_value(raw).expect("valid finite pull");
    assert_eq!(cfg.lifetime(), TransferLifetime::Finite);
    assert_eq!(cfg.max_retries(), Some(2));
    assert!(cfg.validate().is_ok());

    let serialized = serde_json::to_value(&cfg).unwrap();
    assert_eq!(serialized["mode"], "PULL");
    assert_eq!(serialized["lifetime"], "FINITE");
    assert_eq!(serialized["maxRetries"], 2);
}

#[test]
fn push_finite_and_infinite_round_trip() {
    let raw = json!({
        "mode": "PUSH",
        "lifetime": "FINITE",
        "maxRetries": 0,
        "subscribe": {
            "protocol": "HTTP",
            "urlTemplate": "https://producer.example.com/hooks",
            "method": ["POST"]
        },
        "unsubscribe": null
    });

    let cfg: InteractionConfig = serde_json::from_value(raw).expect("valid finite push");
    assert_eq!(cfg.lifetime(), TransferLifetime::Finite);
    assert_eq!(cfg.max_retries(), Some(0));
    assert!(cfg.validate().is_ok());

    let serialized = serde_json::to_value(&cfg).unwrap();
    assert_eq!(serialized["mode"], "PUSH");
    assert_eq!(serialized["lifetime"], "FINITE");
    assert_eq!(serialized["maxRetries"], 0);
}

#[test]
fn validate_rejects_max_retries_above_cap() {
    let raw = json!({
        "mode": "PULL",
        "lifetime": "INFINITE",
        "maxRetries": MAX_INTERACTION_RETRIES + 1,
        "dataAccess": {
            "protocol": "HTTP",
            "urlTemplate": "https://api.example.com/data",
            "method": ["GET"]
        }
    });

    let cfg: InteractionConfig = serde_json::from_value(raw).unwrap();
    let err = cfg.validate().unwrap_err();
    assert!(
        format!("{err:#}").contains("maxRetries"),
        "expected maxRetries validation error, got: {err:#}"
    );
}
