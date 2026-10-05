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

//! Participant ids (DIDs) in route paths, encoded as base64url without padding.
//!
//! A `did:web` with a port carries it percent-encoded (`did:web:host%3A3000`), and every
//! percent-decoding on the way (axum, a proxy, the gateway) turns it into `host:3000`, a
//! different DID. Base64url uses only `A-Z a-z 0-9 - _`, which no decoding changes. Callers
//! encode the id with `ymir::utils::encode_url_safe_no_pad`.

use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::utils::decode_url_safe_no_pad;

/// The participant id behind a path segment encoded as base64url without padding; bad-format
/// error if the segment is not one, or does not hold UTF-8 text.
pub(crate) fn decode_path_id(segment: &str) -> Outcome<String> {
    let bytes = decode_url_safe_no_pad(segment)?;
    String::from_utf8(bytes).map_err(|e| {
        Errors::format(
            BadFormat::Received,
            "path id is not UTF-8 text",
            Some(Box::new(e)),
        )
    })
}
