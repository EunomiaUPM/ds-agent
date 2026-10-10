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

use super::Secret;

/// Serializing a secret writes the mask, never the value.
#[test]
fn serialize_writes_the_mask() {
    let json = serde_json::to_string(&Secret::new("s3cr3t".to_string())).unwrap();
    assert_eq!(json, "\"*****\"");
}

/// `Debug` hides the value too, so secrets never reach the logs.
#[test]
fn debug_hides_the_value() {
    let printed = format!("{:?}", Secret::new("s3cr3t".to_string()));
    assert!(!printed.contains("s3cr3t"));
}

/// Deserializing reads the value in clear, so secrets stay writable.
#[test]
fn deserialize_reads_the_value() {
    let secret: Secret<serde_json::Value> = serde_json::from_str(r#"{"token":"t"}"#).unwrap();
    assert_eq!(secret.expose(), &serde_json::json!({ "token": "t" }));
}
