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

//! TopicPattern: `*` alone is everything, a trailing `*` everything under a prefix, an inner
//! `*` one segment, and the SQL regex that must agree with the matcher.

use events::{Topic, TopicPattern};

fn matches(pattern: &str, topic: &str) -> bool {
    TopicPattern::new(pattern)
        .unwrap()
        .matches(&Topic::new(topic).unwrap())
}

const TOPICS: [&str; 7] = [
    "transfer",
    "transfer.process",
    "transfer.process.create",
    "transfer.process.create.retry",
    "transfer.message.create",
    "catalog.dataset.create",
    "transfers:started",
];

/// `*` alone matches every topic.
#[test]
fn lone_star_matches_everything() {
    for topic in TOPICS {
        assert!(matches("*", topic), "{topic}");
    }
    assert_eq!(TopicPattern::match_all().as_str(), "*");
}

/// `transfer.*` matches everything under `transfer`, at any depth, but not `transfer` itself.
#[test]
fn trailing_star_matches_everything_under_the_prefix() {
    assert!(matches("transfer.*", "transfer.process"));
    assert!(matches("transfer.*", "transfer.process.create"));
    assert!(matches("transfer.*", "transfer.message.create"));
    assert!(!matches("transfer.*", "transfer"));
    assert!(!matches("transfer.*", "catalog.dataset.create"));
}

/// `transfer.process.*` matches everything under `transfer.process` and nothing beside it.
#[test]
fn deeper_trailing_star_narrows_to_that_prefix() {
    assert!(matches("transfer.process.*", "transfer.process.create"));
    assert!(matches(
        "transfer.process.*",
        "transfer.process.create.retry"
    ));
    assert!(!matches("transfer.process.*", "transfer.message.create"));
    assert!(!matches("transfer.process.*", "transfer.process"));
}

/// A pattern without wildcards matches only that exact topic.
#[test]
fn exact_pattern_matches_only_its_topic() {
    assert!(matches(
        "transfer.process.create",
        "transfer.process.create"
    ));
    assert!(!matches(
        "transfer.process.create",
        "transfer.process.create.retry"
    ));
    assert!(!matches("transfer.process.create", "transfer.process"));
}

/// Dots and colons are interchangeable separators.
#[test]
fn dot_and_colon_are_interchangeable() {
    assert!(matches("transfers.*", "transfers:started"));
    assert!(matches("transfers:started", "transfers.started"));
}

/// A `*` in the middle stands for exactly one segment.
#[test]
fn inner_star_matches_one_segment() {
    assert!(matches("transfer.*.create", "transfer.process.create"));
    assert!(matches("transfer.*.create", "transfer.message.create"));
    assert!(!matches(
        "transfer.*.create",
        "transfer.process.create.retry"
    ));
    assert!(!matches("transfer.*.create", "transfer.a.b.create"));
}

/// `**` keeps matching any number of segments, including none.
#[test]
fn double_star_matches_zero_or_more_segments() {
    assert!(matches("**", "transfer.process.create"));
    assert!(matches("transfer.**", "transfer"));
    assert!(matches("transfer.**", "transfer.process.create"));
    assert!(matches("**.create", "transfer.process.create"));
    assert!(matches("a.**.z", "a.z"));
    assert!(matches("a.**.z", "a.b.c.z"));
    assert!(!matches("a.**.z", "a.b"));
}

/// An empty pattern is rejected and surrounding whitespace is trimmed.
#[test]
fn empty_pattern_is_rejected_and_whitespace_trimmed() {
    assert!(TopicPattern::new("  ").is_err());
    assert_eq!(TopicPattern::new(" a.* ").unwrap().as_str(), "a.*");
    assert!(TopicPattern::new("a.*").unwrap().has_wildcard());
    assert!(!TopicPattern::new("a.b").unwrap().has_wildcard());
}

/// The SQL regex encodes each form of pattern.
#[test]
fn sql_regex_per_form() {
    let cases = [
        ("*", ".*"),
        ("**", ".*"),
        ("transfer.process.create", "^transfer[.:]process[.:]create$"),
        ("transfer.*", "^transfer([.:][^.:]+)+$"),
        ("transfer.*.create", "^transfer[.:][^.:]+[.:]create$"),
        ("transfer.**", "^transfer([.:][^.:]+)*$"),
        ("**.create", "^([^.:]+[.:])*create$"),
    ];
    for (pattern, regex) in cases {
        assert_eq!(
            TopicPattern::new(pattern).unwrap().to_sql_regex(),
            regex,
            "{pattern}"
        );
    }
}

/// Characters that mean something in a regex are escaped.
#[test]
fn sql_regex_escapes_special_characters() {
    assert_eq!(
        TopicPattern::new("a+b.c_d-e").unwrap().to_sql_regex(),
        r"^a\+b[.:]c_d-e$"
    );
}

/// The matcher and the SQL regex agree on every pattern and topic, so in-memory and database
/// filters give the same answer.
#[test]
fn sql_regex_agrees_with_the_matcher() {
    let patterns = [
        "*",
        "**",
        "transfer.*",
        "transfer.process.*",
        "transfer.process.create",
        "transfer.*.create",
        "transfer.**",
        "**.create",
        "*.*",
        "**.*",
        "transfers:*",
    ];
    for pattern in patterns {
        let parsed = TopicPattern::new(pattern).unwrap();
        let regex = regex::Regex::new(&parsed.to_sql_regex()).unwrap();
        for topic in TOPICS {
            assert_eq!(
                regex.is_match(topic),
                parsed.matches(&Topic::new(topic).unwrap()),
                "{pattern} vs {topic}"
            );
        }
    }
}
