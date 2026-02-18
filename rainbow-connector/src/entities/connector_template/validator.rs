use crate::entities::common::parameter_visitor::{ExpectedType, ParameterVisitor};
use crate::entities::common::parameters::{ParameterDefinition, ParameterType};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

fn template_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\{\{__(.*?)__\}\}").expect("Invalid Regex"))
}

pub struct TemplateValidator<'a> {
    param_definitions: HashMap<&'a str, ParameterType>,
    pub errors: Vec<String>,
    context_stack: Vec<String>,
}

impl<'a> TemplateValidator<'a> {
    pub fn new(definitions: &'a [ParameterDefinition]) -> Self {
        let mut map = HashMap::new();
        for p in definitions {
            map.insert(p.name.as_str(), p.param_type.clone());
        }
        Self { param_definitions: map, errors: Vec::new(), context_stack: Vec::new() }
    }

    fn current_path(&self) -> String {
        self.context_stack.join(".")
    }

    fn validate_compatibility(
        &mut self,
        param_name: &str,
        defined: ParameterType,
        expected: ExpectedType,
    ) {
        let is_compatible = match (expected, &defined) {
            (ExpectedType::AnyString, _) => true,
            (ExpectedType::StrictInt, ParameterType::Int) => true,
            (ExpectedType::StrictBool, ParameterType::Boolean) => true,
            (ExpectedType::StrictVec, ParameterType::VecString) => true,
            (ExpectedType::StrictMap, ParameterType::MapStringString) => true,
            _ => false,
        };

        if !is_compatible {
            self.errors.push(format!(
                "TYPE ERROR in '{}': {:?} was expected, but parameter '{}' is of type {:?}.",
                self.current_path(),
                expected,
                param_name,
                defined
            ));
        }
    }
}

impl<'a> ParameterVisitor for TemplateValidator<'a> {
    type Error = anyhow::Error;
    fn enter_scope(&mut self, name: &str) {
        self.context_stack.push(name.to_string());
    }
    fn exit_scope(&mut self) {
        self.context_stack.pop();
    }
    fn scan_template_candidate(&mut self, value: &str, expected: ExpectedType) {
        let re = template_regex();
        for cap in re.captures_iter(value) {
            let param_name = &cap[1]; // capture param NAME

            // RUNTIME_* parameters are resolved at runtime (not declared in the parameters object)
            if param_name.starts_with("RUNTIME_") || param_name.starts_with("SYS_") {
                continue;
            }
            match self.param_definitions.get(param_name) {
                Some(defined_type) => {
                    self.validate_compatibility(param_name, defined_type.clone(), expected);
                }
                None => {
                    self.errors.push(format!(
                        "UNDEFINED ERROR en '{}': parameter '{}' is not declared in parameters object.",
                        self.current_path(),
                        param_name
                    ));
                }
            }
        }
    }
}
