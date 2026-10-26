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

//! Declarative profile rules: per class, which properties are mandatory, recommended or optional,
//! how many values they take, what kind of value and from which vocabulary.

use crate::entities::graph::{ResourceGraph, Term};
use crate::entities::resource_kind::ResourceKind;
use crate::entities::vocab::Vocab;
use crate::profiles::{Finding, Severity};

/// How strongly a profile asks for a property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    Mandatory,
    Recommended,
    Optional,
}

/// The kind of value a property takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expect {
    /// A named resource (an IRI).
    Iri,
    /// A text literal; language-tagged when the rule set asks for tags.
    Text,
    /// A date or time literal (`xsd:date`, `dateTime`, `gYear`, `gYearMonth`).
    Temporal,
    /// A literal of any datatype.
    Literal,
    /// Anything, including inline nodes.
    Any,
}

/// One property rule.
#[derive(Debug, Clone, Copy)]
pub struct PropertyRule {
    pub property: &'static str,
    pub requirement: Requirement,
    pub max: Option<usize>,
    pub expect: Expect,
    /// The IRI namespace of the controlled vocabulary values must come from.
    pub vocabulary: Option<(&'static str, Severity)>,
}

impl PropertyRule {
    pub const fn mandatory(property: &'static str, expect: Expect) -> Self {
        Self::new(property, Requirement::Mandatory, expect)
    }

    pub const fn recommended(property: &'static str, expect: Expect) -> Self {
        Self::new(property, Requirement::Recommended, expect)
    }

    pub const fn optional(property: &'static str, expect: Expect) -> Self {
        Self::new(property, Requirement::Optional, expect)
    }

    /// At most one value.
    pub const fn single(mut self) -> Self {
        self.max = Some(1);
        self
    }

    /// Values from the vocabulary under `namespace`; others are findings of `severity`.
    pub const fn from_vocabulary(mut self, namespace: &'static str, severity: Severity) -> Self {
        self.vocabulary = Some((namespace, severity));
        self
    }

    const fn new(property: &'static str, requirement: Requirement, expect: Expect) -> Self {
        Self {
            property,
            requirement,
            max: None,
            expect,
            vocabulary: None,
        }
    }
}

/// The rules of one class.
#[derive(Debug, Clone)]
pub struct ClassRules {
    pub kind: ResourceKind,
    pub properties: Vec<PropertyRule>,
}

/// Every class's rules plus profile-wide switches.
#[derive(Debug, Clone, Default)]
pub struct RuleSet {
    pub classes: Vec<ClassRules>,
    /// Untagged texts are findings of this severity, when set.
    pub language_tags: Option<Severity>,
}

impl RuleSet {
    /// The findings for one resource's effective graph.
    pub fn check(&self, kind: ResourceKind, graph: &ResourceGraph) -> Vec<Finding> {
        let resource = graph.id().unwrap_or_default().to_string();
        let Some(class) = self.classes.iter().find(|c| c.kind == kind) else {
            return vec![];
        };
        let mut findings = Vec::new();
        let mut found = |severity: Severity, property: &str, message: String| {
            findings.push(Finding {
                severity,
                resource: resource.clone(),
                property: Some(property.to_string()),
                message,
            });
        };
        for rule in &class.properties {
            let terms = graph.terms(rule.property);
            if terms.is_empty() {
                match rule.requirement {
                    Requirement::Mandatory => found(
                        Severity::Violation,
                        rule.property,
                        format!("{} is mandatory on a {}", Self::name(rule.property), kind.term()),
                    ),
                    Requirement::Recommended => found(
                        Severity::Warning,
                        rule.property,
                        format!("{} is recommended on a {}", Self::name(rule.property), kind.term()),
                    ),
                    Requirement::Optional => {}
                }
                continue;
            }
            if let Some(max) = rule.max {
                if terms.len() > max {
                    found(
                        Severity::Violation,
                        rule.property,
                        format!("{} takes at most {max} value(s), it has {}", Self::name(rule.property), terms.len()),
                    );
                }
            }
            for term in &terms {
                if let Some(problem) = Self::kind_problem(rule.expect, term) {
                    found(Severity::Violation, rule.property, format!("{}: {problem}", Self::name(rule.property)));
                    continue;
                }
                if let (Some(severity), Expect::Text, Term::Literal(literal)) = (self.language_tags, rule.expect, term) {
                    if literal.language.is_none() {
                        found(
                            severity,
                            rule.property,
                            format!("{} \"{}\" has no language tag", Self::name(rule.property), literal.lexical()),
                        );
                    }
                }
                if let Some((namespace, severity)) = rule.vocabulary {
                    let value = Self::lexical(term);
                    if !value.starts_with(namespace) {
                        found(
                            severity,
                            rule.property,
                            format!("{} {value} is not from the vocabulary {namespace}", Self::name(rule.property)),
                        );
                    }
                }
            }
        }
        findings
    }

    fn kind_problem(expect: Expect, term: &Term<'_>) -> Option<String> {
        match (expect, term) {
            (Expect::Any, _) => None,
            (Expect::Iri, Term::Iri(_)) => None,
            (Expect::Iri, Term::Node(node)) if node.contains_key("@id") => None,
            (Expect::Iri, _) => Some("must be an IRI".to_string()),
            (Expect::Text, Term::Literal(l)) if l.value.is_string() && l.datatype.is_none() => None,
            (Expect::Text, _) => Some("must be a text literal".to_string()),
            (Expect::Literal, Term::Literal(_)) => None,
            (Expect::Literal, _) => Some("must be a literal".to_string()),
            (Expect::Temporal, Term::Literal(l)) => match l.datatype {
                Some(Vocab::XSD_DATE | Vocab::XSD_DATE_TIME | Vocab::XSD_G_YEAR | Vocab::XSD_G_YEAR_MONTH) => None,
                _ => Some("must be typed xsd:date, xsd:dateTime, xsd:gYear or xsd:gYearMonth".to_string()),
            },
            (Expect::Temporal, _) => Some("must be a typed date literal".to_string()),
        }
    }

    fn lexical(term: &Term<'_>) -> String {
        match term {
            Term::Iri(iri) => iri.to_string(),
            Term::Literal(literal) => literal.lexical(),
            Term::Node(node) => node
                .get("@id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string(),
        }
    }

    /// `dct:title` rather than the whole IRI, for messages.
    fn name(property: &str) -> String {
        Vocab::PREFIXES
            .iter()
            .filter(|(_, ns)| property.starts_with(ns))
            .max_by_key(|(_, ns)| ns.len())
            .map(|(prefix, ns)| format!("{prefix}:{}", &property[ns.len()..]))
            .unwrap_or_else(|| property.to_string())
    }
}
