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

//! Creating a template validates its placeholders against its declared parameters.

use super::*;

/// An OAuth2 password-grant template keeps its camelCase fields and nested grant type.
#[tokio::test]
async fn oauth2_password_grant_template_round_trips() {
    let json_dto = json!({
        "authentication": {
            "type": "OAUTH2",
            "grantType": {
                "type": "PASSWORD",
                "username": "{{__USER_NAME__}}",
                "password": { "type": "PLAIN", "content": "{{__USER_PASS__}}" }
            },
            "tokenUrl": "{{__TOKEN_URL__}}",
            "clientId": "{{__CLIENT_ID__}}",
            "clientSecret": { "type": "PLAIN", "content": "{{__CLIENT_SECRET__}}" },
            "scopes": [],
            "onTokenExpire": "REFRESH_OR_REFETCH"
        },
        "interaction": {
            "mode": "PULL",
            "dataAccess": {
                "protocol": "HTTP",
                "urlTemplate": "{{__ACCESS_URL__}}",
                "method": "{{__ACCESS_METHODS__}}",
                "headers": "{{__ACCESS_HEADERS__}}"
            }
        },
        "parameters": [
            { "paramType": "STRING",            "name": "ACCESS_URL",     "title": "Url",             "required": true },
            { "paramType": "VEC<STRING>",        "name": "ACCESS_METHODS", "title": "Methods",         "required": true },
            { "paramType": "MAP<STRING,STRING>", "name": "ACCESS_HEADERS", "title": "Headers",         "required": true },
            { "paramType": "STRING",            "name": "USER_NAME",      "title": "User Name",       "required": true },
            { "paramType": "STRING",            "name": "USER_PASS",      "title": "User Password",   "required": true },
            { "paramType": "STRING",            "name": "TOKEN_URL",      "title": "Token URL",       "required": true },
            { "paramType": "STRING",            "name": "CLIENT_ID",      "title": "Client ID",       "required": true },
            { "paramType": "STRING",            "name": "CLIENT_SECRET",  "title": "Client Secret",   "required": true }
        ]
    });
    let mut dto: ConnectorTemplateDto = serde_json::from_value(json_dto).unwrap();
    let result = mock_entities()
        .create_template(&TestUsers::user("tenant-1", "/admin/tenant-1"), &mut dto)
        .await;
    assert!(
        result.is_ok(),
        "expected ok, got: {:#?}",
        result.unwrap_err()
    );
}

/// A template whose placeholders match its declared parameters and types is created.
#[tokio::test]
async fn template_whose_parameters_match_is_created() {
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

    let result = mock_entities()
        .create_template(&TestUsers::user("tenant-1", "/admin/tenant-1"), &mut dto)
        .await;
    assert!(result.is_ok());
}

/// A declared parameter the template never uses is rejected before the repo.
#[tokio::test]
async fn declared_but_unused_parameter_is_rejected() {
    // SHOULD_FAIL is declared in parameters[] but never used in the template.
    // Validation fails - the repo is never reached (no expectation needed).
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

    let err = mock_entities()
        .create_template(&TestUsers::user("tenant-1", "/admin/tenant-1"), &mut dto)
        .await
        .unwrap_err();

    assert_eq!(err.to_string(), "Error validating connector template");
    let full = format!("{err:#}");
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
    assert!(
        !full.contains("HEADERS"),
        "HEADERS should not be in error: {full}"
    );
}

/// A placeholder without declaration is rejected.
#[tokio::test]
async fn used_but_undeclared_parameter_is_rejected() {
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

    let err = mock_entities()
        .create_template(&TestUsers::user("tenant-1", "/admin/tenant-1"), &mut dto)
        .await
        .unwrap_err();

    assert_eq!(err.to_string(), "Error validating connector template");
    let full = format!("{err:#}");
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
    assert!(
        !full.contains("HEADERS"),
        "HEADERS should not be in error: {full}"
    );
}

/// `SYS_` and `RUNTIME_` names need no declaration, and declaring them is an error.
#[tokio::test]
async fn declaring_sys_or_runtime_parameters_is_rejected() {
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

    let err = mock_entities()
        .create_template(&TestUsers::user("tenant-1", "/admin/tenant-1"), &mut dto)
        .await
        .unwrap_err();

    assert_eq!(err.to_string(), "Error validating connector template");
    let full = format!("{err:#}");
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

/// Undeclared, unused and mistyped parameters are reported in one pass.
#[tokio::test]
async fn undeclared_unused_and_mistyped_parameters_are_reported_together() {
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

    let err = mock_entities()
        .create_template(&TestUsers::user("tenant-1", "/admin/tenant-1"), &mut dto)
        .await
        .unwrap_err();

    assert_eq!(err.to_string(), "Error validating connector template");
    let full = format!("{err:#}");
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
