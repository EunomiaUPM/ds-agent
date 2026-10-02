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

//! Placeholders found in Kafka pull and push specs: topic, brokers and group id.

use super::*;

/// A placeholder in the topic field should be extracted.
#[test]
fn pull_kafka_topic_template_extracts_parameter() {
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
            data_access: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                topic: "{{__TOPIC__}}".to_string(),
                group_id: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("TOPIC", found[0]);
}

/// When brokers is a Template string, the placeholder is extracted.
#[test]
fn pull_kafka_brokers_as_template_extracts_parameter() {
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
            data_access: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Template("{{__BROKERS__}}".to_string()),
                topic: "events".to_string(),
                group_id: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("BROKERS", found[0]);
}

/// A Value vec where one broker address contains a placeholder should be scanned.
#[test]
fn pull_kafka_brokers_value_with_template_item_extracts_parameter() {
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
            data_access: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Value(vec![
                    "localhost:9092".to_string(),
                    "{{__BROKER_HOST__}}:9092".to_string(),
                ]),
                topic: "events".to_string(),
                group_id: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("BROKER_HOST", found[0]);
}

/// A placeholder in the optional group_id should be extracted.
#[test]
fn pull_kafka_group_id_extracts_parameter() {
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
            data_access: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                topic: "events".to_string(),
                group_id: Some("{{__GROUP_ID__}}".to_string()),
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("GROUP_ID", found[0]);
}

/// When group_id is None and all other fields are literals, nothing is found.
#[test]
fn pull_kafka_without_group_id_extracts_nothing() {
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
            data_access: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                topic: "events".to_string(),
                group_id: None,
            }),
        }),
        parameters: vec![],
    };
    assert!(run(dto).is_empty());
}

/// Push interaction can also use Kafka; the topic placeholder should be found.
#[test]
fn push_kafka_subscribe_without_unsubscribe_extracts_parameters() {
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
            subscribe: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                topic: "{{__TOPIC__}}".to_string(),
                group_id: None,
            }),
            unsubscribe: None,
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("TOPIC", found[0]);
}
