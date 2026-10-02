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

//! What a failed check reports.
//!
//! Deliberately protocol-neutral: a [`Violation`] knows nothing about HTTP or
//! about DSP. Rendering it onto a wire error is the job of whichever protocol is
//! answering — see [`crate::validation::render`].

use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

/// Where in the subject the problem is, e.g. `dataAddress.endpointProperties[0].name`.
/// Literal paths allocate nothing; only nested and indexed ones build a string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(Cow<'static, str>);

impl Path {
    /// Path made of one field name.
    pub const fn field(name: &'static str) -> Self {
        Self(Cow::Borrowed(name))
    }

    /// `parent.child`
    pub fn child(&self, name: &str) -> Self {
        Self(Cow::Owned(format!("{}.{}", self.0, name)))
    }

    /// `prefix.parent`
    pub fn prefix(&self, prefix: &str) -> Self {
        if self.0.is_empty() {
            Self(Cow::Owned(prefix.to_string()))
        } else {
            Self(Cow::Owned(format!("{prefix}.{}", self.0)))
        }
    }

    /// `parent[i]`
    pub fn index(&self, i: usize) -> Self {
        Self(Cow::Owned(format!("{}[{}]", self.0, i)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&'static str> for Path {
    fn from(s: &'static str) -> Self {
        Self::field(s)
    }
}

/// Stable, machine-readable kind of failure. `common` owns only the shared [`codes`]; agents
/// number their own, and a published code is never reassigned because peers rely on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViolationCode(pub u32);

impl Display for ViolationCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Codes shared by every protocol. Agent-specific codes start at 1000.
pub mod codes {
    use super::ViolationCode;

    /// A required member is absent.
    pub const MISSING: ViolationCode = ViolationCode(1);
    /// Present, but not the shape the field requires (not a URI, not a URL, …).
    pub const MALFORMED: ViolationCode = ViolationCode(2);
    /// A term that must hold exactly one value holds several.
    pub const NOT_SINGLE_VALUED: ViolationCode = ViolationCode(3);
    /// Present, but not allowed here.
    pub const NOT_ALLOWED: ViolationCode = ViolationCode(4);
}

/// One failed check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub path: Path,
    pub code: ViolationCode,
    pub message: Cow<'static, str>,
    /// Offending value echoed back to the peer. Set it only where it cannot be a credential:
    /// fields like `endpointProperties[].value` routinely carry bearer tokens.
    pub value: Option<String>,
}

impl Violation {
    /// Violation without an echoed value.
    pub fn new(
        path: impl Into<Path>,
        code: ViolationCode,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            path: path.into(),
            code,
            message: message.into(),
            value: None,
        }
    }

    /// Attach the offending value. Read the note on [`Violation::value`] first.
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// `<path>: <message>` — the form that goes into a wire error's reason list.
    pub fn to_reason(&self) -> String {
        format!("{}: {}", self.path, self.message)
    }

    /// Prefix this violation's path with `prefix`.
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.path = self.path.prefix(prefix);
        self
    }
}

/// One or more failures, in the order the rules ran. Stages run in dependency order, so the
/// first entry is the most fundamental one, which is what [`Violations::code`] reports.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Violations(Vec<Violation>);

impl Violations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn one(v: Violation) -> Self {
        Self(vec![v])
    }

    pub fn push(&mut self, v: Violation) {
        self.0.push(v);
    }

    pub fn extend(&mut self, other: Violations) {
        self.0.extend(other.0);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Violation> {
        self.0.iter()
    }

    /// Prefix all violations with a path prefix.
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        for v in &mut self.0 {
            v.path = v.path.prefix(prefix);
        }
        self
    }

    /// Extract human-readable error messages without path prefixes.
    pub fn messages(&self) -> Vec<String> {
        self.0.iter().map(|v| v.message.to_string()).collect()
    }

    /// Formatted reason strings: `<path>: <message>`.
    pub fn to_reasons(&self) -> Vec<String> {
        self.0.iter().map(Violation::to_reason).collect()
    }

    /// The code that represents this failure as a whole: the first one, which is
    /// the most fundamental. `None` only when empty.
    pub fn code(&self) -> Option<ViolationCode> {
        self.0.first().map(|v| v.code)
    }

    /// `Ok(())` when empty, `Err(self)` otherwise — for returning at the end of a
    /// check that accumulated as it went.
    pub fn into_result(self) -> Result<(), Violations> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl From<Violation> for Violations {
    fn from(v: Violation) -> Self {
        Self::one(v)
    }
}

impl IntoIterator for Violations {
    type Item = Violation;
    type IntoIter = std::vec::IntoIter<Violation>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Violation> for Violations {
    fn from_iter<I: IntoIterator<Item = Violation>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Display for Violations {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let joined: Vec<_> = self.0.iter().map(Violation::to_reason).collect();
        f.write_str(&joined.join("; "))
    }
}

/// Shorthand for the overwhelmingly common case: one failure, no value echoed.
pub fn violation(
    path: impl Into<Path>,
    code: ViolationCode,
    message: impl Into<Cow<'static, str>>,
) -> Violations {
    Violations::one(Violation::new(path, code, message))
}
