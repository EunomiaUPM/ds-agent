use crate::entities::connector_template::ConnectorTemplateDto;
use crate::entities::inner::parameters::extractor::ParameterExtractor;
use crate::entities::inner::parameters::validator::ParameterValidator;
use crate::entities::inner::parameters::visitor::ParameterExtractorVisitor;
use anyhow::Context;

/// Validates a new connector template and, once complete, persists and returns it.
///
/// # Pipeline
/// 1. **Extract** — [`ParameterExtractorVisitor`] walks the entire DTO and
///    collects every `{{__NAME__}}` placeholder via [`ParameterExtractor`].
/// 2. **Validate** — [`ParameterValidator`] checks that the found names match
///    the connector's declared `parameters[]` definitions. `RUNTIME_*` names are
///    excluded from validation as they are resolved by the runtime engine.
/// 3. **Persist** — *(not yet implemented)* — the validated template is stored.
///
/// # Errors
/// Returns an error if the parameter sets do not match (undeclared or unused
/// parameters).
///
/// # Note
/// The return value is currently a placeholder stub; the real implementation
/// should return the persisted template retrieved from the repository.
pub fn create_instance(new_template: &ConnectorTemplateDto) -> anyhow::Result<ConnectorTemplateDto> {
    // 1. Extract all {{__NAME__}} placeholders from the template.
    let mut extractor = ParameterExtractor::new();
    let visitor = ParameterExtractorVisitor::new(new_template);
    visitor.extract(&mut extractor);

    // 2. Validate found names against the declared parameter definitions.
    //    RUNTIME_* names are produced by the engine and must not be declared.
    let validator = ParameterValidator::new(&new_template.parameters, true);
    validator
        .validate(extractor.found_parameters())
        .context("Error validating connector template")?;

    // TODO: persist template

    // TODO: return the persisted model instead of this stub
    Ok(new_template.clone())
}

#[cfg(test)]
mod test {
    use crate::entities::connector_template::ConnectorTemplateDto;
    use serde_json::json;

    #[test]
    fn test_create_instance() {
        let dto = json!({
            "authentication": {
                "type": "BASIC_AUTH",
                "username": "asd",
                "password": {
                    "type": "PLAIN",
                    "content": "{{__SYS_RANDSTRING__}}"
                }
            },
            "interaction": {
                "mode": "PULL",
                "dataAccess": {
                    "protocol": "HTTP",
                    "urlTemplate": "http://data-plane/{{__ACCESS_URL__}}",
                    "method": ["GET", "{{__ACCESS_METHODS__}}"],
                    "path": "{{__ACCESS_URL__}}",
                    "headers": {
                        "Content-Type": "application/json",
                        "__EXTRA__": "{{__HEADERS__}}"
                    }
                }
            },
            "parameters": [
                {
                    "paramType": "STRING",
                    "name": "ACCESS_URL",
                    "title": "Access url",
                    "required": true
                },
                {
                    "paramType": "VEC<STRING>",
                    "name": "ACCESS_METHODS",
                    "title": "Access methods",
                    "required": true
                },
                {
                    "paramType": "MAP<STRING,STRING>",
                    "name": "HEADERS",
                    "title": "Url",
                    "required": true
                }
            ]
        });
        let dto: ConnectorTemplateDto = serde_json::from_value(dto).unwrap();
        let result = super::create_instance(&dto);
        assert!(result.is_ok());
    }
    #[test]
    fn test_create_instance_too_much_parameters() {
        // SHOULD_FAIL is declared in parameters[] but never used in the template.
        let dto = json!({
            "authentication": {
                "type": "BASIC_AUTH",
                "username": "asd",
                "password": {
                    "type": "PLAIN",
                    "content": "{{__SYS_RANDSTRING__}}"
                }
            },
            "interaction": {
                "mode": "PULL",
                "dataAccess": {
                    "protocol": "HTTP",
                    "urlTemplate": "http://data-plane/{{__ACCESS_URL__}}",
                    "method": ["GET", "{{__ACCESS_METHODS__}}"],
                    "path": "{{__ACCESS_URL__}}",
                    "headers": {
                        "Content-Type": "application/json",
                        "__EXTRA__": "{{__HEADERS__}}"
                    }
                }
            },
            "parameters": [
                {
                    "paramType": "STRING",
                    "name": "SHOULD_FAIL",
                    "title": "Should fail parameter",
                    "required": true
                },
                {
                    "paramType": "STRING",
                    "name": "ACCESS_URL",
                    "title": "Access url",
                    "required": true
                },
                {
                    "paramType": "VEC<STRING>",
                    "name": "ACCESS_METHODS",
                    "title": "Access methods",
                    "required": true
                },
                {
                    "paramType": "MAP<STRING,STRING>",
                    "name": "HEADERS",
                    "title": "Url",
                    "required": true
                }
            ]
        });
        let dto: ConnectorTemplateDto = serde_json::from_value(dto).unwrap();
        let result = super::create_instance(&dto);
        let err = result.unwrap_err();
        // to_string() gives only the outer context message.
        assert_eq!(err.to_string(), "Error validating connector template");
        // {:#} gives the full chain: "<context>: <cause>".
        let full = format!("{:#}", err);
        assert!(full.contains("Declared parameters not found in template"), "expected unused error, got: {full}");
        assert!(full.contains("SHOULD_FAIL"), "expected SHOULD_FAIL in error, got: {full}");
        // The valid parameters should not appear in the error.
        assert!(!full.contains("ACCESS_URL"), "ACCESS_URL should not be in error: {full}");
        assert!(!full.contains("ACCESS_METHODS"), "ACCESS_METHODS should not be in error: {full}");
        assert!(!full.contains("HEADERS"), "HEADERS should not be in error: {full}");
    }

    #[test]
    fn test_create_instance_too_less_parameters() {
        // SHOULD_FAIL is used in the template but not declared in parameters[].
        let dto = json!({
            "authentication": {
                "type": "BASIC_AUTH",
                "username": "asd",
                "password": {
                    "type": "PLAIN",
                    "content": "{{__SYS_RANDSTRING__}}"
                }
            },
            "interaction": {
                "mode": "PULL",
                "dataAccess": {
                    "protocol": "HTTP",
                    "urlTemplate": "http://data-plane/{{__SHOULD_FAIL__}}",
                    "method": ["GET", "{{__ACCESS_METHODS__}}"],
                    "path": "{{__SHOULD_FAIL__}}",
                    "headers": {
                        "Content-Type": "application/json",
                        "__EXTRA__": "{{__HEADERS__}}"
                    }
                }
            },
            "parameters": [
                {
                    "paramType": "VEC<STRING>",
                    "name": "ACCESS_METHODS",
                    "title": "Access methods",
                    "required": true
                },
                {
                    "paramType": "MAP<STRING,STRING>",
                    "name": "HEADERS",
                    "title": "Url",
                    "required": true
                }
            ]
        });
        let dto: ConnectorTemplateDto = serde_json::from_value(dto).unwrap();
        let result = super::create_instance(&dto);
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "Error validating connector template");
        let full = format!("{:#}", err);
        assert!(full.contains("Undeclared parameters found in template"), "expected undeclared error, got: {full}");
        assert!(full.contains("SHOULD_FAIL"), "expected SHOULD_FAIL in error, got: {full}");
        // The correctly declared parameters should not appear in the error.
        assert!(!full.contains("ACCESS_METHODS"), "ACCESS_METHODS should not be in error: {full}");
        assert!(!full.contains("HEADERS"), "HEADERS should not be in error: {full}");
    }

    #[test]
    fn test_create_instance_runtime_in_parameters() {
        // SYS_* and RUNTIME_* names in the template are excluded from validation —
        // but declaring them explicitly in parameters[] is still an error because
        // the validator sees them as unused (they never appear in found_set).
        // Additionally SHOULD_FAIL is used in the template but not declared, and
        // ACCESS_URL is declared but never used.
        let dto = json!({
            "authentication": {
                "type": "BASIC_AUTH",
                "username": "asd",
                "password": {
                    "type": "PLAIN",
                    "content": "{{__SYS_RANDSTRING__}}"
                }
            },
            "interaction": {
                "mode": "PULL",
                "dataAccess": {
                    "protocol": "HTTP",
                    "urlTemplate": "http://data-plane/{{__SHOULD_FAIL__}}",
                    "method": ["GET", "{{__ACCESS_METHODS__}}"],
                    "path": "{{__SHOULD_FAIL__}}",
                    "headers": {
                        "Content-Type": "application/json",
                        "__EXTRA__": "{{__HEADERS__}}"
                    }
                }
            },
            "parameters": [
                {
                    "paramType": "STRING",
                    "name": "ACCESS_URL",
                    "title": "Access url",
                    "required": true
                },
                {
                    "paramType": "VEC<STRING>",
                    "name": "ACCESS_METHODS",
                    "title": "Access methods",
                    "required": true
                },
                {
                    "paramType": "MAP<STRING,STRING>",
                    "name": "HEADERS",
                    "title": "Url",
                    "required": true
                },
                {
                    "paramType": "STRING",
                    "name": "SYS_RANDSTRING",
                    "title": "Url",
                    "required": true
                }
            ]
        });
        let dto: ConnectorTemplateDto = serde_json::from_value(dto).unwrap();
        let result = super::create_instance(&dto);
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "Error validating connector template");
        let full = format!("{:#}", err);
        // SHOULD_FAIL is in the template but not in parameters[].
        assert!(full.contains("Undeclared parameters found in template"), "expected undeclared error, got: {full}");
        assert!(full.contains("SHOULD_FAIL"), "expected SHOULD_FAIL in error, got: {full}");
        // ACCESS_URL and SYS_RANDSTRING are declared but never appear in the template.
        assert!(full.contains("Declared parameters not found in template"), "expected unused error, got: {full}");
        assert!(full.contains("ACCESS_URL"), "expected ACCESS_URL in error, got: {full}");
        assert!(full.contains("SYS_RANDSTRING"), "expected SYS_RANDSTRING in error, got: {full}");
    }
}