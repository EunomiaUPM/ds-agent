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

//! A SHACL validation report as the agents serve it: one result per failed constraint, with a
//! message a person can read.

use rudof_rdf::rdf_core::term::Object;
use rudof_rdf::rdf_core::SHACLPath;
use serde::Serialize;
use shacl::validator::report::ValidationResult;

use crate::rdf::graph::RdfGraph;

/// How bad a result is: a violation breaks conformance, the rest do not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ShaclSeverity {
    Violation,
    Warning,
    Info,
}

/// One result of a validation.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShaclResult {
    /// The shape group that raised it.
    pub check: String,
    pub severity: ShaclSeverity,
    pub focus_node: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// The constraint component's local name, e.g. `MinCount`.
    pub constraint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_shape: Option<String>,
    pub message: String,
}

/// The outcome of validating one graph against some shape groups.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShaclReport {
    /// Conforms when no result is a violation.
    pub conforms: bool,
    pub checks: Vec<String>,
    pub violations: usize,
    pub warnings: usize,
    pub results: Vec<ShaclResult>,
}

impl ShaclReport {
    pub fn new(checks: Vec<String>, results: Vec<ShaclResult>) -> Self {
        let count = |s: ShaclSeverity| results.iter().filter(|r| r.severity == s).count();
        let violations = count(ShaclSeverity::Violation);
        Self {
            conforms: violations == 0,
            checks,
            violations,
            warnings: count(ShaclSeverity::Warning),
            results,
        }
    }

    /// Results on a blank node name the IRI-named subject holding it instead, so each is about
    /// something a reader can find; the message says it was a nested node.
    pub fn attributed(mut self, graph: &RdfGraph) -> Self {
        for result in &mut self.results {
            if !result.focus_node.starts_with("_:") {
                continue;
            }
            if let Some(owner) = graph.named_owner(&result.focus_node) {
                result.focus_node = owner;
                result.message = format!("in a nested node, {}", result.message);
            }
        }
        self
    }

    /// Only the results about one of `nodes`.
    pub fn about(self, nodes: &[String]) -> Self {
        let results = self
            .results
            .into_iter()
            .filter(|r| nodes.contains(&r.focus_node))
            .collect();
        Self::new(self.checks, results)
    }
}

impl ShaclResult {
    const COMPONENT_SUFFIX: &'static str = "ConstraintComponent";

    /// Reads one result of rudof's report, compacting IRIs in its message with `prefixes`.
    pub fn read(check: &str, result: &ValidationResult, prefixes: &[(&str, &str)]) -> Self {
        let constraint = Self::local_name(&result.constraint_component().to_string());
        let path = result.path().map(Self::path_text);
        let value = result.value().map(Self::term_text);
        let generic = result.message().get(None).cloned().unwrap_or_default();
        let custom = result
            .message()
            .iter()
            .find(|(lang, _)| lang.as_ref().is_some_and(|l| l.to_string() == "en"))
            .or_else(|| result.message().iter().find(|(lang, _)| lang.is_some()))
            .map(|(_, m)| m.clone());
        let severity = Self::severity(result.severity());
        let message = custom.unwrap_or_else(|| {
            Self::message(
                &constraint,
                severity,
                path.as_deref(),
                value.as_deref(),
                &generic,
                prefixes,
            )
        });
        Self {
            check: check.to_string(),
            severity,
            focus_node: Self::term_text(result.focus_node()),
            path,
            value,
            constraint,
            source_shape: result.source().map(Self::term_text),
            message,
        }
    }

    fn severity(severity: &shacl::types::Severity) -> ShaclSeverity {
        use shacl::types::Severity as Sh;
        match severity {
            Sh::Warning => ShaclSeverity::Warning,
            Sh::Info | Sh::Debug | Sh::Trace => ShaclSeverity::Info,
            Sh::Violation | Sh::Generic(_) => ShaclSeverity::Violation,
        }
    }

    /// A sentence for the usual SHACL Core components; rudof's own text for the rest.
    fn message(
        constraint: &str,
        severity: ShaclSeverity,
        path: Option<&str>,
        value: Option<&str>,
        generic: &str,
        prefixes: &[(&str, &str)],
    ) -> String {
        let compact = |iri: &str| Self::compact(iri, prefixes);
        let property = path.map(compact).unwrap_or_else(|| "the node".to_string());
        let value = value.map(compact).unwrap_or_default();
        let number = Self::number_in(generic);
        match constraint {
            "MinCount" if severity == ShaclSeverity::Violation => match number {
                Some(n) if n > 1 => format!("{property} needs at least {n} values"),
                _ => format!("{property} is mandatory"),
            },
            "MinCount" => format!("{property} is recommended"),
            "MaxCount" => format!("{property} takes at most {} value(s)", number.unwrap_or(1)),
            "Class" => format!(
                "{value} under {property} should be a {}",
                compact(generic.rsplit(' ').next().unwrap_or_default())
            ),
            "NodeKind" => format!(
                "{property} must be {}",
                match generic.rsplit(": ").next().unwrap_or_default() {
                    "Iri" => "an IRI",
                    "Literal" => "a literal",
                    "BlankNode" => "a blank node",
                    "BlankNodeOrIri" => "an IRI or a blank node",
                    other => other,
                }
            ),
            "Datatype" => format!("{value} under {property} has the wrong datatype"),
            "Node" => format!("{value} under {property} does not match the shape it must"),
            "Pattern" | "In" | "HasValue" => format!("{value} under {property}: {generic}"),
            _ => format!("{property}: {generic}"),
        }
    }

    /// The first number in rudof's text, e.g. `1` in `MinCount(1) not satisfied`.
    fn number_in(text: &str) -> Option<usize> {
        let start = text.find('(')? + 1;
        let end = start + text[start..].find(')')?;
        text[start..end].parse().ok()
    }

    fn path_text(path: &SHACLPath) -> String {
        match path {
            SHACLPath::Predicate { pred } => pred.as_str().to_string(),
            other => other.to_string(),
        }
    }

    fn term_text(term: &Object) -> String {
        match term {
            Object::Iri(iri) => iri.as_str().to_string(),
            other => other.to_string(),
        }
    }

    /// `dct:title` rather than the whole IRI; an IRI no prefix covers stays whole.
    pub fn compact(iri: &str, prefixes: &[(&str, &str)]) -> String {
        prefixes
            .iter()
            .filter(|(_, ns)| iri.starts_with(ns))
            .max_by_key(|(_, ns)| ns.len())
            .map(|(prefix, ns)| format!("{prefix}:{}", &iri[ns.len()..]))
            .unwrap_or_else(|| iri.to_string())
    }

    fn local_name(iri: &str) -> String {
        let name = iri.rsplit(['#', '/']).next().unwrap_or(iri);
        name.strip_suffix(Self::COMPONENT_SUFFIX)
            .unwrap_or(name)
            .to_string()
    }
}
