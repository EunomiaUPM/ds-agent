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

//! Parameter definitions and localized texts.

use crate::entities::policy_templates::ParameterValue;
use serde::{Deserialize, Serialize};

/// Text in one language.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LocalizedString {
    #[serde(rename = "@value")]
    pub value: String,
    #[serde(rename = "@language")]
    pub language: String,
}

/// Text as a plain string or in several languages.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum LocalizedText {
    Single(String),
    Multiple(Vec<LocalizedString>),
}

/// Value of a selection parameter with its label.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectionOption {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<LocalizedText>,
}

/// Allowed values of a selection: plain strings or labelled options.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum SelectionAllowedValues {
    Simple(Vec<String>),
    Complex(Vec<SelectionOption>),
}

/// Type of a template parameter.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ParameterDataType {
    String,
    Integer,
    Float,
    Boolean,
    /// `YYYY-MM-DD`.
    Date,
    /// `HH:MM` or `HH:MM:SS`.
    Time,
    /// RFC 3339; `datetime` is accepted too.
    #[serde(alias = "datetime")]
    DateTime,
    /// One of `restrictions.values`.
    Selection,
    /// An xsd:duration such as `P30D` or `PT12H`.
    Duration,
    /// An absolute or compact IRI such as `gx:LegalParticipant`.
    Iri,
    /// A list of strings, for operators such as `isAnyOf`.
    List,
}

/// Limits a parameter value must respect, depending on its type.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ValidationRestrictions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_date: Option<String>,

    /// Allowed values of a selection, or of every item of a list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<SelectionAllowedValues>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
}

/// How the admin UI shows a parameter.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiHints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<LocalizedText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<LocalizedText>,
    #[serde(default)]
    pub hidden: bool,
}

/// One template parameter: type, restrictions, UI hints and default.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParameterDefinition {
    #[serde(rename = "dataType")]
    pub data_type: ParameterDataType,
    #[serde(default)]
    pub restrictions: ValidationRestrictions,
    #[serde(default)]
    pub ui: UiHints,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<ParameterValue>,
}
