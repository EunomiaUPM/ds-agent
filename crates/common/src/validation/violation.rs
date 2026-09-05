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

/// Where in the subject the problem is: `consumerPid`,
/// `dataAddress.endpointProperties[0].name`.
///
/// Almost every path is a literal known at compile time, so the common case
/// allocates nothing; only the nested-with-index forms build a string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(Cow<'static, str>);

impl Path {
    pub const fn field(name: &'static str) -> Self {
        Self(Cow::Borrowed(name))
    }

    /// `parent.child`
    pub fn child(&self, name: &str) -> Self {
        Self(Cow::Owned(format!("{}.{}", self.0, name)))
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

/// A stable, machine-readable classification of the failure.
///
/// Transparent on purpose: this crate owns only the codes that recur in every
/// protocol (see [`codes`]); each agent defines its own vocabulary in its own
/// numbering space. Peers program against these, so a code must never be
/// reassigned once published.
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
    /// The offending value, when echoing it helps the sender and is safe.
    ///
    /// Never populate this automatically from whatever field failed. Messages
    /// carry bearer tokens (`endpointProperties[].value` routinely does), and
    /// this field travels back to the peer inside the error body. Set it only
    /// where the value is known not to be a credential.
    pub value: Option<String>,
}

impl Violation {
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
}

/// One or more failures, in the order the rules ran.
///
/// Order is meaningful: rules run in registration order within a stage, and
/// stages run in dependency order, so the first entry is the most fundamental
/// failure. That is what [`Violations::code`] reports.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_paths_read_like_the_document() {
        let p = Path::field("dataAddress")
            .child("endpointProperties")
            .index(0)
            .child("name");
        assert_eq!(p.as_str(), "dataAddress.endpointProperties[0].name");
    }

    #[test]
    fn the_representative_code_is_the_first_failure() {
        // Stages run in dependency order, so "it is missing" outranks whatever a
        // later rule said about the same subject.
        let mut vs = Violations::new();
        vs.push(Violation::new("consumerPid", codes::MISSING, "is required"));
        vs.push(Violation::new("format", codes::MALFORMED, "is not a URI"));
        assert_eq!(vs.code(), Some(codes::MISSING));
        assert_eq!(vs.len(), 2);
    }

    #[test]
    fn an_empty_set_is_a_pass() {
        assert!(Violations::new().into_result().is_ok());
        assert!(Violations::new().code().is_none());
    }

    #[test]
    fn a_value_is_never_attached_unless_asked_for() {
        let v = Violation::new("x", codes::MISSING, "is required");
        assert!(v.value.is_none());
        assert_eq!(v.with_value("42").value.as_deref(), Some("42"));
    }
}
