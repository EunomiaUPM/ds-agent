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

//! Offers in Postgres; an offer and the templates it was composed from are written together.

use std::sync::Arc;

use common::oauth::OwnerScope;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Select, SqlErr, TransactionTrait,
};
use urn::Urn;
use ymir::errors::{Errors, Outcome, RepoIntoErrors};

use crate::data::repo::odrl_offer::{OdrlOfferRepoErrors, OdrlOfferRepoTrait};
use crate::data::sea_orm::orm::odrl_offer as orm;
use crate::data::sea_orm::orm::offer_ref as refs;
use crate::entities::errors::CatalogErrors;
use crate::entities::filters::OdrlPolicyFilter;
use crate::entities::odrl_offer::OdrlOffer;

pub struct SeaOrmOdrlOfferRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmOdrlOfferRepo {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn scoped(scope: &OwnerScope) -> Select<orm::Entity> {
        orm::Entity::find().filter(scope.condition(
            orm::Column::UserId,
            orm::Column::UserRole,
            orm::Column::Visibility,
        ))
    }

    fn filtered(scope: &OwnerScope, filter: &OdrlPolicyFilter) -> Select<orm::Entity> {
        let mut q = Self::scoped(scope);
        if let Some(user_id) = &filter.user_id {
            q = q.filter(orm::Column::UserId.eq(user_id.as_str()));
        }
        if let Some(entity) = &filter.entity {
            q = q.filter(orm::Column::Entity.eq(entity.as_str()));
        }
        if let Some(entity_type) = &filter.entity_type {
            q = q.filter(orm::Column::EntityType.eq(entity_type.as_str()));
        }
        if let Some(template) = &filter.source_template_id {
            q = q.filter(orm::Column::SourceTemplateId.eq(template.as_str()));
        }
        if let Some(version) = &filter.source_template_version {
            q = q.filter(orm::Column::SourceTemplateVersion.eq(version.as_str()));
        }
        if let Some(after) = filter.created_after {
            q = q.filter(orm::Column::CreatedAt.gte(after));
        }
        if let Some(before) = filter.created_before {
            q = q.filter(orm::Column::CreatedAt.lte(before));
        }
        q
    }

    fn decode(models: Vec<orm::Model>) -> Outcome<Vec<OdrlOffer>> {
        models.into_iter().map(orm::Model::into_domain).collect()
    }

    fn fetch_err(e: DbErr) -> Errors {
        OdrlOfferRepoErrors::Fetching(Box::new(e)).into_errors()
    }
}

#[async_trait::async_trait]
impl OdrlOfferRepoTrait for SeaOrmOdrlOfferRepo {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_page(
        &self,
        scope: &OwnerScope,
        filter: &OdrlPolicyFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<OdrlOffer>> {
        let models = Self::filtered(scope, filter)
            .apply_cursor_pagination_with_tie_break(page, sort, orm::Column::CreatedAt, orm::Column::Id)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Self::decode(models)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn count(&self, scope: &OwnerScope, filter: &OdrlPolicyFilter) -> Outcome<u64> {
        Self::filtered(scope, filter)
            .count(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_batch(&self, scope: &OwnerScope, ids: &[Urn]) -> Outcome<Vec<OdrlOffer>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let ids: Vec<String> = ids.iter().map(ToString::to_string).collect();
        let models = Self::scoped(scope)
            .filter(orm::Column::Id.is_in(ids))
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Self::decode(models)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_by_entities(&self, scope: &OwnerScope, entities: &[Urn]) -> Outcome<Vec<OdrlOffer>> {
        if entities.is_empty() {
            return Ok(vec![]);
        }
        let entities: Vec<String> = entities.iter().map(ToString::to_string).collect();
        let models = Self::scoped(scope)
            .filter(orm::Column::Entity.is_in(entities))
            .order_by_asc(orm::Column::CreatedAt)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Self::decode(models)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_by_id(&self, scope: &OwnerScope, id: &Urn) -> Outcome<Option<OdrlOffer>> {
        Self::scoped(scope)
            .filter(orm::Column::Id.eq(id.to_string()))
            .one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .map(orm::Model::into_domain)
            .transpose()
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn insert(&self, offer: &OdrlOffer, references: &[(String, String)]) -> Outcome<OdrlOffer> {
        let create_err = |e: DbErr| match e.sql_err() {
            Some(SqlErr::ForeignKeyConstraintViolation(detail)) => CatalogErrors::conflict(
                format!("the offer is on a resource or template that does not exist: {detail}"),
            ),
            _ => OdrlOfferRepoErrors::Creating(Box::new(e)).into_errors(),
        };
        let txn = self.db.begin().await.map_err(create_err)?;
        let stored = orm::Entity::insert(orm::ActiveModel::from_domain(offer))
            .exec_with_returning(&txn)
            .await
            .map_err(create_err)?
            .into_domain()?;
        if !references.is_empty() {
            let rows = references.iter().map(|(id, version)| refs::ActiveModel {
                offer_id: sea_orm::ActiveValue::Set(stored.id.to_string()),
                ref_id: sea_orm::ActiveValue::Set(id.clone()),
                ref_version: sea_orm::ActiveValue::Set(version.clone()),
            });
            refs::Entity::insert_many(rows)
                .on_conflict_do_nothing()
                .exec(&txn)
                .await
                .map_err(create_err)?;
        }
        txn.commit().await.map_err(create_err)?;
        Ok(stored)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn using_template(&self, scope: &OwnerScope, id: &str, version: &str) -> Outcome<Vec<Urn>> {
        let offer_ids: Vec<String> = refs::Entity::find()
            .select_only()
            .column(refs::Column::OfferId)
            .filter(refs::Column::RefId.eq(id))
            .filter(refs::Column::RefVersion.eq(version))
            .into_tuple()
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        if offer_ids.is_empty() {
            return Ok(vec![]);
        }
        let models = Self::scoped(scope)
            .filter(orm::Column::Id.is_in(offer_ids))
            .order_by_asc(orm::Column::CreatedAt)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Ok(Self::decode(models)?.into_iter().map(|o| o.id).collect())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete(&self, scope: &OwnerScope, id: &Urn) -> Outcome<OdrlOffer> {
        orm::Entity::delete_many()
            .filter(orm::Column::Id.eq(id.to_string()))
            .filter(scope.condition(
                orm::Column::UserId,
                orm::Column::UserRole,
                orm::Column::Visibility,
            ))
            .exec_with_returning(self.db.as_ref())
            .await
            .map_err(|e| OdrlOfferRepoErrors::Deleting(Box::new(e)).into_errors())?
            .into_iter()
            .next()
            .ok_or_else(|| OdrlOfferRepoErrors::NotFound.into_errors())?
            .into_domain()
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete_by_entity(&self, scope: &OwnerScope, entity: &Urn) -> Outcome<Vec<OdrlOffer>> {
        let models = orm::Entity::delete_many()
            .filter(orm::Column::Entity.eq(entity.to_string()))
            .filter(scope.condition(
                orm::Column::UserId,
                orm::Column::UserRole,
                orm::Column::Visibility,
            ))
            .exec_with_returning(self.db.as_ref())
            .await
            .map_err(|e| OdrlOfferRepoErrors::Deleting(Box::new(e)).into_errors())?;
        Self::decode(models)
    }
}
