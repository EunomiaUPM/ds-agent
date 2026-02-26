use std::collections::HashMap;
use crate::entities::common::parameters::{
    TemplateBoolean, TemplateInt, TemplateMapString,
};
use crate::entities::inner::parameters::TemplateField;
use crate::TemplateVecString;
use regex::Regex;
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
/// [`found_parameters`]: ParameterExtractor::found_parameters
pub struct ParameterExtractor {
    found_parameters: Vec<String>,
}

impl ParameterExtractor {
    pub fn new() -> Self {
        Self { found_parameters: Vec::new() }
    }

    /// Returns the ordered list of parameter names found so far.
    /// Names may be duplicated if the same placeholder appears in multiple fields;
    /// deduplication is the responsibility of the downstream [`ParameterValidator`].
    pub fn found_parameters(&self) -> &[String] {
        &self.found_parameters
    }
}

/// Defines how a single [`TemplateField`] token is processed by an extractor.
///
/// The visitor calls this method for every leaf field it encounters. Implementors
/// accumulate or otherwise handle the discovered parameter names.
pub trait ParameterExtractorBehavior {
    /// Scan `template` for `{{__NAME__}}` placeholders and record any names found.
    fn extract(&mut self, template: TemplateField );
}

impl ParameterExtractorBehavior for ParameterExtractor {
    fn extract(&mut self, template: TemplateField) {
        match template {
            TemplateField::TemplateString(t) => {
                self.partial_value_extractor(TemplateField::TemplateString(t))
            }
            TemplateField::TemplateInt(t) => match t {
                TemplateInt::Value(_) => {}
                TemplateInt::Template(value) => self.complete_value_extractor(value.as_str()),
            },
            TemplateField::TemplateBoolean(t) => match t {
                TemplateBoolean::Value(_) => {}
                TemplateBoolean::Template(value) => self.complete_value_extractor(value.as_str()),
            },
            TemplateField::TemplateVecString(t) => match t {
                TemplateVecString::Value(value) => {
                    self.partial_value_extractor(TemplateField::TemplateVecString(t))
                }
                TemplateVecString::Template(value) => self.complete_value_extractor(value.as_str()),
            },
            TemplateField::TemplateMapString(t) => match t {
                TemplateMapString::Value(value) => {
                    self.partial_value_extractor(TemplateField::TemplateMapString(t))
                }
                TemplateMapString::Template(value) => self.complete_value_extractor(value.as_str()),
            },
        }
    }
}

impl ParameterExtractor {
    /// Scans a plain `&str` for embedded `{{__NAME__}}` placeholders.
    /// This is the single primitive all other extractors delegate to.
    fn scan_str(&mut self, value: &str) {
        let re = template_regex();
        for cap in re.captures_iter(value) {
            self.found_parameters.push(cap[1].to_string());
        }
    }

    /// Delegates to [`scan_str`]. Used for `Template` variants of typed fields
    /// such as `TemplateInt::Template` and `TemplateBoolean::Template`, where
    /// the entire string value is a template expression.
    ///
    /// [`scan_str`]: Self::scan_str
    fn complete_value_extractor(&mut self, value: &str) {
        self.scan_str(value);
    }

    /// Dispatches to the type-specific extractor for composite field types
    /// (`TemplateString`, `TemplateVecString`, `TemplateMapString`) whose
    /// `Value` variant may contain interpolated placeholders.
    fn partial_value_extractor(&mut self, template: TemplateField) {
        match template {
            TemplateField::TemplateString(t) => self.scan_str(t),
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
            _ => {}
        }
    }

    /// Scans each element of a string slice for embedded placeholders.
    fn vec_string_parameter_extractor(&mut self, template: &[String]) {
        for s in template {
            self.scan_str(s);
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
            self.scan_str(extra);
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use crate::entities::common::parameters::{TemplateBoolean, TemplateInt, TemplateMapString};
    use crate::entities::inner::parameters::extractor::{
        ParameterExtractor, ParameterExtractorBehavior,
    };
    use crate::entities::inner::parameters::TemplateField;
    use crate::TemplateVecString;

    #[test]
    fn complete_value_extractor_on_int() {
        let mut extractor = ParameterExtractor::new();
        let field = TemplateInt::Template("{{__TIMEOUT__}}".to_string());
        extractor.extract(TemplateField::TemplateInt(&field));
        let found = extractor.found_parameters();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], "TIMEOUT");
    }
    #[test]
    fn complete_value_extractor_on_boolean() {
        let mut extractor = ParameterExtractor::new();
        let field = TemplateBoolean::Template("{{__TEST__}}".to_string());
        extractor.extract(TemplateField::TemplateBoolean(&field));
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 1);
        assert_eq!(found_parameters[0], "TEST");
    }
    #[test]
    fn complete_value_extractor_on_string() {
        let mut extractor = ParameterExtractor::new();
        let field: String = "{{__TEST__}}".to_string();
        extractor.extract(TemplateField::TemplateString(&field));
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 1);
        assert_eq!(found_parameters[0], "TEST");
    }
    #[test]
    fn partial_value_extractor_on_string() {
        let mut extractor = ParameterExtractor::new();
        let field: String = "http://mi-api.com/{{__TEST__}}".to_string();
        extractor.extract(TemplateField::TemplateString(&field));
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 1);
        assert_eq!(found_parameters[0], "TEST");
    }
    #[test]
    fn nothing_to_extract_on_string() {
        let mut extractor = ParameterExtractor::new();
        let field: String = "http://mi-api.com/no-test".to_string();
        extractor.extract(TemplateField::TemplateString(&field));
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 0);
    }
    #[test]
    fn vec_string_parameter_extractor_as_complete() {
        let mut extractor = ParameterExtractor::new();
        let field = TemplateVecString::Template("{{__TEST__}}".to_string());
        extractor.extract(TemplateField::TemplateVecString(&field));
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 1);
        assert_eq!(found_parameters[0], "TEST");
    }
    #[test]
    fn vec_string_parameter_extractor_as_partial() {
        let mut extractor = ParameterExtractor::new();
        let field = TemplateVecString::Value(vec!["test".to_string(), "{{__TEST__}}".to_string()]);
        extractor.extract(TemplateField::TemplateVecString(&field));
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 1);
        assert_eq!(found_parameters[0], "TEST");
    }
    #[test]
    fn nothing_to_extract_on_vec_string() {
        let mut extractor = ParameterExtractor::new();
        let field = TemplateVecString::Value(vec!["test".to_string(), "test".to_string()]);
        extractor.extract(TemplateField::TemplateVecString(&field));
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 0);
    }
    #[test]
    fn map_string_parameter_extractor_as_complete() {
        let mut extractor = ParameterExtractor::new();
        let field = TemplateMapString::Template("{{__TEST__}}".to_string());
        extractor.extract(TemplateField::TemplateMapString(&field));
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 1);
        assert_eq!(found_parameters[0], "TEST");
    }
    #[test]
    fn map_string_parameter_extractor_as_partial() {
        let mut extractor = ParameterExtractor::new();
        let field = TemplateMapString::Value(
            HashMap::from([
                ("__EXTRA__".to_string(), "{{__TEST__}}".to_string())
            ]),
        );
        extractor.extract(TemplateField::TemplateMapString(&field));
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 1);
        assert_eq!(found_parameters[0], "TEST");
    }
    #[test]
    fn nothing_to_extract_on_map_string() {
        let mut extractor = ParameterExtractor::new();
        let field = TemplateMapString::Value(
            HashMap::from([
                ("test".to_string(), "test".to_string())
            ]),
        );
        extractor.extract(TemplateField::TemplateMapString(&field));
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 0);
    }
}
