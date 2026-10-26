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

//! The catalog's published contract served in-process, over its own services.

use std::sync::Arc;

use common::facades::catalog::{CatalogDataset, CatalogDistribution, CatalogFacadeTrait, CatalogOffer};
use urn::Urn;
use ymir::errors::{Errors, Outcome};
use ymir::types::oauth::UserInfo;

use crate::entities::odrl_offer::OdrlOffer;
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::ResourceKind;
use crate::entities::vocab::Vocab;
use crate::setup::context::AppContext;

/// The catalog for the agents and the connector sharing its process.
pub struct CatalogLocalFacade {
    ctx: Arc<AppContext>,
}

impl CatalogLocalFacade {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    /// A missing record keeps its missing-resource error, with `why` as the reason.
    fn not_found<T>(result: Outcome<T>, id: &Urn, why: &str) -> Outcome<T> {
        match result {
            Err(Errors::MissingResourceError { .. }) => Err(Errors::missing_resource(id.to_string(), why, None)),
            other => other,
        }
    }

    fn dataset(resource: CatalogResource) -> CatalogDataset {
        CatalogDataset {
            id: resource.id().to_string(),
            catalog_id: resource.catalog_id().map(ToString::to_string).unwrap_or_default(),
            dct_title: resource.title(),
        }
    }

    fn distribution(resource: CatalogResource) -> CatalogDistribution {
        let description = resource.description();
        CatalogDistribution {
            id: resource.id().to_string(),
            dataset_id: resource.dataset_id().map(ToString::to_string).unwrap_or_default(),
            dct_format: description.first_value(Vocab::DCT_FORMAT),
            dcat_access_service: description
                .first_iri(Vocab::DCAT_ACCESS_SERVICE)
                .unwrap_or_default()
                .to_string(),
        }
    }

    fn offer(offer: OdrlOffer) -> CatalogOffer {
        CatalogOffer {
            id: offer.id.to_string(),
            entity: offer.entity.to_string(),
            entity_type: offer.entity_type,
            odrl_offer: offer.odrl_offer,
        }
    }
}

#[async_trait::async_trait]
impl CatalogFacadeTrait for CatalogLocalFacade {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "catalog"))]
    async fn get_dataset(&self, dataset_id: &Urn) -> Outcome<CatalogDataset> {
        let found = self
            .ctx
            .resources()
            .get(&UserInfo::system(), ResourceKind::Dataset, dataset_id)
            .await;
        Self::not_found(found, dataset_id, "Dataset not found in the catalog").map(Self::dataset)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "catalog"))]
    async fn get_distribution_by_format(&self, dataset_id: &Urn, format: &str) -> Outcome<CatalogDistribution> {
        self.ctx
            .resources()
            .distribution_by_format(&UserInfo::system(), dataset_id, format)
            .await
            .map(Self::distribution)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "catalog", user = %user.id()))]
    async fn get_distribution(&self, user: &UserInfo, distribution_id: &Urn) -> Outcome<CatalogDistribution> {
        let found = self
            .ctx
            .resources()
            .get(user, ResourceKind::Distribution, distribution_id)
            .await;
        let why = "Distribution not found among those the user sees";
        Self::not_found(found, distribution_id, why).map(Self::distribution)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "catalog"))]
    async fn get_offer(&self, offer_id: &Urn) -> Outcome<CatalogOffer> {
        let found = self.ctx.offers().get(&UserInfo::system(), offer_id).await;
        Self::not_found(found, offer_id, "Offer not found in the catalog").map(Self::offer)
    }
}
