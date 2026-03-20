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

use crate::connector_instance::service::ConnectorInstanceEntitiesService;
use crate::data::entities::{connector_distro_relation, connector_instances, connector_templates};
use crate::data::factory_trait::{ConnectorRepoTrait, MockConnectorRepoTrait};
use crate::data::repo_traits::connector_distro_relation_repo::{
    ConnectorDistroRelationRepoTrait, MockConnectorDistroRelationRepoTrait,
};
use crate::data::repo_traits::connector_instance_repo::MockConnectorInstanceRepoTrait;
use crate::data::repo_traits::connector_template_repo::{
    ConnectorTemplateRepoTrait, MockConnectorTemplateRepoTrait,
};
use crate::entities::connector_template::ConnectorTemplateDto;
use crate::facades::distribution_resolver_facade::{Distribution, MockDistributionFacadeTrait};
use crate::{ConnectorInstanceRepoTrait, ConnectorInstanceTrait, ConnectorInstantiationDto};
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;

fn get_template_fixture_dto() -> ConnectorTemplateDto {
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
    let dto: ConnectorTemplateDto = serde_json::from_value(json_dto).unwrap();
    dto
}

fn get_template_fixture_model() -> connector_templates::Model {
    connector_templates::Model {
        name: "template_name".to_string(),
        version: "1.0".to_string(),
        author: "admin".to_string(),
        created_at: chrono::Utc::now().into(),
        spec: serde_json::to_value(get_template_fixture_dto()).unwrap(),
    }
}

fn get_instance_fixture_model() -> connector_instances::Model {
    connector_instances::Model {
        id: "urn:connector-instance:fake".to_string(),
        template_name: "".to_string(),
        template_version: "".to_string(),
        distribution_id: "".to_string(),
        created_at: Default::default(),
        metadata: Default::default(),
        configuration_parameters: Default::default(),
        authentication: Default::default(),
        interaction: Default::default(),
    }
}

fn mock_service() -> ConnectorInstanceEntitiesService {
    // at some point i'd like to call the template repo
    let mut connector_template_repo = MockConnectorTemplateRepoTrait::new();
    connector_template_repo
        .expect_get_template_by_name_and_version()
        .once()
        .returning(|_, _| Ok(Some(get_template_fixture_model())));
    // at some point i'd like to persist
    let mut connector_instance_repo = MockConnectorInstanceRepoTrait::new();
    connector_instance_repo
        .expect_create_instance()
        .times(1)
        .returning(|model| {
            Ok(connector_instances::Model {
                id: "urn:connector-instance:fake".to_string(),
                template_name: model.template_name.clone(),
                template_version: model.template_version.clone(),
                distribution_id: model.distribution_id.clone(),
                created_at: chrono::Utc::now().into(),
                metadata: model.metadata.clone(),
                configuration_parameters: model.configuration_parameters.clone(),
                authentication: model.authentication.clone(),
                interaction: model.interaction.clone(),
            })
        });
    // at some point
    let mut connector_distro_repo = MockConnectorDistroRelationRepoTrait::new();
    connector_distro_repo
        .expect_get_relation_by_distribution()
        .once()
        .returning(|_| Ok(None));
    connector_distro_repo
        .expect_create_relation()
        .once()
        .returning(|_, _| {
            Ok(connector_distro_relation::Model {
                distribution_id: "".to_string(),
                connector_instance_id: "".to_string(),
            })
        });

    // arcs
    let connector_template_repo: Arc<dyn ConnectorTemplateRepoTrait> =
        Arc::new(connector_template_repo);
    let connector_instance_repo: Arc<dyn ConnectorInstanceRepoTrait> =
        Arc::new(connector_instance_repo);
    let connector_distro_repo: Arc<dyn ConnectorDistroRelationRepoTrait> =
        Arc::new(connector_distro_repo);

    // connector repo
    let mut connector_repo = MockConnectorRepoTrait::new();
    connector_repo
        .expect_get_templates_repo()
        .return_const(connector_template_repo);
    connector_repo
        .expect_get_instances_repo()
        .return_const(connector_instance_repo);
    connector_repo
        .expect_get_distro_relation_repo()
        .return_const(connector_distro_repo);
    let connector_repo = Arc::new(connector_repo);

    let mut distribution_facade = MockDistributionFacadeTrait::new();
    distribution_facade
        .expect_resolve_distribution_by_id()
        .once()
        .with(mockall::predicate::always())
        .returning(|_| {
            Ok(Distribution {
                id: "urn:distribution:faked".to_string(),
                dct_issued: chrono::Utc::now().into(),
                dct_modified: Some(chrono::Utc::now().into()),
                dct_title: Some("distribution_title".to_string()),
                dct_description: Some("distribution_title".to_string()),
                dcat_access_service: "urn:data-service:faked".to_string(),
                dataset_id: "urn:dataset:faked".to_string(),
                dct_format: Some("format_iri".to_string()),
            })
        });
    let distribution_facade = Arc::new(distribution_facade);

    let mut conector_instance = ConnectorInstanceEntitiesService::new(
        connector_repo,
        distribution_facade,
        "http://localhost:8080".to_string(),
    );
    conector_instance
}
#[tokio::test]
async fn test_upsert_instance() {
    let service = mock_service();
    let result = service
        .upsert_instance(&mut ConnectorInstantiationDto {
            template_name: "".to_string(),
            template_version: "".to_string(),
            distribution_id: Urn::from_str("urn:uuid:1").unwrap(),
            parameters: HashMap::from([
                ("ACCESS_URL".to_string(), json!("value")),
                ("ACCESS_METHODS".to_string(), json!(["value", "value"])),
                (
                    "HEADERS".to_string(),
                    json!({
                        "value": "value",
                    }),
                ),
            ]),
            metadata: None,
            dry_run: false,
        })
        .await;
    assert!(result.is_ok());
    let instance = result.unwrap();
}
