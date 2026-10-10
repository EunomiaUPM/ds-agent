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
//! Response views of the event bus, built from fixture records.

use events::services::event_bus::SubscriptionView;

use crate::support::fixtures;

/// A subscription's HMAC secret never leaves the API in clear: it shows as `*****`.
#[test]
fn subscription_view_masks_the_secret() {
    let record = fixtures::subscription("sub-1", "http://hook.test", Some("s3cr3t"), None);

    let view = SubscriptionView::assemble(record);
    let json = serde_json::to_string(&view).unwrap();

    assert!(json.contains(r#""secret":"*****""#));
    assert!(!json.contains("s3cr3t"));
}

/// A subscription without secret keeps `secret` empty, so the GUI can tell it apart.
#[test]
fn subscription_view_without_secret_stays_empty() {
    let record = fixtures::subscription("sub-1", "http://hook.test", None, Some(3));

    let view = SubscriptionView::assemble(record);

    assert!(view.secret.is_none());
    assert_eq!(view.retry_limit, Some(3));
}
