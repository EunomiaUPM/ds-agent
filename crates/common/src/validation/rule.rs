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

//! One check over one subject. Rules are pure and synchronous: anything that
//! needs a lookup belongs to whoever loads the facts, not here.

use crate::validation::violation::Violations;

pub trait Rule<S>: Send + Sync {
    fn check(&self, subject: &S) -> Result<(), Violations>;
}

/// Any function of the right shape is a rule, with no wrapper.
///
/// There is deliberately only one such impl: a second one over a different `Fn`
/// signature cannot be proven disjoint and the compiler rejects it.
impl<S, F> Rule<S> for F
where
    F: Fn(&S) -> Result<(), Violations> + Send + Sync,
{
    fn check(&self, subject: &S) -> Result<(), Violations> {
        self(subject)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::violation::{codes, violation};

    struct Msg {
        pid: Option<String>,
    }

    fn pid_required(m: &Msg) -> Result<(), Violations> {
        match &m.pid {
            Some(p) if !p.is_empty() => Ok(()),
            _ => Err(violation("pid", codes::MISSING, "is required")),
        }
    }

    #[test]
    fn a_plain_function_is_a_rule() {
        let rules: Vec<Box<dyn Rule<Msg>>> = vec![Box::new(pid_required)];
        assert!(rules[0]
            .check(&Msg {
                pid: Some("x".into())
            })
            .is_ok());
        assert!(rules[0].check(&Msg { pid: None }).is_err());
    }

    #[test]
    fn a_closure_carrying_data_is_a_rule_too() {
        let forbidden = "urn:uuid:bad".to_string();
        let not_forbidden = move |m: &Msg| match &m.pid {
            Some(p) if *p == forbidden => Err(violation("pid", codes::NOT_ALLOWED, "is denied")),
            _ => Ok(()),
        };
        let rules: Vec<Box<dyn Rule<Msg>>> = vec![Box::new(not_forbidden)];
        assert!(rules[0]
            .check(&Msg {
                pid: Some("urn:uuid:bad".into())
            })
            .is_err());
    }
}
