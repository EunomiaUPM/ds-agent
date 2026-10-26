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

//! A term table: prefixes, class terms and property terms with their coercions. It is the one
//! source of a context and of the compaction that writes with it.

use serde_json::{json, Map, Value};

/// How a term reads its plain JSON values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coercion {
    /// A string is a plain literal.
    None,
    /// A string is an IRI.
    Id,
    /// A string is an IRI, relative to the vocabulary.
    Vocab,
    /// A map from language to strings.
    Language,
    /// A string is a literal of this datatype.
    Typed(&'static str),
    /// The value is a JSON literal.
    Json,
}

/// One property term.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermDef {
    pub term: &'static str,
    pub iri: &'static str,
    pub coercion: Coercion,
    /// Always written as an array.
    pub set: bool,
}

impl TermDef {
    pub const fn new(term: &'static str, iri: &'static str, coercion: Coercion, set: bool) -> Self {
        Self {
            term,
            iri,
            coercion,
            set,
        }
    }

    /// The term's definition in a context.
    fn definition(&self) -> Value {
        let mut definition = Map::new();
        definition.insert("@id".into(), json!(self.iri));
        match self.coercion {
            Coercion::None => {}
            Coercion::Id => {
                definition.insert("@type".into(), json!("@id"));
            }
            Coercion::Vocab => {
                definition.insert("@type".into(), json!("@vocab"));
            }
            Coercion::Language => {
                definition.insert("@container".into(), json!("@language"));
            }
            Coercion::Typed(datatype) => {
                definition.insert("@type".into(), json!(datatype));
            }
            Coercion::Json => {
                definition.insert("@type".into(), json!("@json"));
            }
        }
        if self.set && self.coercion != Coercion::Language {
            definition.insert("@container".into(), json!("@set"));
        }
        if definition.len() == 1 {
            json!(self.iri)
        } else {
            Value::Object(definition)
        }
    }
}

/// Prefixes, class terms and property terms of one vocabulary of terms.
#[derive(Debug, Clone, Default)]
pub struct TermTable {
    pub prefixes: Vec<(&'static str, &'static str)>,
    pub classes: Vec<(&'static str, &'static str)>,
    pub terms: Vec<TermDef>,
}

impl TermTable {
    pub fn new(
        prefixes: &[(&'static str, &'static str)],
        classes: &[(&'static str, &'static str)],
        terms: &[TermDef],
    ) -> Self {
        Self {
            prefixes: prefixes.to_vec(),
            classes: classes.to_vec(),
            terms: terms.to_vec(),
        }
    }

    /// The term defined for `iri`, if any.
    pub fn term_for(&self, iri: &str) -> Option<&TermDef> {
        self.terms.iter().find(|t| t.iri == iri)
    }

    /// The class term of `iri`, else its compact IRI.
    pub fn class_term(&self, iri: &str) -> String {
        self.classes
            .iter()
            .find(|(_, class)| *class == iri)
            .map(|(term, _)| term.to_string())
            .unwrap_or_else(|| self.compact_iri(iri))
    }

    /// `iri` as `prefix:local` when one of the prefixes covers it, else whole.
    pub fn compact_iri(&self, iri: &str) -> String {
        self.prefixes
            .iter()
            .filter(|(_, ns)| iri.starts_with(ns) && iri.len() > ns.len())
            .max_by_key(|(_, ns)| ns.len())
            .map(|(prefix, ns)| format!("{prefix}:{}", &iri[ns.len()..]))
            .unwrap_or_else(|| iri.to_string())
    }

    /// The JSON-LD context these terms make, protected like the DSP one.
    pub fn context(&self) -> Value {
        let mut context = Map::new();
        context.insert("@version".into(), json!(1.1));
        context.insert("@protected".into(), json!(true));
        for (prefix, ns) in &self.prefixes {
            context.insert(prefix.to_string(), json!(ns));
        }
        for (term, class) in &self.classes {
            context.insert(term.to_string(), json!(class));
        }
        for term in &self.terms {
            context.insert(term.term.to_string(), term.definition());
        }
        json!({ "@context": Value::Object(context) })
    }
}
