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

//! Placeholders found in the authentication config, and visited before the interaction.

use super::*;

/// NoAuth has no fields to scan, the auth step should contribute nothing.
#[test]
fn no_auth_extracts_no_parameters() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    assert!(run(dto).is_empty());
}

/// The visitor scans the username field of BasicAuth.
#[test]
fn basic_auth_username_template_extracts_parameter() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::BasicAuth(BasicAuthConfig {
            username: "{{__USERNAME__}}".to_string(),
            password: SecretString {
                source: SecretSource::Plain("secret".to_string()),
            },
        }),
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("USERNAME", found[0]);
}

/// A literal username value contains no template placeholders.
#[test]
fn basic_auth_literal_username_extracts_no_parameters() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::BasicAuth(BasicAuthConfig {
            username: "admin".to_string(),
            password: SecretString {
                source: SecretSource::Plain("secret".to_string()),
            },
        }),
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    assert!(run(dto).is_empty());
}

/// A placeholder in a plain bearer token is found.
#[test]
fn bearer_token_plain_template_extracts_parameter() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::BearerToken {
            token: SecretString {
                source: SecretSource::Plain("{{__TOKEN__}}".to_string()),
            },
        },
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("TOKEN", found[0]);
}

/// A literal key name has no placeholders; value is SecretString (not scanned).
#[test]
fn api_key_with_literal_key_extracts_no_parameters() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::ApiKey {
            key: "X-Api-Key".to_string(),
            value: SecretString {
                source: SecretSource::Plain("s3cr3t".to_string()),
            },
            location: ApiKeyLocation::Header,
        },
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    assert!(run(dto).is_empty());
}

/// The header/query-param name may itself be parameterised.
#[test]
fn api_key_with_template_key_extracts_parameter() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::ApiKey {
            key: "{{__API_KEY_HEADER__}}".to_string(),
            value: SecretString {
                source: SecretSource::Plain("s3cr3t".to_string()),
            },
            location: ApiKeyLocation::Header,
        },
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("API_KEY_HEADER", found[0]);
}

/// All fields are literals — no placeholders anywhere, including client_secret.
#[test]
fn oauth2_with_literal_fields_extracts_no_parameters() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::OAuth2 {
            grant_type: OAuthGrantType::ClientCredentials,
            token_url: "https://auth.example.com/token".to_string(),
            client_id: "my-client".to_string(),
            client_secret: SecretString {
                source: SecretSource::Plain("s3cr3t".to_string()),
            },
            scopes: TemplateVecString::Value(vec!["read".to_string()]),
            on_token_expire: Default::default(),
        },
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    assert!(run(dto).is_empty());
}

/// token_url, client_id, and scopes all support placeholders.
#[test]
fn oauth2_with_template_fields_extracts_parameters() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::OAuth2 {
            grant_type: OAuthGrantType::ClientCredentials,
            token_url: "{{__TOKEN_URL__}}".to_string(),
            client_id: "{{__CLIENT_ID__}}".to_string(),
            client_secret: SecretString {
                source: SecretSource::Plain("s3cr3t".to_string()),
            },
            scopes: TemplateVecString::Template("{{__SCOPES__}}".to_string()),
            on_token_expire: Default::default(),
        },
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(3, found.len());
    assert!(found.contains(&"TOKEN_URL".to_string()));
    assert!(found.contains(&"CLIENT_ID".to_string()));
    assert!(found.contains(&"SCOPES".to_string()));
}

/// Auth is visited before interaction; parameters appear in that order.
#[test]
fn basic_auth_and_http_url_extracts_all_parameters() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::BasicAuth(BasicAuthConfig {
            username: "{{__USERNAME__}}".to_string(),
            password: SecretString {
                source: SecretSource::Plain("secret".to_string()),
            },
        }),
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/{{__RESOURCE_ID__}}".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(2, found.len());
    assert_eq!("USERNAME", found[0]); // auth is visited first
    assert_eq!("RESOURCE_ID", found[1]);
}
