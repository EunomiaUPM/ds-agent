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

//! Offering service over the resource and offer services.

use std::sync::Arc;

use serde_json::json;
use tracing::warn;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::types::oauth::UserInfo;
use ymir::types::participants::Visibility;

use crate::entities::admin_document::AdminDocument;
use crate::entities::commands::{NewOdrlOfferCommand, NewResourceCommand};
use crate::entities::dataset_offering::{NewDatasetOfferingCommand, PolicyOfferingInput};
use crate::entities::graph::ResourceGraph;
use crate::entities::odrl_offer::{CatalogEntityTypes, OdrlOffer};
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::ResourceKind;
use crate::entities::vocab::Vocab;
use crate::services::dataset_offerings::{DatasetOffering, DatasetOfferingServiceTrait};
use crate::setup::context::AppContext;

pub struct DatasetOfferingService {
    ctx: Arc<AppContext>,
}

impl DatasetOfferingService {
    const ODRL_USE: &'static str = "http://www.w3.org/ns/odrl/2/use";
    const ODRL_PROFILE: &'static str = "http://www.w3.org/ns/odrl/2/";

    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    /// The document as a resource of `kind`, or a malformed body.
    async fn read(body: &serde_json::Value, kind: ResourceKind) -> Outcome<AdminDocument> {
        let document = AdminDocument::read(body.clone()).await?;
        if document.kind != kind {
            return Err(Errors::format(
                BadFormat::Received,
                format!("the {} of an offering must be a {}", kind.term(), kind.term()),
                None,
            ));
        }
        Ok(document)
    }

    /// The main resource of `kind`, which an offering falls back to.
    async fn main(&self, kind: ResourceKind) -> Outcome<Urn> {
        self.ctx
            .resources()
            .main(kind)
            .await?
            .map(|r| r.id().clone())
            .ok_or_else(|| {
                Errors::missing_resource(
                    "main",
                    format!("the connector has no main {}", kind.term()),
                    None,
                )
            })
    }

    async fn create_rest(
        &self,
        user: &UserInfo,
        offering: &NewDatasetOfferingCommand,
        dataset: &CatalogResource,
    ) -> Outcome<(CatalogResource, Option<OdrlOffer>)> {
        let mut document = Self::read(&offering.distribution, ResourceKind::Distribution).await?;
        if document.parent.as_ref().is_some_and(|p| p != dataset.id()) {
            return Err(Errors::format(
                BadFormat::Received,
                "the distribution of an offering belongs to its dataset",
                None,
            ));
        }
        let served = [Vocab::DCAT_ACCESS_SERVICE, Vocab::DCAT_ACCESS_URL, Vocab::DCAT_DOWNLOAD_URL];
        if !served.iter().any(|p| document.description.has(p)) {
            let service = self.main(ResourceKind::DataService).await?;
            document
                .description
                .put(Vocab::DCAT_ACCESS_SERVICE, vec![ResourceGraph::iri(service.as_ref())]);
        }
        let distribution = self
            .ctx
            .resources()
            .create(
                user,
                &NewResourceCommand {
                    kind: ResourceKind::Distribution,
                    id: document.id,
                    owner: Some(dataset.owner().clone()),
                    visibility: Some(dataset.owner().visibility.clone()),
                    parent: Some(dataset.id().clone()),
                    description: document.description,
                },
            )
            .await?;
        let policy = match &offering.policy {
            Some(policy) => Some(
                self.ctx
                    .offers()
                    .create(user, &Self::policy_command(policy, dataset)?)
                    .await?,
            ),
            None => None,
        };
        Ok((distribution, policy))
    }

    fn policy_command(policy: &PolicyOfferingInput, dataset: &CatalogResource) -> Outcome<NewOdrlOfferCommand> {
        let offer = json!({
            "profile": [policy.profile.as_deref().unwrap_or(Self::ODRL_PROFILE)],
            "permission": [{
                "action": policy.action.as_deref().unwrap_or(Self::ODRL_USE),
                "constraint": policy.constraints.clone().unwrap_or_default(),
            }],
        });
        Ok(NewOdrlOfferCommand {
            id: None,
            visibility: Some(dataset.owner().visibility.clone()),
            owner: Some(dataset.owner().clone()),
            odrl_offer: serde_json::from_value(offer)
                .map_err(|e| Errors::format(BadFormat::Received, e.to_string(), None))?,
            entity_id: dataset.id().clone(),
            entity_type: CatalogEntityTypes::Dataset,
            source_template_id: None,
            source_template_version: None,
            instantiation_parameters: None,
            description: Some(
                policy
                    .description
                    .clone()
                    .unwrap_or_else(|| "Dataset Usage Policy".to_string()),
            ),
        })
    }
}

#[async_trait::async_trait]
impl DatasetOfferingServiceTrait for DatasetOfferingService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create_offering(
        &self,
        user: &UserInfo,
        offering: &NewDatasetOfferingCommand,
        visibility: Option<Visibility>,
    ) -> Outcome<DatasetOffering> {
        let document = Self::read(&offering.dataset, ResourceKind::Dataset).await?;
        let catalog = match document.parent {
            Some(catalog) => catalog,
            None => self.main(ResourceKind::Catalog).await?,
        };
        let dataset = self
            .ctx
            .resources()
            .create(
                user,
                &NewResourceCommand {
                    kind: ResourceKind::Dataset,
                    id: document.id,
                    owner: None,
                    visibility,
                    parent: Some(catalog),
                    description: document.description,
                },
            )
            .await?;
        match self.create_rest(user, offering, &dataset).await {
            Ok((distribution, policy)) => Ok(DatasetOffering {
                dataset,
                distribution,
                policy,
            }),
            Err(e) => {
                let removed = self
                    .ctx
                    .resources()
                    .delete(user, ResourceKind::Dataset, dataset.id(), None)
                    .await;
                if let Err(cleanup) = removed {
                    warn!(dataset = %dataset.id(), error = %cleanup, "Could not remove partial offering");
                }
                Err(e)
            }
        }
    }
}
