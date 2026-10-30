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

//! The catalog contract over the catalog agent's management API.

use std::str::FromStr;

use async_trait::async_trait;
use axum::http::StatusCode;
use urn::Urn;
use ymir::config::types::HostType;
use ymir::errors::{BadFormat, Errors, Outcome, PetitionFailure};
use ymir::services::client::ClientExt;
use ymir::types::oauth::UserInfo;
use ymir::utils::http_client;

use serde_json::Value;

use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::MinKnownConfigTrait;
use crate::dsp_common::odrl::OdrlPolicyInfo;
use crate::eunomia::document::JsonLd;
use crate::eunomia::EunomiaProfile;
use crate::facades::catalog::{
    CatalogDataset, CatalogDistribution, CatalogFacadeTrait, CatalogOffer,
};

/// Reads the catalog agent's API. Calls carry no user token yet, as for the auth facades.
/// Datasets and distributions come as JSON-LD in the eunomia catalog context and are read through
/// RDF; offers come as ODRL JSON-LD and are handed over without their context, as stored.
pub struct CatalogRemoteFacade {
    base_url: String,
}

impl CatalogRemoteFacade {
    /// Path segment the catalog agent mounts its API under.
    const SERVICE: &'static str = "catalog-agent";
    const DCT: &'static str = "http://purl.org/dc/terms/";
    const DCAT: &'static str = "http://www.w3.org/ns/dcat#";
    const PUBLICATION_STATE: &'static str = "https://w3id.org/eunomia/v1/publicationState";

    pub fn new(catalog: &MinKnownConfig) -> Self {
        Self {
            base_url: format!(
                "{}{}/{}",
                catalog.get_host(HostType::Http),
                catalog.get_api_version(),
                Self::SERVICE
            ),
        }
    }

    /// GETs `path`; a 404 becomes the missing-resource error the local adapter returns.
    async fn get<T: serde::de::DeserializeOwned + Send>(
        &self,
        path: &str,
        what: &str,
    ) -> Outcome<T> {
        let url = format!("{}/{path}", self.base_url);
        match http_client().get_json(&url, None).await {
            Err(Errors::PetitionError {
                failure: PetitionFailure::HttpStatus(StatusCode::NOT_FOUND),
                ..
            }) => Err(Errors::missing_resource(
                path,
                format!("{what} not found in the catalog"),
                None,
            )),
            other => other,
        }
    }
}

impl CatalogRemoteFacade {
    /// The resource document at `path`, expanded: its one node.
    async fn node(&self, path: &str, what: &str) -> Outcome<Value> {
        let document: Value = self.get(path, what).await?;
        let expansion = EunomiaProfile::shared().expand(&document).await?;
        expansion
            .expanded
            .as_array()
            .and_then(|nodes| nodes.first())
            .cloned()
            .ok_or_else(|| {
                Errors::missing_resource(path, format!("{what} not found in the catalog"), None)
            })
    }

    /// The first value of `predicate`, IRI or literal.
    fn first(node: &Value, predicate: &str) -> Option<String> {
        let value = node.get(predicate)?.as_array()?.first()?;
        value
            .get("@id")
            .or_else(|| value.get("@value"))
            .and_then(Value::as_str)
            .map(str::to_string)
    }

    /// The parent a reverse membership link names.
    fn parent(node: &Value, membership: &str) -> String {
        node.get("@reverse")
            .and_then(|r| r.get(membership))
            .and_then(Value::as_array)
            .and_then(|links| links.first())
            .and_then(|l| l.get("@id"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    }

    fn id(node: &Value) -> String {
        node.get("@id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    }

    fn distribution(node: &Value) -> CatalogDistribution {
        CatalogDistribution {
            id: Self::id(node),
            dataset_id: Self::parent(node, &format!("{}distribution", Self::DCAT)),
            dct_format: Self::first(node, &format!("{}format", Self::DCT)),
            dcat_access_service: Self::first(node, &format!("{}accessService", Self::DCAT))
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl CatalogFacadeTrait for CatalogRemoteFacade {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "catalog"))]
    async fn get_dataset(&self, dataset_id: &Urn) -> Outcome<CatalogDataset> {
        let node = self
            .node(&format!("datasets/{dataset_id}"), "Dataset")
            .await?;
        Ok(CatalogDataset {
            id: Self::id(&node),
            catalog_id: Self::parent(&node, &format!("{}dataset", Self::DCAT)),
            dct_title: Self::first(&node, &format!("{}title", Self::DCT)),
            draft: Self::first(&node, Self::PUBLICATION_STATE).as_deref() == Some("draft"),
        })
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "catalog"))]
    async fn get_distribution_by_format(
        &self,
        dataset_id: &Urn,
        format: &str,
    ) -> Outcome<CatalogDistribution> {
        let path = format!("distributions/dataset/{dataset_id}/format/{format}");
        Ok(Self::distribution(&self.node(&path, "Distribution").await?))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "catalog"))]
    async fn get_distribution(
        &self,
        _user: &UserInfo,
        distribution_id: &Urn,
    ) -> Outcome<CatalogDistribution> {
        let node = self
            .node(&format!("distributions/{distribution_id}"), "Distribution")
            .await?;
        Ok(Self::distribution(&node))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "catalog"))]
    async fn get_offer(&self, offer_id: &Urn) -> Outcome<CatalogOffer> {
        let mut offer: CatalogOffer = self
            .get(&format!("odrl-policies/{offer_id}"), "Offer")
            .await?;
        offer.odrl_offer = JsonLd::<OdrlPolicyInfo>::untag(offer.odrl_offer)
            .map_err(|reason| Errors::format(BadFormat::Received, reason, None))?;
        if offer.entity_type == "Dataset" {
            let dataset = Urn::from_str(&offer.entity)
                .map_err(|_| Errors::format(BadFormat::Received, "offer entity is no URN", None))?;
            if self.get_dataset(&dataset).await?.draft {
                return Err(Errors::missing_resource(
                    offer_id.to_string(),
                    "the offer's dataset is a draft",
                    None,
                ));
            }
        }
        Ok(offer)
    }
}
