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

//! Placeholders found in Kafka specs: topic, brokers, groupId and clientProperties.

use super::*;
use connector::entities::parameters::TemplateMapString;
use std::collections::HashMap;

/// A placeholder in the topic field should be extracted.
#[test]
fn push_kafka_topic_template_extracts_parameter() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Push(PushLifecycle {
            lifetime: Default::default(),
            max_retries: None,
            subscribe: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                topic: "{{__TOPIC__}}".to_string(),
                ..Default::default()
            }),
            unsubscribe: None,
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("TOPIC", found[0]);
}

/// When brokers is a Template string, the placeholder is extracted.
#[test]
fn push_kafka_brokers_as_template_extracts_parameter() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Push(PushLifecycle {
            lifetime: Default::default(),
            max_retries: None,
            subscribe: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Template("{{__BROKERS__}}".to_string()),
                topic: "events".to_string(),
                ..Default::default()
            }),
            unsubscribe: None,
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("BROKERS", found[0]);
}

/// A Value vec where one broker address contains a placeholder should be scanned.
#[test]
fn push_kafka_brokers_value_with_template_item_extracts_parameter() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Push(PushLifecycle {
            lifetime: Default::default(),
            max_retries: None,
            subscribe: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Value(vec![
                    "localhost:9092".to_string(),
                    "{{__BROKER_HOST__}}:9092".to_string(),
                ]),
                topic: "events".to_string(),
                ..Default::default()
            }),
            unsubscribe: None,
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("BROKER_HOST", found[0]);
}

/// A placeholder in the optional group_id should be extracted.
#[test]
fn push_kafka_group_id_extracts_parameter() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Push(PushLifecycle {
            lifetime: Default::default(),
            max_retries: None,
            subscribe: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                topic: "events".to_string(),
                group_id: Some("{{__GROUP_ID__}}".to_string()),
                ..Default::default()
            }),
            unsubscribe: None,
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("GROUP_ID", found[0]);
}

/// When group_id is None and all other fields are literals, nothing is found.
#[test]
fn push_kafka_without_group_id_extracts_nothing() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Push(PushLifecycle {
            lifetime: Default::default(),
            max_retries: None,
            subscribe: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                topic: "events".to_string(),
                ..Default::default()
            }),
            unsubscribe: None,
        }),
        parameters: vec![],
    };
    assert!(run(dto).is_empty());
}

/// Placeholders inside `clientProperties` (`Template` or `__EXTRA__` entry) are extracted.
#[test]
fn push_kafka_client_properties_extracts_parameters() {
    let dto_template = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Push(PushLifecycle {
            lifetime: Default::default(),
            max_retries: None,
            subscribe: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                topic: "events".to_string(),
                client_properties: Some(TemplateMapString::Template(
                    "{{__CLIENT_PROPS__}}".to_string(),
                )),
                ..Default::default()
            }),
            unsubscribe: None,
        }),
        parameters: vec![],
    };
    assert_eq!(vec!["CLIENT_PROPS"], run(dto_template));

    let dto_extra = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Push(PushLifecycle {
            lifetime: Default::default(),
            max_retries: None,
            subscribe: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                topic: "events".to_string(),
                client_properties: Some(TemplateMapString::Value(HashMap::from([(
                    "__EXTRA__".to_string(),
                    "{{__EXTRA_PROPS__}}".to_string(),
                )]))),
                ..Default::default()
            }),
            unsubscribe: None,
        }),
        parameters: vec![],
    };
    assert_eq!(vec!["EXTRA_PROPS"], run(dto_extra));
}
