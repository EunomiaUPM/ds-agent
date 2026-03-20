use crate::entities::parameters::template_parameters_extractor::ParameterExtractorBehavior;
use crate::entities::parameters::TemplateField;
use crate::entities::parameters::{SysParameterType, TemplateMapString, TemplateVecString};
use regex::Regex;
use std::sync::OnceLock;

fn template_sys_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\{\{\s*__(SYS.*?)__\s*\}\}").expect("Invalid Regex"))
}

pub struct SystemParameterExtractor {
    found_parameters: Vec<FoundSysParameter>,
}

#[derive(Clone)]
pub struct FoundSysParameter {
    pub name: String,
    pub content_type: SysParameterType,
}

impl SystemParameterExtractor {
    pub fn new() -> Self {
        Self { found_parameters: Vec::new() }
    }

    pub fn found_sys_parameters(&self) -> &[FoundSysParameter] {
        &self.found_parameters
    }

    fn scan_str(&mut self, value: &str) {
        let re = template_sys_regex();
        for cap in re.captures_iter(value) {
            let name = cap[1].to_string();
            if let Some(content_type) = self.sys_parameter_type_from_name(&name) {
                self.found_parameters.push(FoundSysParameter { name, content_type });
            }
        }
    }

    fn sys_parameter_type_from_name(&self, name: &str) -> Option<SysParameterType> {
        match name {
            "SYS_URN" => Some(SysParameterType::SysUrn),
            "SYS_TOKEN" => Some(SysParameterType::SysToken),
            "SYS_TIMESTAMP" => Some(SysParameterType::SysTimestamp),
            "SYS_ISO8601" => Some(SysParameterType::SysIso8601),
            "SYS_OWN_URL" => Some(SysParameterType::SysOwnUrl { host_docker_internal: false }),
            "SYS_OWN_URL_DOCKER" => {
                Some(SysParameterType::SysOwnUrl { host_docker_internal: true })
            }
            _ => None,
        }
    }
}

/// Only `TemplateInt` with `SYS_TIMESTAMP` makes sense for numeric fields.
/// Booleans have no meaningful sys parameter — silently skipped.
/// Strings (plain, vec items, map `__EXTRA__` value) can carry any sys placeholder.
impl ParameterExtractorBehavior for SystemParameterExtractor {
    fn extract(&mut self, template: TemplateField) {
        match template {
            // Plain string or interpolated string — scan always
            TemplateField::TemplateString(t) => self.scan_str(t),

            // Vec items are strings — scan each element or the whole template string
            TemplateField::TemplateVecString(t) => match t {
                TemplateVecString::Value(vec) => {
                    for s in vec {
                        self.scan_str(s);
                    }
                }
                TemplateVecString::Template(value) => self.scan_str(value.as_str()),
            },

            // Map: only the __EXTRA__ value is dynamic; keys are static header names
            TemplateField::TemplateMapString(t) => match t {
                TemplateMapString::Value(map) => {
                    if let Some(extra) = map.get("__EXTRA__") {
                        self.scan_str(extra);
                    }
                }
                TemplateMapString::Template(value) => self.scan_str(value.as_str()),
            },
        }
    }
}

#[cfg(test)]
#[cfg(test)]
mod tests;
