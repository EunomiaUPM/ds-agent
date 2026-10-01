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

//! An ordered set of rules over one subject, grouped into stages.
//!
//! A stage accumulates every failure; the next stage runs only if the previous
//! one passed, so a rule never sees state an earlier rule already rejected.

use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;

use crate::validation::rule::Rule;
use crate::validation::violation::{codes, violation, Path, Violations};

/// Rules over one subject, run in stages.
pub struct Validator<S> {
    stages: Vec<Vec<Box<dyn Rule<S> + Send + Sync>>>,
}

impl<S> Default for Validator<S> {
    fn default() -> Self {
        Self {
            stages: vec![Vec::new()],
        }
    }
}

impl<S> Rule<S> for Validator<S> {
    fn check(&self, subject: &S) -> Result<(), Violations> {
        self.validate(subject)
    }
}

impl<S> Validator<S> {
    /// Validator with one empty stage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a rule to the current stage.
    pub fn rule(mut self, rule: impl Rule<S> + 'static) -> Self {
        self.stages
            .last_mut()
            .expect("a validator always has a stage")
            .push(Box::new(rule));
        self
    }

    /// Open a new stage: what follows runs only if everything so far passed.
    pub fn then(mut self) -> Self {
        self.stages.push(Vec::new());
        self
    }

    /// Assert a boolean condition over the subject in the current stage.
    pub fn ensure(
        self,
        path: impl Into<Path>,
        message: impl Into<Cow<'static, str>>,
        predicate: impl Fn(&S) -> bool + Send + Sync + 'static,
    ) -> Self {
        let p = path.into();
        let msg = message.into();
        self.rule(move |s: &S| {
            if predicate(s) {
                Ok(())
            } else {
                Err(violation(p.clone(), codes::NOT_ALLOWED, msg.clone()))
            }
        })
    }

    /// Ensure an extracted string is a valid URN (`urn:…`).
    pub fn ensure_urn(
        self,
        path: impl Into<Path>,
        extractor: impl Fn(&S) -> Option<&str> + Send + Sync + 'static,
    ) -> Self {
        let p = path.into();
        self.rule(move |s: &S| match extractor(s) {
            Some(val) if val.starts_with("urn:") => Ok(()),
            Some(_) => Err(violation(
                p.clone(),
                codes::MALFORMED,
                "must be a valid URN",
            )),
            None => Err(violation(p.clone(), codes::MISSING, "field is required")),
        })
    }

    /// Ensure an extracted string field is present and not blank.
    pub fn ensure_not_empty(
        self,
        path: impl Into<Path>,
        extractor: impl Fn(&S) -> Option<&str> + Send + Sync + 'static,
    ) -> Self {
        let p = path.into();
        self.rule(move |s: &S| match extractor(s) {
            Some(val) if !val.trim().is_empty() => Ok(()),
            _ => Err(violation(p.clone(), codes::MISSING, "field is required")),
        })
    }

    /// Conditionally run a rule when a predicate holds.
    pub fn when(
        self,
        predicate: impl Fn(&S) -> bool + Send + Sync + 'static,
        rule: impl Rule<S> + 'static,
    ) -> Self {
        self.rule(
            move |s: &S| {
                if predicate(s) {
                    rule.check(s)
                } else {
                    Ok(())
                }
            },
        )
    }

    /// Focus validation onto a nested subfield with a path prefix.
    pub fn field<T: 'static>(
        self,
        prefix: &'static str,
        extractor: impl Fn(&S) -> &T + Send + Sync + 'static,
        sub: Validator<T>,
    ) -> Self {
        self.rule(move |s: &S| {
            let target = extractor(s);
            sub.validate(target).map_err(|vs| vs.with_prefix(prefix))
        })
    }

    /// Focus validation onto an optional nested subfield if present.
    pub fn field_opt<T: 'static>(
        self,
        prefix: &'static str,
        extractor: impl Fn(&S) -> Option<&T> + Send + Sync + 'static,
        sub: Validator<T>,
    ) -> Self {
        self.rule(move |s: &S| {
            if let Some(target) = extractor(s) {
                sub.validate(target).map_err(|vs| vs.with_prefix(prefix))
            } else {
                Ok(())
            }
        })
    }

    /// Validate each element of a collection with an indexed path prefix.
    pub fn each<T: 'static>(
        self,
        prefix: &'static str,
        extractor: impl Fn(&S) -> &[T] + Send + Sync + 'static,
        item_validator: Validator<T>,
    ) -> Self {
        self.rule(move |s: &S| {
            let mut all = Violations::new();
            for (i, item) in extractor(s).iter().enumerate() {
                if let Err(vs) = item_validator.validate(item) {
                    let idx_prefix = format!("{prefix}[{i}]");
                    all.extend(vs.with_prefix(&idx_prefix));
                }
            }
            all.into_result()
        })
    }

    /// Merge stages from another validator into this one.
    pub fn merge(mut self, other: Validator<S>) -> Self {
        let max_len = std::cmp::max(self.stages.len(), other.stages.len());
        self.stages.resize_with(max_len, Vec::new);
        for (i, stage) in other.stages.into_iter().enumerate() {
            self.stages[i].extend(stage);
        }
        self
    }

    /// Runs every stage in order and stops at the first one that fails.
    pub fn validate(&self, subject: &S) -> Result<(), Violations> {
        for stage in &self.stages {
            let mut found = Violations::new();
            for rule in stage {
                if let Err(v) = rule.check(subject) {
                    found.extend(v);
                }
            }
            if !found.is_empty() {
                return Err(found);
            }
        }
        Ok(())
    }
}

/// Validators by message type. Registering a key twice runs both, so a profile can add rules
/// without touching the core's.
pub struct ValidatorRegistry<K, S> {
    by_key: HashMap<K, Vec<Validator<S>>>,
}

impl<K, S> Default for ValidatorRegistry<K, S> {
    fn default() -> Self {
        Self {
            by_key: HashMap::new(),
        }
    }
}

impl<K: Eq + Hash, S> ValidatorRegistry<K, S> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a validator for `key`; earlier ones for the same key keep running.
    pub fn register(&mut self, key: K, validator: Validator<S>) {
        self.by_key.entry(key).or_default().push(validator);
    }

    /// Merge another registry into this one by appending validators.
    pub fn merge(&mut self, other: ValidatorRegistry<K, S>) {
        for (key, validators) in other.by_key {
            self.by_key.entry(key).or_default().extend(validators);
        }
    }

    /// Check if a validator is registered for the specified key.
    pub fn has_key(&self, key: &K) -> bool {
        self.by_key.contains_key(key)
    }

    /// Fails closed: a key with nothing registered is rejected, not waved
    /// through. Forgetting to register is then a test failure, not a hole.
    pub fn validate(&self, key: &K, subject: &S) -> Result<(), Violations> {
        let Some(validators) = self.by_key.get(key) else {
            return Err(crate::validation::violation::violation(
                "@type",
                crate::validation::violation::codes::NOT_ALLOWED,
                "no validator registered for this message type",
            ));
        };
        let mut found = Violations::new();
        for v in validators {
            if let Err(vs) = v.validate(subject) {
                found.extend(vs);
            }
        }
        found.into_result()
    }
}
