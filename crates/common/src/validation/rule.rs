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

/// One pure check over a subject.
pub trait Rule<S>: Send + Sync {
    /// `Ok` or every violation found.
    fn check(&self, subject: &S) -> Result<(), Violations>;
}

/// Any function of the right shape is a rule. Only one such impl is possible: a second `Fn`
/// signature cannot be proven disjoint.
impl<S, F> Rule<S> for F
where
    F: Fn(&S) -> Result<(), Violations> + Send + Sync,
{
    fn check(&self, subject: &S) -> Result<(), Violations> {
        self(subject)
    }
}
