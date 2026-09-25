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

use crate::data::entities::odrl_offer::NewOdrlOfferModel;
use crate::data::entities::{catalog, dataservice, dataset, distribution, odrl_offer};
use crate::data::repo_traits::catalog_db_errors::{
    CatalogAgentRepoErrors, CatalogRepoErrors, DataServiceRepoErrors, DatasetRepoErrors,
    DistributionRepoErrors, OdrlOfferRepoErrors,
};
use crate::data::repo_traits::odrl_offer_repo::OdrlOfferRepositoryTrait;
use crate::entities::filters::OdrlPolicyFilter;
use crate::entities::odrl_policies::CatalogEntityTypes;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use common::query::FilterApplier;
use sea_orm::QueryTrait;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Select,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

impl FilterApplier<Select<odrl_offer::Entity>> for OdrlPolicyFilter {
    fn apply_to(&self, mut q: Select<odrl_offer::Entity>) -> Select<odrl_offer::Entity> {
        if let Some(ref tenant_id) = self.tenant_id {
            q = q.filter(odrl_offer::Column::TenantId.eq(tenant_id));
        }
        if let Some(ref entity) = self.entity {
            q = q.filter(odrl_offer::Column::Entity.eq(entity));
        }
        if let Some(ref entity_type) = self.entity_type {
            q = q.filter(odrl_offer::Column::EntityType.eq(entity_type));
        }
        if let Some(ref source_template_id) = self.source_template_id {
            q = q.filter(odrl_offer::Column::SourceTemplateId.eq(source_template_id));
        }
        if let Some(ref source_template_version) = self.source_template_version {
            q = q.filter(odrl_offer::Column::SourceTemplateVersion.eq(source_template_version));
        }
        if let Some(after) = self.created_after {
            q = q.filter(odrl_offer::Column::CreatedAt.gte(after));
        }
        if let Some(before) = self.created_before {
            q = q.filter(odrl_offer::Column::CreatedAt.lte(before));
        }
        q
    }
}

pub struct OdrlOfferRepositoryForSql {
    db_connection: DatabaseConnection,
}

impl OdrlOfferRepositoryForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl OdrlOfferRepositoryTrait for OdrlOfferRepositoryForSql {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_all_odrl_offers(
        &self,
        filters: &OdrlPolicyFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<odrl_offer::Model>, Option<u64>)> {
        let mut q = odrl_offer::Entity::find();
        q = filters.apply_to(q);

        let total = q.clone().count(&self.db_connection).await.map_err(|err| {
            CatalogAgentRepoErrors::OdrlOfferRepoErrors(
                OdrlOfferRepoErrors::ErrorFetchingOdrlOffer(err.into()),
            )
            .into_errors()
        })?;

        let items = q
            .apply_cursor_pagination_with_tie_break(
                page,
                sort,
                odrl_offer::Column::CreatedAt,
                odrl_offer::Column::Id,
            )
            .all(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::OdrlOfferRepoErrors(
                    OdrlOfferRepoErrors::ErrorFetchingOdrlOffer(err.into()),
                )
                .into_errors()
            })?;

        Ok((items, Some(total)))
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_batch_odrl_offers(
        &self,
        tenant_id: Option<String>,
        ids: &[Urn],
    ) -> Outcome<Vec<odrl_offer::Model>> {
        let odrl_ids = ids.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        let odrl_process = odrl_offer::Entity::find()
            .apply_if(tenant_id, |q, t| {
                q.filter(odrl_offer::Column::TenantId.eq(t))
            })
            .filter(odrl_offer::Column::Id.is_in(odrl_ids))
            .all(&self.db_connection)
            .await;
        match odrl_process {
            Ok(odrl_process) => Ok(odrl_process),
            Err(err) => Err(CatalogAgentRepoErrors::OdrlOfferRepoErrors(
                OdrlOfferRepoErrors::ErrorFetchingOdrlOffer(err.into()),
            )
            .into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_all_odrl_offers_by_entity(
        &self,
        tenant_id: Option<String>,
        entity: &Urn,
    ) -> Outcome<Vec<odrl_offer::Model>> {
        let entity = entity.to_string();
        let odrl_offers = odrl_offer::Entity::find()
            .apply_if(tenant_id, |q, t| {
                q.filter(odrl_offer::Column::TenantId.eq(t))
            })
            .filter(odrl_offer::Column::Entity.eq(entity))
            .all(&self.db_connection)
            .await;
        match odrl_offers {
            Ok(odrl_offers) => Ok(odrl_offers),
            Err(err) => Err(CatalogAgentRepoErrors::OdrlOfferRepoErrors(
                OdrlOfferRepoErrors::ErrorFetchingOdrlOffer(err.into()),
            )
            .into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_odrl_offer_by_id(
        &self,
        tenant_id: Option<String>,
        odrl_offer_id: &Urn,
    ) -> Outcome<Option<odrl_offer::Model>> {
        let odrl_offer_id = odrl_offer_id.to_string();
        let odrl_offer = odrl_offer::Entity::find_by_id(odrl_offer_id)
            .apply_if(tenant_id, |q, t| {
                q.filter(odrl_offer::Column::TenantId.eq(t))
            })
            .one(&self.db_connection)
            .await;
        match odrl_offer {
            Ok(odrl_offer) => Ok(odrl_offer),
            Err(err) => Err(CatalogAgentRepoErrors::OdrlOfferRepoErrors(
                OdrlOfferRepoErrors::ErrorFetchingOdrlOffer(err.into()),
            )
            .into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn create_odrl_offer(
        &self,
        new_odrl_offer_model: &NewOdrlOfferModel,
    ) -> Outcome<odrl_offer::Model> {
        let model: odrl_offer::ActiveModel = new_odrl_offer_model.into();
        let entity_id = new_odrl_offer_model.entity_id.to_string();
        let odrl_offer = match new_odrl_offer_model.entity_type {
            CatalogEntityTypes::Distribution => {
                let _ = distribution::Entity::find_by_id(entity_id)
                    .filter(distribution::Column::TenantId.eq(&new_odrl_offer_model.tenant_id))
                    .one(&self.db_connection)
                    .await
                    .map_err(|err| {
                        CatalogAgentRepoErrors::DistributionRepoErrors(
                            DistributionRepoErrors::ErrorFetchingDistribution(err.into()),
                        )
                        .into_errors()
                    })?
                    .ok_or(
                        CatalogAgentRepoErrors::DistributionRepoErrors(
                            DistributionRepoErrors::DistributionNotFound,
                        )
                        .into_errors(),
                    )?;
                let odrl_offer = odrl_offer::Entity::insert(model)
                    .exec_with_returning(&self.db_connection)
                    .await;
                odrl_offer
            }
            CatalogEntityTypes::DataService => {
                let _ = dataservice::Entity::find_by_id(entity_id)
                    .filter(dataservice::Column::TenantId.eq(&new_odrl_offer_model.tenant_id))
                    .one(&self.db_connection)
                    .await
                    .map_err(|err| {
                        CatalogAgentRepoErrors::DataServiceRepoErrors(
                            DataServiceRepoErrors::ErrorFetchingDataService(err.into()),
                        )
                        .into_errors()
                    })?
                    .ok_or(
                        CatalogAgentRepoErrors::DataServiceRepoErrors(
                            DataServiceRepoErrors::DataServiceNotFound,
                        )
                        .into_errors(),
                    )?;
                let odrl_offer = odrl_offer::Entity::insert(model)
                    .exec_with_returning(&self.db_connection)
                    .await;
                odrl_offer
            }
            CatalogEntityTypes::Catalog => {
                let _ = catalog::Entity::find_by_id(entity_id)
                    .filter(catalog::Column::TenantId.eq(&new_odrl_offer_model.tenant_id))
                    .one(&self.db_connection)
                    .await
                    .map_err(|err| {
                        CatalogAgentRepoErrors::CatalogRepoErrors(
                            CatalogRepoErrors::ErrorFetchingCatalog(err.into()),
                        )
                        .into_errors()
                    })?
                    .ok_or(
                        CatalogAgentRepoErrors::CatalogRepoErrors(
                            CatalogRepoErrors::CatalogNotFound,
                        )
                        .into_errors(),
                    )?;
                let odrl_offer = odrl_offer::Entity::insert(model)
                    .exec_with_returning(&self.db_connection)
                    .await;
                odrl_offer
            }
            CatalogEntityTypes::Dataset => {
                let _ = dataset::Entity::find_by_id(entity_id)
                    .filter(dataset::Column::TenantId.eq(&new_odrl_offer_model.tenant_id))
                    .one(&self.db_connection)
                    .await
                    .map_err(|err| {
                        CatalogAgentRepoErrors::DatasetRepoErrors(
                            DatasetRepoErrors::ErrorFetchingDataset(err.into()),
                        )
                        .into_errors()
                    })?
                    .ok_or(
                        CatalogAgentRepoErrors::DatasetRepoErrors(
                            DatasetRepoErrors::DatasetNotFound,
                        )
                        .into_errors(),
                    )?;
                let odrl_offer = odrl_offer::Entity::insert(model)
                    .exec_with_returning(&self.db_connection)
                    .await;
                odrl_offer
            }
        };

        match odrl_offer {
            Ok(odrl_offer) => Ok(odrl_offer),
            Err(err) => Err(CatalogAgentRepoErrors::OdrlOfferRepoErrors(
                OdrlOfferRepoErrors::ErrorCreatingOdrlOffer(err.into()),
            )
            .into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete_odrl_offer_by_id(
        &self,
        tenant_id: Option<String>,
        odrl_offer_id: &Urn,
    ) -> Outcome<odrl_offer::Model> {
        // Single round-trip: DELETE ... RETURNING, tenant-scoped; empty result means not found.
        let deleted = odrl_offer::Entity::delete_many()
            .filter(odrl_offer::Column::Id.eq(odrl_offer_id.to_string()))
            .apply_if(tenant_id, |q, t| {
                q.filter(odrl_offer::Column::TenantId.eq(t))
            })
            .exec_with_returning(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::OdrlOfferRepoErrors(
                    OdrlOfferRepoErrors::ErrorDeletingOdrlOffer(err.into()),
                )
                .into_errors()
            })?;
        deleted.into_iter().next().ok_or_else(|| {
            CatalogAgentRepoErrors::OdrlOfferRepoErrors(OdrlOfferRepoErrors::OdrlOfferNotFound)
                .into_errors()
        })
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete_odrl_offers_by_entity(
        &self,
        tenant_id: Option<String>,
        entity_id: &Urn,
    ) -> Outcome<Vec<odrl_offer::Model>> {
        // Single round-trip: DELETE ... RETURNING, tenant-scoped; empty result means not found.
        let deleted = odrl_offer::Entity::delete_many()
            .filter(odrl_offer::Column::Entity.eq(entity_id.to_string()))
            .apply_if(tenant_id, |q, t| {
                q.filter(odrl_offer::Column::TenantId.eq(t))
            })
            .exec_with_returning(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::OdrlOfferRepoErrors(
                    OdrlOfferRepoErrors::ErrorDeletingOdrlOffer(err.into()),
                )
                .into_errors()
            })?;
        if deleted.is_empty() {
            return Err(CatalogAgentRepoErrors::OdrlOfferRepoErrors(
                OdrlOfferRepoErrors::OdrlOfferNotFound,
            )
            .into_errors());
        }
        Ok(deleted)
    }
}
