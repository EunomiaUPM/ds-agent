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

//! One contract, two adapters: the in-process facade and the HTTP facade must answer alike.

use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use common::auth::{AccessScope, Claims, OauthTokenValidator, RbacRole, ServiceHttpClient};
use common::config::types::min_known_config::MinKnownConfig;
use connector::{
    AuthenticationConfig, ConnectorInstanceDto, ConnectorInstanceFacadeTrait,
    ConnectorInstanceLocalFacade, ConnectorInstanceRemoteFacade, ConnectorInstanceServiceTrait,
    ConnectorInstantiationDto, ConnectorMetadata, HttpSpec, InteractionConfig, ProtocolSpec,
    PullLifecycle, TemplateVecString,
};
use urn::Urn;
use ymir::errors::{Errors, Outcome};

const TENANT_A: &str = "tenant-a";
const TENANT_B: &str = "tenant-b";

fn instance_urn() -> Urn {
    Urn::from_str("urn:connector-instance:1").unwrap()
}

fn distribution_urn() -> Urn {
    Urn::from_str("urn:distribution:1").unwrap()
}

fn instance() -> ConnectorInstanceDto {
    ConnectorInstanceDto {
        id: instance_urn(),
        metadata: ConnectorMetadata {
            name: Some("contract".to_string()),
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication_config: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        distribution_id: distribution_urn(),
    }
}

/// One instance owned by `TENANT_A`; reads honour the caller's tenant filter.
struct FakeInstances {
    owner: String,
    instance: ConnectorInstanceDto,
}

impl FakeInstances {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            owner: TENANT_A.to_string(),
            instance: instance(),
        })
    }

    fn visible(&self, scope: &AccessScope, found: bool) -> Option<ConnectorInstanceDto> {
        (found && scope.permits(&self.owner)).then(|| self.instance.clone())
    }
}

#[async_trait::async_trait]
impl ConnectorInstanceServiceTrait for FakeInstances {
    async fn get_instance_by_id(
        &self,
        scope: &AccessScope,
        id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        Ok(self.visible(scope, *id == self.instance.id))
    }

    async fn get_instance_by_distribution(
        &self,
        scope: &AccessScope,
        distribution_id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        Ok(self.visible(scope, *distribution_id == self.instance.distribution_id))
    }

    async fn upsert_instance(
        &self,
        _scope: &AccessScope,
        _dto: &mut ConnectorInstantiationDto,
    ) -> Outcome<ConnectorInstanceDto> {
        unimplemented!("not part of the facade contract")
    }

    async fn delete_instance_by_id(&self, _scope: &AccessScope, _id: &Urn) -> Outcome<()> {
        unimplemented!("not part of the facade contract")
    }
}

/// Every bearer is the service token: Admin of the `admin` tenant.
struct ServiceTokenValidator;

#[async_trait::async_trait]
impl OauthTokenValidator for ServiceTokenValidator {
    async fn validate_token(&self, _token: &str) -> Outcome<Claims> {
        Ok(Claims {
            sub: "admin".to_string(),
            role: RbacRole::Admin,
            iat: 0,
            exp: i64::MAX,
        })
    }
}

fn found_or_404(found: Outcome<Option<ConnectorInstanceDto>>) -> Response {
    match found {
        Ok(Some(instance)) => Json(instance).into_response(),
        Ok(None) => {
            Errors::missing_resource("instance", "Instance not found", None).into_response()
        }
        Err(e) => e.into_response(),
    }
}

async fn by_id(
    State(svc): State<Arc<FakeInstances>>,
    scope: AccessScope,
    Path(id): Path<String>,
) -> Response {
    found_or_404(
        svc.get_instance_by_id(&scope, &Urn::from_str(&id).unwrap())
            .await,
    )
}

async fn by_distribution(
    State(svc): State<Arc<FakeInstances>>,
    scope: AccessScope,
    Path(id): Path<String>,
) -> Response {
    found_or_404(
        svc.get_instance_by_distribution(&scope, &Urn::from_str(&id).unwrap())
            .await,
    )
}

/// Serves the catalog's connector API (real auth middleware and scope extractor) plus a
/// token endpoint, and returns the remote facade pointed at it.
async fn remote_facade(svc: Arc<FakeInstances>) -> ConnectorInstanceRemoteFacade {
    let validator: Arc<dyn OauthTokenValidator> = Arc::new(ServiceTokenValidator);
    let instances = Router::new()
        .route("/{id}", get(by_id))
        .route("/distribution/{id}", get(by_distribution))
        .route_layer(axum::middleware::from_fn_with_state(
            validator,
            common::auth::http::AuthHttpMiddleware::run,
        ))
        .with_state(svc);
    let token = post(|| async {
        Json(serde_json::json!({ "access_token": "service-token", "expires_in": 3600 }))
    });
    let app = Router::new()
        .nest("/api/v1/connector/instances", instances)
        .route("/oauth/token", token);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let catalog: MinKnownConfig = serde_json::from_value(serde_json::json!({
        "hosts": {
            "http": { "protocol": "http", "url": "127.0.0.1", "port": port.to_string(), "internal_port": null },
            "grpc": null, "graphql": null
        },
        "api_version": "v1",
        "service_client": {
            "client_id": "svc", "client_secret": "svc",
            "token_url": format!("http://127.0.0.1:{port}/oauth/token")
        }
    }))
    .unwrap();
    let client = Arc::new(ServiceHttpClient::new(&catalog.service_client, ""));
    ConnectorInstanceRemoteFacade::new(&catalog, client)
}

async fn assert_contract(facade: &dyn ConnectorInstanceFacadeTrait) {
    let found = facade
        .get_instance_by_id(TENANT_A, &instance_urn())
        .await
        .unwrap();
    assert_eq!(found.map(|i| i.id), Some(instance_urn()));

    let by_dist = facade
        .get_instance_by_distribution(TENANT_A, &distribution_urn())
        .await
        .unwrap();
    assert_eq!(by_dist.map(|i| i.id), Some(instance_urn()));

    // A foreign tenant's instance looks exactly like a missing one.
    let foreign = facade
        .get_instance_by_id(TENANT_B, &instance_urn())
        .await
        .unwrap();
    assert!(foreign.is_none());

    let missing = Urn::from_str("urn:connector-instance:missing").unwrap();
    assert!(facade
        .get_instance_by_id(TENANT_A, &missing)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn local_facade_honours_the_contract() {
    assert_contract(&ConnectorInstanceLocalFacade::new(FakeInstances::new())).await;
}

#[tokio::test]
async fn remote_facade_honours_the_contract() {
    assert_contract(&remote_facade(FakeInstances::new()).await).await;
}
