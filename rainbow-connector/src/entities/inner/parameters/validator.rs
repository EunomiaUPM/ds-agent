use crate::entities::common::parameters::ParameterDefinition;
use std::collections::HashSet;

/// Validates that the set of parameter names found in a connector template
/// matches the set of [`ParameterDefinition`]s declared in `parameters[]`.
///
/// Two kinds of mismatches are detected and reported together:
///
/// - **Undeclared** — a placeholder `{{__NAME__}}` appears in the template but
///   no `ParameterDefinition` with that `name` exists.
/// - **Unused** — a `ParameterDefinition` is declared but its placeholder never
///   appears in the template.
///
/// Duplicate occurrences in `parameters_found` are collapsed before comparison
/// so that using the same placeholder in multiple fields does not produce false
/// positives.
pub struct ParameterValidator<'a> {
    parameters: &'a [ParameterDefinition],
    /// When `true`, names that start with `RUNTIME_` are silently ignored during
    /// validation. These names are injected by the runtime engine at execution
    /// time and are not expected to appear in `parameters[]`.
    exclude_runtime: bool,
}

impl<'a> ParameterValidator<'a> {
    pub fn new(parameters: &'a [ParameterDefinition], exclude_runtime: bool) -> Self {
        Self { parameters, exclude_runtime }
    }

    /// Validates `parameters_found` against the declared definitions.
    ///
    /// Returns `Ok(())` when the two sets are equal (after deduplication and
    /// optional `RUNTIME_*` exclusion). Returns `Err` with a human-readable
    /// description listing both undeclared and unused names when mismatches exist.
    pub fn validate(&self, parameters_found: &[String]) -> anyhow::Result<()> {
        // Deduplicate found names so that repeated placeholders count once.
        // When exclude_runtime is set, RUNTIME_* names are silently skipped — they
        // are resolved at runtime and are not expected to appear in parameters[].
        let found_set: HashSet<&str> = parameters_found
            .iter()
            .map(|s| s.as_str())
            .filter(|s| !self.exclude_runtime || !s.starts_with("RUNTIME_"))
            .filter(|s| !self.exclude_runtime || !s.starts_with("SYS_"))
            .collect();
        let defined_set: HashSet<&str> = self.parameters.iter().map(|p| p.name.as_str()).collect();

        let mut errors: Vec<String> = Vec::new();

        // Used in the template but not declared in parameters[].
        let mut undeclared: Vec<&str> = found_set.difference(&defined_set).copied().collect();
        if !undeclared.is_empty() {
            undeclared.sort();
            errors.push(format!(
                "Undeclared parameters found in template: [{}]",
                undeclared.join(", ")
            ));
        }

        // Declared in parameters[] but never used in the template.
        let mut unused: Vec<&str> = defined_set.difference(&found_set).copied().collect();
        if !unused.is_empty() {
            unused.sort();
            errors.push(format!(
                "Declared parameters not found in template: [{}]",
                unused.join(", ")
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(errors.join("; ")))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::entities::common::parameters::{
        AutoFillable, ParameterDefinition, ParameterType,
    };
    use crate::entities::inner::parameters::validator::ParameterValidator;

    fn make_def(name: &str) -> ParameterDefinition {
        ParameterDefinition {
            name: name.to_string(),
            title: name.to_string(),
            description: None,
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            auto_fillable: AutoFillable::default(),
        }
    }

    #[test]
    fn ok_when_found_and_defined_match_exactly() {
        let defs = vec![make_def("HOST"), make_def("PORT")];
        let found = vec!["HOST".to_string(), "PORT".to_string()];
        assert!(ParameterValidator::new(&defs, false).validate(&found).is_ok());
    }

    #[test]
    fn ok_when_both_empty() {
        let defs: Vec<ParameterDefinition> = vec![];
        let found: Vec<String> = vec![];
        assert!(ParameterValidator::new(&defs, false).validate(&found).is_ok());
    }

    #[test]
    fn ok_when_duplicate_found_entries_deduplicated() {
        // The same placeholder may appear more than once in a template; it counts once.
        let defs = vec![make_def("HOST")];
        let found = vec!["HOST".to_string(), "HOST".to_string()];
        assert!(ParameterValidator::new(&defs, false).validate(&found).is_ok());
    }

    #[test]
    fn err_when_found_parameter_is_not_declared() {
        let defs = vec![make_def("HOST")];
        let found = vec!["HOST".to_string(), "UNKNOWN".to_string()];
        let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Undeclared"), "expected undeclared error, got: {msg}");
        assert!(msg.contains("UNKNOWN"));
    }

    #[test]
    fn err_when_declared_parameter_is_not_used_in_template() {
        let defs = vec![make_def("HOST"), make_def("UNUSED")];
        let found = vec!["HOST".to_string()];
        let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Declared parameters not found"), "expected unused error, got: {msg}");
        assert!(msg.contains("UNUSED"));
    }

    #[test]
    fn err_contains_both_issues_when_undeclared_and_unused_exist() {
        let defs = vec![make_def("HOST"), make_def("UNUSED")];
        let found = vec!["HOST".to_string(), "GHOST".to_string()];
        let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("GHOST"), "expected GHOST in error: {msg}");
        assert!(msg.contains("UNUSED"), "expected UNUSED in error: {msg}");
    }

    #[test]
    fn err_lists_multiple_undeclared_parameters_sorted() {
        let defs: Vec<ParameterDefinition> = vec![];
        let found = vec!["ZEBRA".to_string(), "ALPHA".to_string()];
        let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
        let msg = err.to_string();
        // sorted: ALPHA before ZEBRA
        let alpha_pos = msg.find("ALPHA").unwrap();
        let zebra_pos = msg.find("ZEBRA").unwrap();
        assert!(alpha_pos < zebra_pos);
    }

    // =========================================================================
    // exclude_runtime = true
    // =========================================================================

    #[test]
    fn ok_when_runtime_parameter_is_excluded() {
        // RUNTIME_* names are silently skipped and do not need a ParameterDefinition.
        let defs = vec![make_def("HOST")];
        let found = vec!["HOST".to_string(), "RUNTIME_TIMESTAMP".to_string()];
        assert!(ParameterValidator::new(&defs, true).validate(&found).is_ok());
    }

    #[test]
    fn ok_when_only_runtime_parameters_found_and_no_definitions() {
        // With no user-defined parameters, only RUNTIME_* in the template is valid.
        let defs: Vec<ParameterDefinition> = vec![];
        let found = vec!["RUNTIME_URN".to_string(), "RUNTIME_TOKEN".to_string()];
        assert!(ParameterValidator::new(&defs, true).validate(&found).is_ok());
    }

    #[test]
    fn err_when_non_runtime_undeclared_alongside_runtime_excluded() {
        // RUNTIME_* is ignored, but GHOST is still an undeclared error.
        let defs = vec![make_def("HOST")];
        let found = vec!["HOST".to_string(), "RUNTIME_TIMESTAMP".to_string(), "GHOST".to_string()];
        let err = ParameterValidator::new(&defs, true).validate(&found).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("GHOST"), "expected GHOST in error: {msg}");
        assert!(!msg.contains("RUNTIME_TIMESTAMP"), "RUNTIME_ should be excluded: {msg}");
    }

    #[test]
    fn runtime_parameter_causes_error_when_exclude_runtime_is_false() {
        // With exclude_runtime off, RUNTIME_* is treated as any other undeclared name.
        let defs: Vec<ParameterDefinition> = vec![];
        let found = vec!["RUNTIME_TIMESTAMP".to_string()];
        let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
        assert!(err.to_string().contains("RUNTIME_TIMESTAMP"));
    }
}
