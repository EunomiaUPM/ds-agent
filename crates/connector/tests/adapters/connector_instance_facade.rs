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

//! ConnectorInstanceFacadeTrait: the in-process facade and the HTTP facade answer alike,
//! against a fake service behind the real auth middleware.

use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use common::oauth::{
    FixedUserValidator, OauthTokenValidatorTrait, OwnerScope, RolePath, UserInfo, Visibility,
};
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

fn instance_urn() -> Urn {
    Urn::from_str("urn:connector-instance:1").unwrap()
}

fn distribution_urn() -> Urn {
    Urn::from_str("urn:distribution:1").unwrap()
}

fn instance() -> ConnectorInstanceDto {
    ConnectorInstanceDto {
        id: instance_urn(),
        user_id: "user-1".to_string(),
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

/// One private instance of `TENANT_A`; reads honour what the caller sees.
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

    fn visible(&self, user: &UserInfo, found: bool) -> Option<ConnectorInstanceDto> {
        let role: RolePath = "/admin/a".parse().unwrap();
        let seen = OwnerScope::seeing(user).admits(&self.owner, &role, &Visibility::Private);
        (found && seen).then(|| self.instance.clone())
    }
}

#[async_trait::async_trait]
impl ConnectorInstanceServiceTrait for FakeInstances {
    async fn get_instance_by_id(
        &self,
        user: &UserInfo,
        id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        Ok(self.visible(user, *id == self.instance.id))
    }

    async fn get_instance_by_distribution(
        &self,
        user: &UserInfo,
        distribution_id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        Ok(self.visible(user, *distribution_id == self.instance.distribution_id))
    }

    async fn upsert_instance(
        &self,
        _user: &UserInfo,
        _dto: &mut ConnectorInstantiationDto,
    ) -> Outcome<ConnectorInstanceDto> {
        unimplemented!("not part of the facade contract")
    }

    async fn delete_instance_by_id(&self, _user: &UserInfo, _id: &Urn) -> Outcome<()> {
        unimplemented!("not part of the facade contract")
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
    user: UserInfo,
    Path(id): Path<String>,
) -> Response {
    found_or_404(
        svc.get_instance_by_id(&user, &Urn::from_str(&id).unwrap())
            .await,
    )
}

async fn by_distribution(
    State(svc): State<Arc<FakeInstances>>,
    user: UserInfo,
    Path(id): Path<String>,
) -> Response {
    found_or_404(
        svc.get_instance_by_distribution(&user, &Urn::from_str(&id).unwrap())
            .await,
    )
}

/// Serves the catalog's connector API (real auth middleware and user extractor, with the static
/// identity provider acting as the root) and returns the remote facade pointed at it.
async fn remote_facade(svc: Arc<FakeInstances>) -> ConnectorInstanceRemoteFacade {
    let validator: Arc<dyn OauthTokenValidatorTrait> =
        Arc::new(FixedUserValidator::new(UserInfo::system()));
    let instances = Router::new()
        .route("/{id}", get(by_id))
        .route("/distribution/{id}", get(by_distribution))
        .route_layer(axum::middleware::from_fn_with_state(
            validator,
            ymir::http::OauthHttpMiddleware::run,
        ))
        .with_state(svc);
    let app = Router::new().nest("/api/v1/connector/instances", instances);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let catalog: MinKnownConfig = serde_json::from_value(serde_json::json!({
        "hosts": {
            "http": { "protocol": "http", "url": "127.0.0.1", "port": port.to_string(), "internal_port": null },
            "grpc": null, "graphql": null
        },
        "api_version": "v1"
    }))
    .unwrap();
    ConnectorInstanceRemoteFacade::new(&catalog)
}

/// Finds by id and by distribution, and a missing instance is `None`.
async fn assert_contract(facade: &dyn ConnectorInstanceFacadeTrait) {
    let found = facade
        .get_instance_by_id(&instance_urn())
        .await
        .unwrap();
    assert_eq!(found.map(|i| i.id), Some(instance_urn()));

    let by_dist = facade
        .get_instance_by_distribution(&distribution_urn())
        .await
        .unwrap();
    assert_eq!(by_dist.map(|i| i.id), Some(instance_urn()));

    let missing = Urn::from_str("urn:connector-instance:missing").unwrap();
    assert!(facade
        .get_instance_by_id(&missing)
        .await
        .unwrap()
        .is_none());
}

/// The local facade finds by id and distribution whoever owns the instance: in-process flows
/// already know which one they want.
#[tokio::test]
async fn local_facade_honours_the_contract() {
    let facade = ConnectorInstanceLocalFacade::new(FakeInstances::new());
    assert_contract(&facade).await;
}

/// The HTTP facade gives the same answers as the local one.
#[tokio::test]
async fn remote_facade_honours_the_contract() {
    assert_contract(&remote_facade(FakeInstances::new()).await).await;
}
