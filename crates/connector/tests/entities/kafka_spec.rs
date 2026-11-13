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

//! Deserialization, SASL × `AuthenticationConfig` compatibility matrix, allowlist,
//! interaction constraints, and resolved value checks for `KafkaSpec`.

use std::collections::HashMap;

use connector::entities::auth_config::AuthenticationConfig;
use connector::entities::connector_template::{ConnectorMetadata, ConnectorTemplateDto};
use connector::entities::interaction::InteractionConfig;
use connector::entities::parameters::{TemplateMapString, TemplateVecString};
use connector::entities::resource::{
    KafkaOffsetReset, KafkaSaslMechanism, KafkaSecurityProtocol, KafkaSpec, ProtocolSpec,
};
use serde_json::json;

fn sample_basic_auth() -> AuthenticationConfig {
    serde_json::from_value(json!({
        "type": "BASIC_AUTH",
        "username": "alice",
        "password": { "type": "PLAIN", "content": "s3cret" }
    }))
    .unwrap()
}

fn sample_bearer_auth() -> AuthenticationConfig {
    serde_json::from_value(json!({
        "type": "BEARER_TOKEN",
        "token": { "type": "PLAIN", "content": "jwt-token" }
    }))
    .unwrap()
}

fn sample_oauth2_auth() -> AuthenticationConfig {
    serde_json::from_value(json!({
        "type": "OAUTH2",
        "grantType": { "type": "CLIENT_CREDENTIALS" },
        "tokenUrl": "https://idp.example.com/oauth/token",
        "clientId": "dataplane-client",
        "clientSecret": { "type": "PLAIN", "content": "secret" },
        "scopes": ["kafka.read"]
    }))
    .unwrap()
}

fn sample_api_key_auth() -> AuthenticationConfig {
    serde_json::from_value(json!({
        "type": "API_KEY",
        "location": "HEADER",
        "key": "X-Api-Key",
        "value": { "type": "PLAIN", "content": "abc" }
    }))
    .unwrap()
}

#[test]
fn kafka_spec_defaults_to_plaintext_and_latest() {
    let raw = json!({
        "brokers": ["localhost:9092"],
        "topic": "telemetry.events"
    });
    let spec: KafkaSpec = serde_json::from_value(raw).expect("minimal KafkaSpec");
    assert_eq!(spec.security_protocol, KafkaSecurityProtocol::Plaintext);
    assert_eq!(spec.sasl_mechanism, None);
    assert_eq!(spec.auto_offset_reset, KafkaOffsetReset::Latest);
    assert_eq!(spec.group_id, None);
    assert!(spec.client_properties.is_none());

    let out = serde_json::to_value(&spec).unwrap();
    assert_eq!(out["securityProtocol"], "PLAINTEXT");
    assert_eq!(out["autoOffsetReset"], "LATEST");
    assert!(out.get("saslMechanism").is_none());
    assert!(out.get("groupId").is_none());
    assert!(out.get("clientProperties").is_none());
}

#[test]
fn kafka_spec_sasl_mechanisms_round_trip_and_map_to_rdkafka() {
    let cases = [
        ("PLAIN", KafkaSaslMechanism::Plain, "PLAIN"),
        (
            "SCRAM_SHA_256",
            KafkaSaslMechanism::ScramSha256,
            "SCRAM-SHA-256",
        ),
        (
            "SCRAM_SHA_512",
            KafkaSaslMechanism::ScramSha512,
            "SCRAM-SHA-512",
        ),
        (
            "OAUTHBEARER",
            KafkaSaslMechanism::Oauthbearer,
            "OAUTHBEARER",
        ),
    ];

    for (wire, expected_enum, rdkafka_str) in cases {
        let parsed: KafkaSaslMechanism =
            serde_json::from_value(json!(wire)).expect("valid SASL mechanism");
        assert_eq!(parsed, expected_enum);
        assert_eq!(parsed.as_rdkafka_str(), rdkafka_str);
        assert_eq!(serde_json::to_value(parsed).unwrap(), json!(wire));
    }

    assert_eq!(KafkaOffsetReset::Latest.as_rdkafka_str(), "latest");
    assert_eq!(KafkaOffsetReset::Earliest.as_rdkafka_str(), "earliest");
    assert_eq!(
        KafkaSecurityProtocol::SaslSsl.as_rdkafka_str(),
        "SASL_SSL"
    );
}

#[test]
fn no_auth_accepts_plaintext_and_ssl_but_rejects_sasl() {
    let mut spec = KafkaSpec {
        brokers: TemplateVecString::Value(vec!["broker:9092".into()]),
        topic: "sensors".into(),
        ..Default::default()
    };
    assert!(spec.validate_with_auth(&AuthenticationConfig::NoAuth).is_ok());

    spec.security_protocol = KafkaSecurityProtocol::Ssl;
    assert!(spec.validate_with_auth(&AuthenticationConfig::NoAuth).is_ok());

    spec.security_protocol = KafkaSecurityProtocol::SaslSsl;
    spec.sasl_mechanism = Some(KafkaSaslMechanism::Plain);
    assert!(spec.validate_with_auth(&AuthenticationConfig::NoAuth).is_err());
}

#[test]
fn basic_auth_accepts_plain_and_scram_over_sasl() {
    let auth = sample_basic_auth();
    for proto in [
        KafkaSecurityProtocol::SaslPlaintext,
        KafkaSecurityProtocol::SaslSsl,
    ] {
        for mech in [
            KafkaSaslMechanism::Plain,
            KafkaSaslMechanism::ScramSha256,
            KafkaSaslMechanism::ScramSha512,
        ] {
            let spec = KafkaSpec {
                brokers: TemplateVecString::Value(vec!["broker:9093".into()]),
                topic: "orders".into(),
                security_protocol: proto,
                sasl_mechanism: Some(mech),
                ..Default::default()
            };
            assert!(
                spec.validate_with_auth(&auth).is_ok(),
                "expected {proto:?} + {mech:?} to succeed with BASIC_AUTH"
            );
        }
    }
}

#[test]
fn basic_auth_rejects_non_sasl_or_oauthbearer() {
    let auth = sample_basic_auth();
    let non_sasl = KafkaSpec {
        brokers: TemplateVecString::Value(vec!["broker:9092".into()]),
        topic: "orders".into(),
        security_protocol: KafkaSecurityProtocol::Plaintext,
        ..Default::default()
    };
    assert!(non_sasl.validate_with_auth(&auth).is_err());

    let wrong_mech = KafkaSpec {
        brokers: TemplateVecString::Value(vec!["broker:9093".into()]),
        topic: "orders".into(),
        security_protocol: KafkaSecurityProtocol::SaslSsl,
        sasl_mechanism: Some(KafkaSaslMechanism::Oauthbearer),
        ..Default::default()
    };
    assert!(wrong_mech.validate_with_auth(&auth).is_err());
}

#[test]
fn bearer_token_and_oauth2_require_sasl_oauthbearer() {
    let ok_spec = KafkaSpec {
        brokers: TemplateVecString::Value(vec!["broker:9093".into()]),
        topic: "events".into(),
        security_protocol: KafkaSecurityProtocol::SaslSsl,
        sasl_mechanism: Some(KafkaSaslMechanism::Oauthbearer),
        ..Default::default()
    };
    assert!(ok_spec.validate_with_auth(&sample_bearer_auth()).is_ok());
    assert!(ok_spec.validate_with_auth(&sample_oauth2_auth()).is_ok());

    let bad_spec = KafkaSpec {
        sasl_mechanism: Some(KafkaSaslMechanism::ScramSha256),
        ..ok_spec
    };
    assert!(bad_spec.validate_with_auth(&sample_bearer_auth()).is_err());
    assert!(bad_spec.validate_with_auth(&sample_oauth2_auth()).is_err());
}

#[test]
fn api_key_auth_is_rejected_for_kafka() {
    let spec = KafkaSpec {
        brokers: TemplateVecString::Value(vec!["broker:9092".into()]),
        topic: "events".into(),
        ..Default::default()
    };
    let err = spec.validate_with_auth(&sample_api_key_auth()).unwrap_err();
    assert!(format!("{err:#}").contains("API_KEY"));
}

#[test]
fn sasl_mechanism_presence_must_match_security_protocol() {
    let missing_mech = KafkaSpec {
        brokers: TemplateVecString::Value(vec!["broker:9093".into()]),
        topic: "events".into(),
        security_protocol: KafkaSecurityProtocol::SaslPlaintext,
        sasl_mechanism: None,
        ..Default::default()
    };
    assert!(missing_mech.validate_with_auth(&sample_basic_auth()).is_err());

    let stray_mech = KafkaSpec {
        brokers: TemplateVecString::Value(vec!["broker:9092".into()]),
        topic: "events".into(),
        security_protocol: KafkaSecurityProtocol::Plaintext,
        sasl_mechanism: Some(KafkaSaslMechanism::Plain),
        ..Default::default()
    };
    assert!(stray_mech
        .validate_with_auth(&AuthenticationConfig::NoAuth)
        .is_err());
}

#[test]
fn interaction_rejects_kafka_in_pull_mode() {
    let raw = json!({
        "mode": "PULL",
        "dataAccess": {
            "protocol": "KAFKA",
            "brokers": ["localhost:9092"],
            "topic": "events"
        }
    });
    let cfg: InteractionConfig = serde_json::from_value(raw).unwrap();
    let err = cfg.validate().unwrap_err();
    assert!(format!("{err:#}").contains("PULL"));
}

#[test]
fn interaction_rejects_kafka_in_push_unsubscribe() {
    let raw = json!({
        "mode": "PUSH",
        "subscribe": {
            "protocol": "KAFKA",
            "brokers": ["localhost:9092"],
            "topic": "events"
        },
        "unsubscribe": {
            "protocol": "KAFKA",
            "brokers": ["localhost:9092"],
            "topic": "events"
        }
    });
    let cfg: InteractionConfig = serde_json::from_value(raw).unwrap();
    let err = cfg.validate().unwrap_err();
    assert!(format!("{err:#}").contains("unsubscribe"));
}

#[test]
fn client_properties_allowlist_accepts_safe_keys_and_blocks_dangerous_keys() {
    let mut allowed = HashMap::new();
    allowed.insert("session.timeout.ms".to_string(), "10000".to_string());
    allowed.insert("max.poll.interval.ms".to_string(), "300000".to_string());
    allowed.insert("fetch.min.bytes".to_string(), "1".to_string());
    allowed.insert("fetch.max.bytes".to_string(), "1048576".to_string());
    allowed.insert("ssl.ca.location".to_string(), "/etc/ssl/ca.pem".to_string());

    let ok_spec = KafkaSpec {
        brokers: TemplateVecString::Value(vec!["broker:9092".into()]),
        topic: "events".into(),
        client_properties: Some(TemplateMapString::Value(allowed)),
        ..Default::default()
    };
    assert!(ok_spec
        .validate_with_auth(&AuthenticationConfig::NoAuth)
        .is_ok());
    assert!(ok_spec.validate_resolved().is_ok());

    for forbidden in ["plugin.library.paths", "sasl.password", "bootstrap.servers"] {
        let mut bad = HashMap::new();
        bad.insert(forbidden.to_string(), "injected".to_string());
        let bad_spec = KafkaSpec {
            brokers: TemplateVecString::Value(vec!["broker:9092".into()]),
            topic: "events".into(),
            client_properties: Some(TemplateMapString::Value(bad)),
            ..Default::default()
        };
        assert!(
            bad_spec
                .validate_with_auth(&AuthenticationConfig::NoAuth)
                .is_err(),
            "expected {forbidden} to be rejected in validate_with_auth"
        );
        assert!(
            bad_spec.validate_resolved().is_err(),
            "expected {forbidden} to be rejected in validate_resolved"
        );
    }
}

#[test]
fn validate_resolved_accepts_valid_brokers_and_topics_including_ipv6() {
    let spec = KafkaSpec {
        brokers: TemplateVecString::Value(vec![
            "redpanda.internal:9092".into(),
            "[::1]:9092".into(),
        ]),
        topic: "iot.sensors_v1-raw".into(),
        group_id: Some("ds-group-1".into()),
        ..Default::default()
    };
    assert!(spec.validate_resolved().is_ok());
}

#[test]
fn validate_resolved_rejects_invalid_brokers() {
    for bad_brokers in [
        vec![],
        vec!["localhost".to_string()],
        vec!["localhost:0".to_string()],
        vec!["localhost:notaport".to_string()],
        vec![":9092".to_string()],
    ] {
        let spec = KafkaSpec {
            brokers: TemplateVecString::Value(bad_brokers.clone()),
            topic: "events".into(),
            ..Default::default()
        };
        assert!(
            spec.validate_resolved().is_err(),
            "expected brokers {bad_brokers:?} to fail validation"
        );
    }
}

#[test]
fn validate_resolved_rejects_invalid_topics_and_empty_group_id() {
    let long_topic = "a".repeat(250);
    for bad_topic in ["", ".", "..", "topic/slash", "topic space", long_topic.as_str()] {
        let spec = KafkaSpec {
            brokers: TemplateVecString::Value(vec!["broker:9092".into()]),
            topic: bad_topic.to_string(),
            ..Default::default()
        };
        assert!(
            spec.validate_resolved().is_err(),
            "expected topic '{bad_topic}' to fail validation"
        );
    }

    let empty_group = KafkaSpec {
        brokers: TemplateVecString::Value(vec!["broker:9092".into()]),
        topic: "events".into(),
        group_id: Some("   ".into()),
        ..Default::default()
    };
    assert!(empty_group.validate_resolved().is_err());
}

#[test]
fn validate_resolved_permits_deferred_runtime_placeholders() {
    let spec = KafkaSpec {
        brokers: TemplateVecString::Template("{{__RUNTIME_BROKERS__}}".into()),
        topic: "{{__RUNTIME_TOPIC__}}".into(),
        group_id: Some("{{__RUNTIME_GROUP__}}".into()),
        client_properties: Some(TemplateMapString::Template("{{__RUNTIME_PROPS__}}".into())),
        ..Default::default()
    };
    assert!(spec.validate_resolved().is_ok());

    let unresolved_instance_prop = KafkaSpec {
        brokers: TemplateVecString::Value(vec!["broker:9092".into()]),
        topic: "events".into(),
        client_properties: Some(TemplateMapString::Template("{{__PROPS__}}".into())),
        ..Default::default()
    };
    assert!(unresolved_instance_prop.validate_resolved().is_err());
}

#[test]
fn connector_template_dto_validates_end_to_end_semantics_and_resolved() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: Some("kafka-telemetry".into()),
            author: Some("UPM".into()),
            description: Some("Kafka push stream".into()),
            version: Some("1.0.0".into()),
            created_at: None,
        },
        authentication: sample_basic_auth(),
        interaction: serde_json::from_value(json!({
            "mode": "PUSH",
            "lifetime": "INFINITE",
            "maxRetries": 3,
            "subscribe": {
                "protocol": "KAFKA",
                "brokers": ["redpanda:9092"],
                "topic": "telemetry.v1",
                "securityProtocol": "SASL_SSL",
                "saslMechanism": "SCRAM_SHA_256",
                "autoOffsetReset": "EARLIEST"
            },
            "unsubscribe": null
        }))
        .unwrap(),
        parameters: vec![],
    };

    assert!(dto.validate_semantics().is_ok());
    assert!(dto.validate_resolved().is_ok());

    if let InteractionConfig::Push(lc) = &dto.interaction {
        assert!(matches!(&lc.subscribe, ProtocolSpec::Kafka(_)));
    } else {
        panic!("expected Push interaction");
    }
}
