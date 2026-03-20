use crate::entities::parameters::{
    FoundParameterType, TemplateField, TemplateMapString, TemplateVecString,
};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Returns the lazily-compiled regex that matches `{{__NAME__}}` placeholders.
/// The inner capture group yields the bare parameter name (e.g. `"HOST"`).
fn template_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\{\{\s*__(.*?)__\s*\}\}").expect("Invalid Regex"))
}

/// Accumulates all `{{__NAME__}}` placeholder names found across a series of
/// template fields.
///
/// Instantiate once per validation pass, call [`ParameterExtractorBehavior::extract`]
/// for every field of interest, then read the results via [`found_parameters`].
///
/// [`found_parameters`]: TemplateParameterExtractor::found_parameters
pub struct TemplateParameterExtractor {
    found_parameters: Vec<FoundParameter>,
}

#[derive(Clone)]
pub struct FoundParameter {
    pub name: String,
    pub content_type: FoundParameterType,
}

impl TemplateParameterExtractor {
    pub fn new() -> Self {
        Self { found_parameters: Vec::new() }
    }

    /// Returns the ordered list of parameter names found so far.
    /// Names may be duplicated if the same placeholder appears in multiple fields;
    /// deduplication is the responsibility of the downstream [`ParameterValidator`].
    pub fn found_parameters(&self) -> &[FoundParameter] {
        &self.found_parameters
    }
}

/// Defines how a single [`TemplateField`] token is processed by an extractor.
///
/// The visitor calls this method for every leaf field it encounters. Implementors
/// accumulate or otherwise handle the discovered parameter names.
pub trait ParameterExtractorBehavior {
    /// Scan `template` for `{{__NAME__}}` placeholders and record any names found.
    fn extract(&mut self, template: TemplateField);
}

impl ParameterExtractorBehavior for TemplateParameterExtractor {
    fn extract(&mut self, template: TemplateField) {
        match template {
            TemplateField::TemplateString(t) => {
                self.partial_value_extractor(TemplateField::TemplateString(t))
            }
            TemplateField::TemplateVecString(t) => match t {
                TemplateVecString::Value(_) => {
                    self.partial_value_extractor(TemplateField::TemplateVecString(t))
                }
                TemplateVecString::Template(value) => {
                    self.complete_value_extractor(value.as_str(), &FoundParameterType::VecString)
                }
            },
            TemplateField::TemplateMapString(t) => match t {
                TemplateMapString::Value(_) => {
                    self.partial_value_extractor(TemplateField::TemplateMapString(t))
                }
                TemplateMapString::Template(value) => {
                    self.complete_value_extractor(value.as_str(), &FoundParameterType::MapString)
                }
            },
        }
    }
}

impl TemplateParameterExtractor {
    /// Scans a plain `&str` for embedded `{{__NAME__}}` placeholders.
    /// This is the single primitive all other extractors delegate to.
    fn scan_str(&mut self, value: &str, content_type: &FoundParameterType) {
        let re = template_regex();
        for cap in re.captures_iter(value) {
            self.found_parameters
                .push(FoundParameter { name: cap[1].to_string(), content_type: *content_type });
        }
    }

    /// Delegates to [`scan_str`]. Used for `Template` variants of typed fields
    /// such as `TemplateInt::Template` and `TemplateBoolean::Template`, where
    /// the entire string value is a template expression.
    ///
    /// [`scan_str`]: Self::scan_str
    fn complete_value_extractor(&mut self, value: &str, content_type: &FoundParameterType) {
        self.scan_str(value, content_type);
    }

    /// Dispatches to the type-specific extractor for composite field types
    /// (`TemplateString`, `TemplateVecString`, `TemplateMapString`) whose
    /// `Value` variant may contain interpolated placeholders.
    fn partial_value_extractor(&mut self, template: TemplateField) {
        match template {
            TemplateField::TemplateString(t) => self.scan_str(t, &FoundParameterType::String),
            TemplateField::TemplateVecString(t) => {
                if let TemplateVecString::Value(vec) = t {
                    self.vec_string_parameter_extractor(vec)
                }
            }
            TemplateField::TemplateMapString(t) => {
                if let TemplateMapString::Value(map) = t {
                    self.map_string_parameter_extractor(map)
                }
            }
        }
    }

    /// Scans each element of a string slice for embedded placeholders.
    fn vec_string_parameter_extractor(&mut self, template: &[String]) {
        for s in template {
            self.scan_str(s, &FoundParameterType::VecString);
        }
    }

    /// Scans the value of the special `"__EXTRA__"` key in a map for placeholders.
    ///
    /// Only `__EXTRA__` is inspected because regular map keys are static header
    /// names set by the connector author, not runtime parameters. `__EXTRA__` is
    /// the conventional escape hatch for dynamic header values that depend on a
    /// parameter (e.g. `"Bearer {{__TOKEN__}}"`).
    fn map_string_parameter_extractor(&mut self, template: &HashMap<String, String>) {
        if let Some(extra) = template.get("__EXTRA__") {
            self.scan_str(extra, &FoundParameterType::MapString);
        }
    }
}

#[cfg(test)]
mod tests;
