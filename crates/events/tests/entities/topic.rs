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

//! Topic: which names are valid and how they split into segments.

use events::Topic;

/// Dots and colons both separate segments.
#[test]
fn dots_and_colons_separate_segments() {
    let topic = Topic::new("transfers:dataplane.started").unwrap();
    assert_eq!(topic.segments(), vec!["transfers", "dataplane", "started"]);
}

/// Surrounding whitespace is trimmed.
#[test]
fn surrounding_whitespace_is_trimmed() {
    assert_eq!(
        Topic::new("  catalogs:created ").unwrap().as_str(),
        "catalogs:created"
    );
}

/// An empty or blank topic is rejected.
#[test]
fn empty_topic_is_rejected() {
    assert!(Topic::new("").is_err());
    assert!(Topic::new("   ").is_err());
}

/// A published topic is concrete: wildcards belong to patterns.
#[test]
fn wildcards_are_rejected() {
    assert!(Topic::new("transfers:*").is_err());
    assert!(Topic::new("transfers:start?").is_err());
}

/// Every segment must have content.
#[test]
fn empty_segments_are_rejected() {
    for bad in ["transfers::started", "transfers.", ":started", "a. .b"] {
        assert!(Topic::new(bad).is_err(), "{bad} should be rejected");
    }
}

/// A topic parses from a string the same way `new` builds it.
#[test]
fn parses_from_str() {
    let topic: Topic = "negotiations:agreed".parse().unwrap();
    assert_eq!(topic.to_string(), "negotiations:agreed");
}
