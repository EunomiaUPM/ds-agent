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

use crate::data::entities::offer;
use crate::data::entities::offer::{Model, NewOfferModel};
use crate::data::repo_traits::offer_repo::{OfferRepoErrors, OfferRepoTrait};
use crate::entities::filters::OfferFilter;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use common::query::FilterApplier;
use sea_orm::QueryTrait;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Select,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

impl FilterApplier<Select<offer::Entity>> for OfferFilter {
    fn apply_to(&self, mut q: Select<offer::Entity>) -> Select<offer::Entity> {
        if let Some(ref id) = self.id {
            q = q.filter(offer::Column::Id.eq(id));
        }
        if let Some(ref tenant_id) = self.tenant_id {
            q = q.filter(offer::Column::TenantId.eq(tenant_id));
        }
        if let Some(ref process_id) = self.process_id {
            q = q.filter(offer::Column::NegotiationAgentProcessId.eq(process_id));
        }
        if let Some(ref offer_id) = self.offer_id {
            q = q.filter(offer::Column::OfferId.eq(offer_id));
        }
        if let Some(after) = self.created_after {
            q = q.filter(offer::Column::CreatedAt.gte(after));
        }
        if let Some(before) = self.created_before {
            q = q.filter(offer::Column::CreatedAt.lte(before));
        }
        q
    }
}

pub struct OfferRepoForSql {
    db_connection: DatabaseConnection,
}

impl OfferRepoForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl OfferRepoTrait for OfferRepoForSql {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_all_offers(
        &self,
        filters: &OfferFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<Model>, Option<u64>)> {
        let q = filters.apply_to(offer::Entity::find());

        let total = q
            .clone()
            .count(&self.db_connection)
            .await
            .map_err(|e| OfferRepoErrors::ErrorFetchingOffer(e.into()).into_errors())?;

        let items = q
            .apply_cursor_pagination_with_tie_break(
                page,
                sort,
                offer::Column::CreatedAt,
                offer::Column::Id,
            )
            .all(&self.db_connection)
            .await
            .map_err(|e| OfferRepoErrors::ErrorFetchingOffer(e.into()).into_errors())?;

        Ok((items, Some(total)))
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_batch_offers(
        &self,
        tenant_id: Option<String>,
        ids: &[Urn],
    ) -> Outcome<Vec<Model>> {
        let offer_ids = ids.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        let offers = offer::Entity::find()
            .apply_if(tenant_id, |q, t| q.filter(offer::Column::TenantId.eq(t)))
            .filter(offer::Column::Id.is_in(offer_ids))
            .all(&self.db_connection)
            .await;

        match offers {
            Ok(offers) => Ok(offers),
            Err(e) => Err(OfferRepoErrors::ErrorFetchingOffer(e.into()).into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_offers_by_negotiation_process(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Vec<Model>> {
        let pid = id.to_string();
        let offers = offer::Entity::find()
            .apply_if(tenant_id, |q, t| q.filter(offer::Column::TenantId.eq(t)))
            .filter(offer::Column::NegotiationAgentProcessId.eq(pid))
            .order_by_asc(offer::Column::CreatedAt)
            .all(&self.db_connection)
            .await;

        match offers {
            Ok(offers) => Ok(offers),
            Err(e) => Err(OfferRepoErrors::ErrorFetchingOffer(e.into()).into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_last_offer_by_negotiation_process(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<Model>> {
        let pid = id.to_string();
        let offers = offer::Entity::find()
            .apply_if(tenant_id, |q, t| q.filter(offer::Column::TenantId.eq(t)))
            .filter(offer::Column::NegotiationAgentProcessId.eq(pid))
            .order_by_desc(offer::Column::CreatedAt)
            .one(&self.db_connection)
            .await;

        match offers {
            Ok(offers) => Ok(offers),
            Err(e) => Err(OfferRepoErrors::ErrorFetchingOffer(e.into()).into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_offer_by_id(&self, tenant_id: Option<String>, id: &Urn) -> Outcome<Option<Model>> {
        let oid = id.to_string();
        let offer = offer::Entity::find_by_id(oid)
            .apply_if(tenant_id, |q, t| q.filter(offer::Column::TenantId.eq(t)))
            .one(&self.db_connection)
            .await;

        match offer {
            Ok(offer) => Ok(offer),
            Err(e) => Err(OfferRepoErrors::ErrorFetchingOffer(e.into()).into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_offer_by_negotiation_message(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<Model>> {
        let mid = id.to_string();
        let offer = offer::Entity::find()
            .apply_if(tenant_id, |q, t| q.filter(offer::Column::TenantId.eq(t)))
            .filter(offer::Column::NegotiationAgentMessageId.eq(mid))
            .one(&self.db_connection)
            .await;

        match offer {
            Ok(offer) => Ok(offer),
            Err(e) => Err(OfferRepoErrors::ErrorFetchingOffer(e.into()).into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_offer_by_offer_id(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<Model>> {
        let external_offer_id = id.to_string();
        let offer = offer::Entity::find()
            .apply_if(tenant_id, |q, t| q.filter(offer::Column::TenantId.eq(t)))
            .filter(offer::Column::OfferId.eq(external_offer_id))
            .one(&self.db_connection)
            .await;

        match offer {
            Ok(offer) => Ok(offer),
            Err(e) => Err(OfferRepoErrors::ErrorFetchingOffer(e.into()).into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn create_offer(&self, new_model: &NewOfferModel) -> Outcome<Model> {
        let model: offer::ActiveModel = new_model.clone().into();
        let result = offer::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;

        match result {
            Ok(offer) => Ok(offer),
            Err(e) => Err(OfferRepoErrors::ErrorCreatingOffer(e.into()).into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete_offer(&self, tenant_id: Option<String>, id: &Urn) -> Outcome<String> {
        let oid = id.to_string();
        let result = offer::Entity::delete_many()
            .filter(offer::Column::Id.eq(&oid))
            .apply_if(tenant_id, |q, t| q.filter(offer::Column::TenantId.eq(t)))
            .exec_with_returning(&self.db_connection)
            .await;

        match result {
            Ok(rows) => rows
                .into_iter()
                .next()
                .map(|row| row.tenant_id)
                .ok_or_else(|| OfferRepoErrors::OfferNotFound.into_errors()),
            Err(e) => Err(OfferRepoErrors::ErrorDeletingOffer(e.into()).into_errors()),
        }
    }
}
