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

//! Rules for each key contributed by core, profiles and plugins, run in global stages.
//!
//! Within a stage every rule runs and failures accumulate, whoever registered them; the next
//! stage runs only if this one passed, so no rule sees a subject an earlier stage rejected.

use crate::validation::stage::Stage;
use crate::validation::validator::Validator;
use crate::validation::violation::{codes, violation, Violations};
use std::sync::Arc;
use ymir::registry::{MultiRegistry, Origin, RegistrationId, RegistryKey};

/// A validator and the stage it belongs to.
pub struct StagedValidator<S> {
    pub stage: Stage,
    pub validator: Validator<S>,
}

/// Validators by key from every origin; a key without any core rule is rejected.
pub struct ValidationRegistry<K, S> {
    rules: MultiRegistry<K, StagedValidator<S>>,
}

impl<K: RegistryKey, S: 'static> ValidationRegistry<K, S> {
    /// Empty registry for the extension point `point`.
    pub fn new(point: &'static str) -> Self {
        Self {
            rules: MultiRegistry::new(point),
        }
    }

    pub fn point(&self) -> &'static str {
        self.rules.point()
    }

    /// Boot-time builder: adds a core rule for `key`.
    pub fn with(self, key: K, stage: Stage, validator: Validator<S>) -> Self {
        self.register(key, stage, validator, Origin::Core);
        self
    }

    /// Boot-time builder: adds a core rule that applies to every key.
    pub fn with_any(self, stage: Stage, validator: Validator<S>) -> Self {
        self.register_any(stage, validator, Origin::Core);
        self
    }

    /// Adds a rule for `key` at runtime, from a profile or a plugin.
    pub fn register(
        &self,
        key: K,
        stage: Stage,
        validator: Validator<S>,
        origin: Origin,
    ) -> RegistrationId {
        self.rules.register(
            key,
            Arc::new(StagedValidator { stage, validator }),
            origin,
            0,
        )
    }

    /// Adds a rule for every key at runtime.
    pub fn register_any(
        &self,
        stage: Stage,
        validator: Validator<S>,
        origin: Origin,
    ) -> RegistrationId {
        self.rules
            .register_any(Arc::new(StagedValidator { stage, validator }), origin, 0)
    }

    /// Whether `key` has a core rule, the condition for it to validate at all.
    pub fn has_key(&self, key: &K) -> bool {
        self.rules
            .snapshot()
            .for_key(key)
            .iter()
            .any(|e| e.origin == Origin::Core)
    }

    /// Drops every rule `origin` registered; core rules stay.
    pub fn unregister_origin(&self, origin: &Origin) -> usize {
        self.rules.unregister_origin(origin)
    }

    /// Runs the stages in order over one snapshot; fails closed when `key` has no core rule.
    pub fn validate(&self, key: &K, subject: &S) -> Result<(), Violations> {
        let snapshot = self.rules.snapshot();
        if !snapshot
            .for_key(key)
            .iter()
            .any(|e| e.origin == Origin::Core)
        {
            return Err(violation(
                "@type",
                codes::NOT_ALLOWED,
                "no validator registered for this message type",
            ));
        }
        let entries = snapshot.all(key);
        for stage in Stage::ALL {
            let mut found = Violations::new();
            for entry in entries.iter().filter(|e| e.value.stage == stage) {
                if let Err(vs) = entry.value.validator.validate(subject) {
                    found.extend(vs.with_origin(&entry.origin));
                }
            }
            if !found.is_empty() {
                return Err(found);
            }
        }
        Ok(())
    }
}
