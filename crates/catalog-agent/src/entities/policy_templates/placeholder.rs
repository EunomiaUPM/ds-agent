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

//! Template placeholders: a JSON string `"$name"` stands for a parameter, `"$$..."` for a literal `$`.

use std::collections::HashMap;

use serde_json::Value;

/// Reads, collects and fills the placeholders of template content.
pub struct Placeholder;

impl Placeholder {
    /// The parameter a string names, `$` included; none for plain or escaped (`$$`) strings.
    pub fn name(text: &str) -> Option<&str> {
        (text.starts_with('$') && !text.starts_with("$$")).then_some(text)
    }

    /// Every placeholder in the value, in document order and with repeats.
    pub fn collect(value: &Value) -> Vec<String> {
        let mut found = Vec::new();
        Self::walk(value, &mut found);
        found
    }

    /// Replaces each placeholder with its value and unescapes `$$`; unknown ones stay as they are.
    pub fn fill(value: &mut Value, values: &HashMap<String, Value>) {
        match value {
            Value::String(text) => {
                if let Some(replacement) = Self::name(text).and_then(|name| values.get(name)) {
                    *value = replacement.clone();
                } else if let Some(literal) = text.strip_prefix('$').filter(|t| t.starts_with('$')) {
                    *value = Value::String(literal.to_string());
                }
            }
            Value::Array(items) => items.iter_mut().for_each(|item| Self::fill(item, values)),
            Value::Object(map) => map.values_mut().for_each(|item| Self::fill(item, values)),
            _ => {}
        }
    }

    fn walk(value: &Value, found: &mut Vec<String>) {
        match value {
            Value::String(text) => found.extend(Self::name(text).map(str::to_string)),
            Value::Array(items) => items.iter().for_each(|item| Self::walk(item, found)),
            Value::Object(map) => map.values().for_each(|item| Self::walk(item, found)),
            _ => {}
        }
    }
}
