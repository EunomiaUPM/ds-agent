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

use std::collections::HashMap;
use std::hash::Hash;

use crate::validation::rule::Rule;
use crate::validation::violation::Violations;

pub struct Validator<S> {
    stages: Vec<Vec<Box<dyn Rule<S>>>>,
}

impl<S> Default for Validator<S> {
    fn default() -> Self {
        Self {
            stages: vec![Vec::new()],
        }
    }
}

impl<S> Validator<S> {
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

/// Validators by message type. Registering twice for the same key composes:
/// both run and their failures merge, so a profile can add rules without
/// touching the core's.
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

    pub fn register(&mut self, key: K, validator: Validator<S>) {
        self.by_key.entry(key).or_default().push(validator);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::violation::{codes, violation};

    struct Msg {
        pid: Option<String>,
        format: Option<String>,
    }

    fn pid_present(m: &Msg) -> Result<(), Violations> {
        m.pid
            .as_ref()
            .map(|_| ())
            .ok_or_else(|| violation("pid", codes::MISSING, "is required"))
    }
    fn format_present(m: &Msg) -> Result<(), Violations> {
        m.format
            .as_ref()
            .map(|_| ())
            .ok_or_else(|| violation("format", codes::MISSING, "is required"))
    }
    fn pid_is_urn(m: &Msg) -> Result<(), Violations> {
        match m.pid.as_deref() {
            Some(p) if p.starts_with("urn:") => Ok(()),
            _ => Err(violation("pid", codes::MALFORMED, "must be a URN")),
        }
    }

    fn validator() -> Validator<Msg> {
        Validator::new()
            .rule(pid_present)
            .rule(format_present)
            .then()
            .rule(pid_is_urn)
    }

    /// DSP renders several reasons in one error, so a stage reports all of them.
    #[test]
    fn a_stage_reports_every_failure_not_just_the_first() {
        let out = validator().validate(&Msg {
            pid: None,
            format: None,
        });
        assert_eq!(out.unwrap_err().len(), 2);
    }

    /// The second stage would say "must be a URN" about a pid that is not there.
    #[test]
    fn a_failed_stage_stops_the_next_one() {
        let out = validator().validate(&Msg {
            pid: None,
            format: Some("f".into()),
        });
        let vs = out.unwrap_err();
        assert_eq!(vs.len(), 1);
        assert_eq!(vs.code(), Some(codes::MISSING));
    }

    #[test]
    fn later_stages_run_when_the_earlier_ones_pass() {
        let out = validator().validate(&Msg {
            pid: Some("nope".into()),
            format: Some("f".into()),
        });
        assert_eq!(out.unwrap_err().code(), Some(codes::MALFORMED));

        let ok = validator().validate(&Msg {
            pid: Some("urn:uuid:cc".into()),
            format: Some("f".into()),
        });
        assert!(ok.is_ok());
    }

    #[test]
    fn registering_twice_composes_instead_of_replacing() {
        let mut reg: ValidatorRegistry<&str, Msg> = ValidatorRegistry::new();
        reg.register("Request", Validator::new().rule(pid_present));
        reg.register("Request", Validator::new().rule(format_present));
        let vs = reg
            .validate(
                &"Request",
                &Msg {
                    pid: None,
                    format: None,
                },
            )
            .unwrap_err();
        assert_eq!(vs.len(), 2, "both registrations ran");
    }

    #[test]
    fn an_unregistered_key_is_rejected() {
        let reg: ValidatorRegistry<&str, Msg> = ValidatorRegistry::new();
        let subject = Msg {
            pid: Some("urn:uuid:cc".into()),
            format: Some("f".into()),
        };
        assert!(reg.validate(&"Unknown", &subject).is_err());
    }
}
