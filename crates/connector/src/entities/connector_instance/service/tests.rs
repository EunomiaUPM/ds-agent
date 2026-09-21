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

use crate::connector_instance::service::ConnectorInstanceEntitiesService;
use crate::data::entities::{connector_distro_relation, connector_instances, connector_templates};
use crate::data::factory_trait::MockConnectorRepoTrait;
use crate::data::repo_traits::connector_distro_relation_repo::{
    ConnectorDistroRelationRepoTrait, MockConnectorDistroRelationRepoTrait,
};
use crate::data::repo_traits::connector_instance_repo::MockConnectorInstanceRepoTrait;
use crate::data::repo_traits::connector_repo_errors::ConnectorInstanceRepoErrors;
use crate::data::repo_traits::connector_template_repo::{
    ConnectorTemplateRepoTrait, MockConnectorTemplateRepoTrait,
};
use crate::entities::connector_template::ConnectorTemplateDto;
use crate::facades::distribution_resolver_facade::MockDistributionFacadeTrait;
use crate::{ConnectorInstanceRepoTrait, ConnectorInstanceTrait, ConnectorInstantiationDto};
use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::RepoIntoErrors;

fn get_template_fixture_dto() -> ConnectorTemplateDto {
    let json_dto = json!({
        "authentication": {
            "type": "BASIC_AUTH",
            "username": "asd",
            "password": {
                "type": "PLAIN",
                "content": "{{__SYS_TOKEN__}}"
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
        tenant_id: "test-tenant".to_string(),
        author: "admin".to_string(),
        created_at: chrono::Utc::now().into(),
        spec: serde_json::to_value(get_template_fixture_dto()).unwrap(),
    }
}

fn mock_service() -> ConnectorInstanceEntitiesService {
    // at some point i'd like to call the template repo
    let mut connector_template_repo = MockConnectorTemplateRepoTrait::new();
    connector_template_repo
        .expect_get_template_by_name_and_version()
        .once()
        .returning(|_, _, _| Ok(Some(get_template_fixture_model())));
    // at some point i'd like to persist
    let mut connector_instance_repo = MockConnectorInstanceRepoTrait::new();
    connector_instance_repo
        .expect_create_instance()
        .times(1)
        .returning(|model| {
            Ok(connector_instances::Model {
                id: "urn:connector-instance:fake".to_string(),
                tenant_id: model.tenant_id.clone(),
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
        .returning(|_, _| Ok(None));
    connector_distro_repo
        .expect_create_relation()
        .once()
        .returning(|_, _, _| {
            Ok(connector_distro_relation::Model {
                distribution_id: "".to_string(),
                tenant_id: "test-tenant".to_string(),
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
        .returning(|_| Ok(()));
    let distribution_facade = Arc::new(distribution_facade);

    let conector_instance = ConnectorInstanceEntitiesService::new(
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
        .upsert_instance(
            &common::auth::AccessScope::system(),
            &mut ConnectorInstantiationDto {
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
                tenant_id: None,
            },
        )
        .await;
    assert!(result.is_ok());
}

// Multi-tenancy isolation tests ──────────────────────────────────────────────

fn tenant_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Owner, &tenant.to_string())
}

fn reader_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Reader, &tenant.to_string())
}

fn build_service(
    template_repo: MockConnectorTemplateRepoTrait,
    instance_repo: MockConnectorInstanceRepoTrait,
    distro_repo: MockConnectorDistroRelationRepoTrait,
    distro_facade: MockDistributionFacadeTrait,
) -> ConnectorInstanceEntitiesService {
    let mut connector_repo = MockConnectorRepoTrait::new();
    connector_repo
        .expect_get_templates_repo()
        .return_const(Arc::new(template_repo) as Arc<dyn ConnectorTemplateRepoTrait>);
    connector_repo
        .expect_get_instances_repo()
        .return_const(Arc::new(instance_repo) as Arc<dyn ConnectorInstanceRepoTrait>);
    connector_repo
        .expect_get_distro_relation_repo()
        .return_const(Arc::new(distro_repo) as Arc<dyn ConnectorDistroRelationRepoTrait>);

    ConnectorInstanceEntitiesService::new(
        Arc::new(connector_repo),
        Arc::new(distro_facade),
        "http://localhost:8080".to_string(),
    )
}

#[tokio::test]
async fn get_instance_foreign_tenant_returns_none() {
    let template_repo = MockConnectorTemplateRepoTrait::new();
    let mut instance_repo = MockConnectorInstanceRepoTrait::new();
    let distro_repo = MockConnectorDistroRelationRepoTrait::new();
    let distro_facade = MockDistributionFacadeTrait::new();

    let urn = Urn::from_str("urn:uuid:f47ac10b-58cc-4372-a567-0e02b2c3d479").unwrap();
    let urn_str = urn.to_string();

    instance_repo
        .expect_get_instance_by_id()
        .withf(move |tenant, id| tenant == "tenant-2" && id == &urn_str)
        .times(1)
        .returning(|_, _| Ok(None));

    let svc = build_service(template_repo, instance_repo, distro_repo, distro_facade);
    let res = svc
        .get_instance_by_id(&tenant_scope("tenant-2"), &urn)
        .await
        .unwrap();
    assert!(res.is_none());
}

#[tokio::test]
async fn get_instance_by_distribution_foreign_tenant_returns_none() {
    let template_repo = MockConnectorTemplateRepoTrait::new();
    let instance_repo = MockConnectorInstanceRepoTrait::new();
    let mut distro_repo = MockConnectorDistroRelationRepoTrait::new();
    let distro_facade = MockDistributionFacadeTrait::new();

    let distro_urn = Urn::from_str("urn:uuid:f47ac10b-58cc-4372-a567-0e02b2c3d479").unwrap();
    let distro_str = distro_urn.to_string();

    distro_repo
        .expect_get_relation_by_distribution()
        .withf(move |tenant, distro| tenant == "tenant-2" && distro == &distro_str)
        .times(1)
        .returning(|_, _| Ok(None));

    let svc = build_service(template_repo, instance_repo, distro_repo, distro_facade);
    let res = svc
        .get_instance_by_distribution(&tenant_scope("tenant-2"), &distro_urn)
        .await
        .unwrap();
    assert!(res.is_none());
}

#[tokio::test]
async fn delete_instance_foreign_tenant_returns_not_found() {
    let template_repo = MockConnectorTemplateRepoTrait::new();
    let mut instance_repo = MockConnectorInstanceRepoTrait::new();
    let mut distro_repo = MockConnectorDistroRelationRepoTrait::new();
    let distro_facade = MockDistributionFacadeTrait::new();

    let urn = Urn::from_str("urn:uuid:f47ac10b-58cc-4372-a567-0e02b2c3d479").unwrap();
    let urn_str = urn.to_string();

    distro_repo
        .expect_delete_relation_by_instance()
        .returning(|_, _| Ok(()));

    instance_repo
        .expect_delete_instance_by_id()
        .withf(move |tenant, id| tenant == "tenant-2" && id == &urn_str)
        .times(1)
        .returning(|_, _| Err(ConnectorInstanceRepoErrors::InstanceNotFound.into_errors()));

    let svc = build_service(template_repo, instance_repo, distro_repo, distro_facade);
    let res = svc
        .delete_instance_by_id(&tenant_scope("tenant-2"), &urn)
        .await;
    assert!(res.is_err());
}

#[tokio::test]
async fn upsert_forces_caller_tenant_for_non_admin() {
    let mut template_repo = MockConnectorTemplateRepoTrait::new();
    template_repo
        .expect_get_template_by_name_and_version()
        .withf(|tenant, name, ver| tenant == "tenant-2" && name == "template_name" && ver == "1.0")
        .times(1)
        .returning(|_, _, _| Ok(Some(get_template_fixture_model())));

    let mut instance_repo = MockConnectorInstanceRepoTrait::new();
    instance_repo
        .expect_create_instance()
        .withf(|m| m.tenant_id == "tenant-2")
        .times(1)
        .returning(|model| {
            Ok(connector_instances::Model {
                id: "urn:connector-instance:fake".to_string(),
                tenant_id: model.tenant_id.clone(),
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

    let mut distro_repo = MockConnectorDistroRelationRepoTrait::new();
    distro_repo
        .expect_get_relation_by_distribution()
        .withf(|tenant, _| tenant == "tenant-2")
        .times(1)
        .returning(|_, _| Ok(None));
    distro_repo
        .expect_create_relation()
        .withf(|tenant, _, _| tenant == "tenant-2")
        .times(1)
        .returning(|_, _, _| {
            Ok(connector_distro_relation::Model {
                distribution_id: "".to_string(),
                tenant_id: "tenant-2".to_string(),
                connector_instance_id: "".to_string(),
            })
        });

    let mut distro_facade = MockDistributionFacadeTrait::new();
    distro_facade
        .expect_resolve_distribution_by_id()
        .times(1)
        .returning(|_| Ok(()));

    let svc = build_service(template_repo, instance_repo, distro_repo, distro_facade);

    let mut dto = ConnectorInstantiationDto {
        template_name: "template_name".to_string(),
        template_version: "1.0".to_string(),
        distribution_id: Urn::from_str("urn:uuid:1").unwrap(),
        parameters: HashMap::from([
            ("ACCESS_URL".to_string(), json!("value")),
            ("ACCESS_METHODS".to_string(), json!(["value", "value"])),
            ("HEADERS".to_string(), json!({ "value": "value" })),
        ]),
        metadata: None,
        dry_run: false,
        tenant_id: Some("tenant-1".to_string()),
    };

    let result = svc
        .upsert_instance(&tenant_scope("tenant-2"), &mut dto)
        .await;
    assert!(result.is_ok());
    assert_eq!(dto.tenant_id.as_deref(), Some("tenant-2"));
}

#[tokio::test]
async fn reader_cannot_upsert_or_delete() {
    let template_repo = MockConnectorTemplateRepoTrait::new();
    let instance_repo = MockConnectorInstanceRepoTrait::new();
    let distro_repo = MockConnectorDistroRelationRepoTrait::new();
    let distro_facade = MockDistributionFacadeTrait::new();

    let svc = build_service(template_repo, instance_repo, distro_repo, distro_facade);
    let reader = reader_scope("tenant-1");

    let mut dto = ConnectorInstantiationDto {
        template_name: "template_name".to_string(),
        template_version: "1.0".to_string(),
        distribution_id: Urn::from_str("urn:uuid:1").unwrap(),
        parameters: HashMap::new(),
        metadata: None,
        dry_run: false,
        tenant_id: None,
    };

    let upsert_res = svc.upsert_instance(&reader, &mut dto).await;
    assert!(upsert_res.is_err());

    let urn = Urn::from_str("urn:uuid:f47ac10b-58cc-4372-a567-0e02b2c3d479").unwrap();
    let delete_res = svc.delete_instance_by_id(&reader, &urn).await;
    assert!(delete_res.is_err());
}
