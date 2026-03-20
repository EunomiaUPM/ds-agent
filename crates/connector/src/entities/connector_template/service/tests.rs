/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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


use crate::data::entities::connector_templates;
use crate::data::factory_trait::MockConnectorRepoTrait;
use crate::data::repo_traits::connector_template_repo::{
    ConnectorTemplateRepoTrait, MockConnectorTemplateRepoTrait,
};
use crate::entities::connector_template::connector_template::ConnectorTemplateEntitiesService;
use crate::entities::connector_template::{ConnectorTemplateDto, ConnectorTemplateEntitiesTrait};
use serde_json::json;
use std::sync::Arc;

/// Builds a `ConnectorTemplateEntitiesService` backed by mock repos.
///
/// `create_template` is pre-configured to echo the received model back,
/// simulating a successful DB insert.  Tests that exercise the validation
/// path never reach the repo, so the expectation is simply never triggered
/// — mockall only panics on *unexpected* calls, not on uncalled ones.
fn mock_entities() -> ConnectorTemplateEntitiesService {
    let mut template_repo = MockConnectorTemplateRepoTrait::new();
    template_repo.expect_create_template().returning(|m| Ok(echo_model(m)));

    let template_repo: Arc<dyn ConnectorTemplateRepoTrait> = Arc::new(template_repo);
    let template_repo_clone = Arc::clone(&template_repo);

    let mut repo = MockConnectorRepoTrait::new();
    repo.expect_get_templates_repo().returning(move || Arc::clone(&template_repo_clone));

    ConnectorTemplateEntitiesService::new(Arc::new(repo))
}

/// Simulates a successful DB insert by echoing the `spec` from the received
/// [`NewConnectorTemplateModel`] back as a [`connector_templates::Model`].
/// Since `map_model_to_dto` reads `authentication`, `interaction`, and
/// `parameters` from `spec`, echoing the same JSON guarantees a valid
/// round-trip without a real database.
///
/// [`NewConnectorTemplateModel`]: crate::data::entities::connector_templates::NewConnectorTemplateModel
fn echo_model(
    m: &crate::data::entities::connector_templates::NewConnectorTemplateModel,
) -> connector_templates::Model {
    connector_templates::Model {
        name: m.name.clone().unwrap_or_default(),
        version: m.version.clone().unwrap_or_default(),
        author: m.author.clone().unwrap_or_default(),
        created_at: chrono::Utc::now().into(),
        spec: m.spec.clone(),
    }
}

/// Happy path: all declared parameters match the template and the types are compatible.
///
/// Parameter / context compatibility verified here:
///
/// | Name           | Declared type       | Found context       | Compatible? |
/// |----------------|---------------------|---------------------|-------------|
/// | `ACCESS_URL`   | `STRING`            | `urlTemplate` (String interpolation) | ✓ |
/// | `ACCESS_METHODS` | `VEC<STRING>`     | `method` array item (VecString)      | ✓ |
/// | `HEADERS`      | `MAP<STRING,STRING>`| `__EXTRA__` header value (MapString) | ✓ |
///
/// `SYS_RANDSTRING` appears in the password field, which is a `SecretString`
/// and is intentionally not scanned. Even if it were found, it would be silently
/// excluded by the `exclude_runtime = true` flag (prefix `SYS_`).
#[tokio::test]
async fn test_create_instance() {
    let json_dto = json!({
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
                "headers": {
                    "Content-Type": "application/json",
                    "__EXTRA__": "{{__HEADERS__}}"
                }
            }
        },
        "parameters": [
            { "paramType": "STRING",           "name": "ACCESS_URL",     "title": "Access url",     "required": true },
            { "paramType": "VEC<STRING>",       "name": "ACCESS_METHODS", "title": "Access methods", "required": true },
            { "paramType": "MAP<STRING,STRING>","name": "HEADERS",        "title": "Headers",        "required": true }
        ]
    });
    let mut dto: ConnectorTemplateDto = serde_json::from_value(json_dto).unwrap();

    let result = mock_entities().create_template(&mut dto).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_instance_too_much_parameters() {
    // SHOULD_FAIL is declared in parameters[] but never used in the template.
    // Validation fails → the repo is never reached (no expectation needed).
    let json_dto = json!({
        "authentication": {
            "type": "BASIC_AUTH",
            "username": "asd",
            "password": { "type": "PLAIN", "content": "{{__SYS_RANDSTRING__}}" }
        },
        "interaction": {
            "mode": "PULL",
            "dataAccess": {
                "protocol": "HTTP",
                "urlTemplate": "http://data-plane/{{__ACCESS_URL__}}",
                "method": ["GET", "{{__ACCESS_METHODS__}}"],
                "headers": { "Content-Type": "application/json", "__EXTRA__": "{{__HEADERS__}}" }
            }
        },
        "parameters": [
            { "paramType": "STRING",           "name": "SHOULD_FAIL",    "title": "Should fail",    "required": true },
            { "paramType": "STRING",           "name": "ACCESS_URL",     "title": "Access url",     "required": true },
            { "paramType": "VEC<STRING>",       "name": "ACCESS_METHODS", "title": "Access methods", "required": true },
            { "paramType": "MAP<STRING,STRING>","name": "HEADERS",        "title": "Headers",        "required": true }
        ]
    });
    let mut dto: ConnectorTemplateDto = serde_json::from_value(json_dto).unwrap();

    let err = mock_entities().create_template(&mut dto).await.unwrap_err();

    assert_eq!(err.to_string(), "Error validating connector template");
    let full = format!("{:#}", err);
    assert!(
        full.contains("Declared parameters not found in template"),
        "expected unused error, got: {full}"
    );
    assert!(
        full.contains("SHOULD_FAIL"),
        "expected SHOULD_FAIL in error, got: {full}"
    );
    assert!(
        !full.contains("ACCESS_URL"),
        "ACCESS_URL should not be in error: {full}"
    );
    assert!(
        !full.contains("ACCESS_METHODS"),
        "ACCESS_METHODS should not be in error: {full}"
    );
    assert!(!full.contains("HEADERS"), "HEADERS should not be in error: {full}");
}

#[tokio::test]
async fn test_create_instance_too_less_parameters() {
    // SHOULD_FAIL is used in the template but not declared in parameters[].
    let json_dto = json!({
        "authentication": {
            "type": "BASIC_AUTH",
            "username": "asd",
            "password": { "type": "PLAIN", "content": "{{__SYS_RANDSTRING__}}" }
        },
        "interaction": {
            "mode": "PULL",
            "dataAccess": {
                "protocol": "HTTP",
                "urlTemplate": "http://data-plane/{{__SHOULD_FAIL__}}",
                "method": ["GET", "{{__ACCESS_METHODS__}}"],
                "headers": { "Content-Type": "application/json", "__EXTRA__": "{{__HEADERS__}}" }
            }
        },
        "parameters": [
            { "paramType": "VEC<STRING>",       "name": "ACCESS_METHODS", "title": "Access methods", "required": true },
            { "paramType": "MAP<STRING,STRING>","name": "HEADERS",        "title": "Headers",        "required": true }
        ]
    });
    let mut dto: ConnectorTemplateDto = serde_json::from_value(json_dto).unwrap();

    let err = mock_entities().create_template(&mut dto).await.unwrap_err();

    assert_eq!(err.to_string(), "Error validating connector template");
    let full = format!("{:#}", err);
    assert!(
        full.contains("Undeclared parameters found in template"),
        "expected undeclared error, got: {full}"
    );
    assert!(
        full.contains("SHOULD_FAIL"),
        "expected SHOULD_FAIL in error, got: {full}"
    );
    assert!(
        !full.contains("ACCESS_METHODS"),
        "ACCESS_METHODS should not be in error: {full}"
    );
    assert!(!full.contains("HEADERS"), "HEADERS should not be in error: {full}");
}

#[tokio::test]
async fn test_create_instance_runtime_in_parameters() {
    // SYS_* and RUNTIME_* names in the template are excluded from validation —
    // but declaring them explicitly in parameters[] is still an error because
    // the validator sees them as unused (they never appear in found_set).
    // Additionally SHOULD_FAIL is used in the template but not declared, and
    // ACCESS_URL is declared but never used.
    let json_dto = json!({
        "authentication": {
            "type": "BASIC_AUTH",
            "username": "asd",
            "password": { "type": "PLAIN", "content": "{{__SYS_RANDSTRING__}}" }
        },
        "interaction": {
            "mode": "PULL",
            "dataAccess": {
                "protocol": "HTTP",
                "urlTemplate": "http://data-plane/{{__SHOULD_FAIL__}}",
                "method": ["GET", "{{__ACCESS_METHODS__}}"],
                "headers": { "Content-Type": "application/json", "__EXTRA__": "{{__HEADERS__}}" }
            }
        },
        "parameters": [
            { "paramType": "STRING",           "name": "ACCESS_URL",      "title": "Access url",     "required": true },
            { "paramType": "VEC<STRING>",       "name": "ACCESS_METHODS",  "title": "Access methods", "required": true },
            { "paramType": "MAP<STRING,STRING>","name": "HEADERS",         "title": "Headers",        "required": true },
            { "paramType": "STRING",           "name": "SYS_RANDSTRING",  "title": "Sys rand",       "required": true }
        ]
    });
    let mut dto: ConnectorTemplateDto = serde_json::from_value(json_dto).unwrap();

    let err = mock_entities().create_template(&mut dto).await.unwrap_err();

    assert_eq!(err.to_string(), "Error validating connector template");
    let full = format!("{:#}", err);
    assert!(
        full.contains("Undeclared parameters found in template"),
        "expected undeclared error, got: {full}"
    );
    assert!(
        full.contains("SHOULD_FAIL"),
        "expected SHOULD_FAIL in error, got: {full}"
    );
    assert!(
        full.contains("Declared parameters not found in template"),
        "expected unused error, got: {full}"
    );
    assert!(
        full.contains("ACCESS_URL"),
        "expected ACCESS_URL in error, got: {full}"
    );
    assert!(
        full.contains("SYS_RANDSTRING"),
        "expected SYS_RANDSTRING in error, got: {full}"
    );
}

/// Verifies that the validator catches simultaneously undeclared parameters,
/// unused parameters, and type mismatches in a single pass.
///
/// Parameter / context analysis for this template:
///
/// | Name             | Declared type  | Found context              | Error kind      |
/// |------------------|----------------|----------------------------|-----------------|
/// | `SHOULD_FAIL`    | *(not declared)* | `urlTemplate` (String)   | Undeclared      |
/// | `ACCESS_URL`     | `VEC<STRING>`  | *(never used)*             | Unused          |
/// | `ACCESS_METHODS` | `VEC<STRING>`  | `method` array item (VecString) | ✓ compatible |
/// | `HEADERS`        | `VEC<STRING>`  | `__EXTRA__` header value (MapString) | Type mismatch |
///
/// `VEC<STRING>` is only compatible when the placeholder occupies an entire
/// `TemplateVecString` slot or an element inside a `Value` vec.  Using it in
/// a `MapString` context (the value of `__EXTRA__`) is a mismatch because
/// `FoundParameterType::MapString` is only compatible with `ParameterType::MapStringString`.
#[tokio::test]
async fn test_create_instance_type_error() {
    let json_dto = json!({
        "authentication": {
            "type": "BASIC_AUTH",
            "username": "asd",
            "password": { "type": "PLAIN", "content": "{{__SYS_RANDSTRING__}}" }
        },
        "interaction": {
            "mode": "PULL",
            "dataAccess": {
                "protocol": "HTTP",
                "urlTemplate": "http://data-plane/{{__SHOULD_FAIL__}}",
                "method": ["GET", "{{__ACCESS_METHODS__}}"],
                "headers": { "Content-Type": "application/json", "__EXTRA__": "{{__HEADERS__}}" }
            }
        },
        "parameters": [
            { "paramType": "VEC<STRING>","name": "ACCESS_URL",     "title": "Access url",     "required": true },
            { "paramType": "VEC<STRING>","name": "ACCESS_METHODS", "title": "Access methods", "required": true },
            { "paramType": "VEC<STRING>","name": "HEADERS",        "title": "Headers",        "required": true }
        ]
    });
    let mut dto: ConnectorTemplateDto = serde_json::from_value(json_dto).unwrap();

    let err = mock_entities().create_template(&mut dto).await.unwrap_err();

    assert_eq!(err.to_string(), "Error validating connector template");
    let full = format!("{:#}", err);
    assert!(
        full.contains("Undeclared parameters found in template"),
        "expected undeclared error, got: {full}"
    );
    assert!(
        full.contains("SHOULD_FAIL"),
        "expected SHOULD_FAIL in undeclared error, got: {full}"
    );
    assert!(
        full.contains("Declared parameters not found in template"),
        "expected unused error, got: {full}"
    );
    assert!(
        full.contains("ACCESS_URL"),
        "expected ACCESS_URL in unused error, got: {full}"
    );
    assert!(
        full.contains("Type mismatches"),
        "expected type mismatch error, got: {full}"
    );
    assert!(
        full.contains("HEADERS"),
        "expected HEADERS in type mismatch error, got: {full}"
    );
    assert!(
        !full.contains("ACCESS_METHODS"),
        "ACCESS_METHODS should not be in error: {full}"
    );
}
