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

//! Resources in Postgres. Filters on the description run as `jsonb` containment over the GIN
//! index; a resource and its links are written in one transaction.

use std::sync::Arc;

use common::oauth::OwnerScope;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use sea_orm::sea_query::{Condition, Expr};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Select, SqlErr, TransactionTrait,
};
use serde_json::json;
use urn::Urn;
use ymir::errors::{Errors, Outcome, RepoIntoErrors};

use crate::data::repo::resource::{ResourceRepoErrors, ResourceRepoTrait};
use crate::data::sea_orm::orm::resource as orm;
use crate::data::sea_orm::orm::resource_link as links;
use crate::entities::errors::CatalogErrors;
use crate::entities::filters::ResourceFilter;
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::ResourceKind;
use crate::entities::vocab::Vocab;

pub struct SeaOrmResourceRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmResourceRepo {
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

    fn filtered(scope: &OwnerScope, filter: &ResourceFilter) -> Select<orm::Entity> {
        let mut q = Self::scoped(scope);
        if let Some(kind) = filter.kind {
            q = q.filter(orm::Column::Kind.eq(kind.as_str()));
        }
        if let Some(user_id) = &filter.user_id {
            q = q.filter(orm::Column::UserId.eq(user_id.as_str()));
        }
        if let Some(catalog_id) = &filter.catalog_id {
            q = q.filter(orm::Column::CatalogId.eq(catalog_id.as_str()));
        }
        if let Some(dataset_id) = &filter.dataset_id {
            q = q.filter(orm::Column::DatasetId.eq(dataset_id.as_str()));
        }
        if let Some(title) = &filter.title {
            q = q.filter(Expr::cust_with_values(
                "catalog_resources.title ILIKE $1",
                [format!("%{}%", Self::like_escaped(title))],
            ));
        }
        if let Some(keyword) = &filter.keyword {
            q = q.filter(Self::contains_literal(Vocab::DCAT_KEYWORD, keyword));
        }
        let iri_filters = [
            (Vocab::DCAT_THEME, &filter.theme),
            (Vocab::DCT_CREATOR, &filter.creator),
            (Vocab::DCT_PUBLISHER, &filter.publisher),
            (Vocab::DCT_CONFORMS_TO, &filter.conforms_to),
            (Vocab::DSPACE_PARTICIPANT_ID, &filter.participant_id),
            (Vocab::DCAT_ACCESS_SERVICE, &filter.access_service),
            (Vocab::DCAT_ENDPOINT_URL, &filter.endpoint_url),
        ];
        for (predicate, value) in iri_filters {
            if let Some(value) = value {
                q = q.filter(
                    Condition::any()
                        .add(Self::contains_iri(predicate, value))
                        .add(Self::contains_literal(predicate, value)),
                );
            }
        }
        if let Some(format) = &filter.format {
            q = q.filter(Self::format_is(format));
        }
        if let Some(main) = filter.main {
            q = q.filter(orm::Column::IsMain.eq(main));
        }
        if let Some(after) = filter.created_after {
            q = q.filter(orm::Column::Issued.gte(after));
        }
        if let Some(before) = filter.created_before {
            q = q.filter(orm::Column::Issued.lte(before));
        }
        q
    }

    /// `dct:format` is a literal (`http+pull`) or an IRI (a file-type concept).
    fn format_is(format: &str) -> Condition {
        Condition::any()
            .add(Self::contains_literal(Vocab::DCT_FORMAT, format))
            .add(Self::contains_iri(Vocab::DCT_FORMAT, format))
    }

    fn contains_literal(predicate: &str, value: &str) -> sea_orm::sea_query::SimpleExpr {
        Expr::cust_with_values(
            "catalog_resources.description @> $1",
            [json!({ predicate: [{ "@value": value }] })],
        )
    }

    fn contains_iri(predicate: &str, iri: &str) -> sea_orm::sea_query::SimpleExpr {
        Expr::cust_with_values(
            "catalog_resources.description @> $1",
            [json!({ predicate: [{ "@id": iri }] })],
        )
    }

    fn like_escaped(text: &str) -> String {
        text.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
    }

    fn decode(models: Vec<orm::Model>) -> Outcome<Vec<CatalogResource>> {
        models.into_iter().map(orm::Model::into_domain).collect()
    }

    fn fetch_err(e: DbErr) -> Errors {
        ResourceRepoErrors::Fetching(Box::new(e)).into_errors()
    }

    /// Key violations are the caller's to fix (409); anything else is ours.
    fn write_err(e: DbErr, fallback: fn(Box<dyn std::error::Error + Send + Sync>) -> ResourceRepoErrors) -> Errors {
        match e.sql_err() {
            Some(SqlErr::ForeignKeyConstraintViolation(detail)) => CatalogErrors::conflict(format!(
                "the resource links to one that does not exist, or one that is linked cannot go: {detail}"
            )),
            Some(SqlErr::UniqueConstraintViolation(detail)) => {
                CatalogErrors::conflict(format!("the resource exists already: {detail}"))
            }
            _ => fallback(Box::new(e)).into_errors(),
        }
    }

    async fn insert_links<C: sea_orm::ConnectionTrait>(
        db: &C,
        resource: &CatalogResource,
    ) -> Result<(), DbErr> {
        let rows = links::ActiveModel::from_domain(resource);
        if rows.is_empty() {
            return Ok(());
        }
        links::Entity::insert_many(rows).exec(db).await.map(|_| ())
    }
}

#[async_trait::async_trait]
impl ResourceRepoTrait for SeaOrmResourceRepo {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_page(
        &self,
        scope: &OwnerScope,
        filter: &ResourceFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<CatalogResource>> {
        let models = Self::filtered(scope, filter)
            .apply_cursor_pagination_with_tie_break(page, sort, orm::Column::Issued, orm::Column::Id)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Self::decode(models)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn count(&self, scope: &OwnerScope, filter: &ResourceFilter) -> Outcome<u64> {
        Self::filtered(scope, filter)
            .count(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_all(&self, scope: &OwnerScope, filter: &ResourceFilter) -> Outcome<Vec<CatalogResource>> {
        let models = Self::filtered(scope, filter)
            .order_by_asc(orm::Column::Issued)
            .order_by_asc(orm::Column::Id)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Self::decode(models)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_batch(&self, scope: &OwnerScope, ids: &[Urn]) -> Outcome<Vec<CatalogResource>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let ids: Vec<String> = ids.iter().map(ToString::to_string).collect();
        let models = Self::scoped(scope)
            .filter(orm::Column::Id.is_in(ids))
            .order_by_asc(orm::Column::Issued)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Self::decode(models)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_by_id(&self, scope: &OwnerScope, id: &Urn) -> Outcome<Option<CatalogResource>> {
        Self::scoped(scope)
            .filter(orm::Column::Id.eq(id.to_string()))
            .one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .map(orm::Model::into_domain)
            .transpose()
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_main(&self, kind: ResourceKind) -> Outcome<Option<CatalogResource>> {
        orm::Entity::find()
            .filter(orm::Column::Kind.eq(kind.as_str()))
            .filter(orm::Column::IsMain.eq(true))
            .one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .map(orm::Model::into_domain)
            .transpose()
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_children(
        &self,
        scope: &OwnerScope,
        kind: Option<ResourceKind>,
        parents: &[Urn],
    ) -> Outcome<Vec<CatalogResource>> {
        if parents.is_empty() {
            return Ok(vec![]);
        }
        let parents: Vec<String> = parents.iter().map(ToString::to_string).collect();
        let mut q = Self::scoped(scope).filter(
            Condition::any()
                .add(orm::Column::CatalogId.is_in(parents.clone()))
                .add(orm::Column::DatasetId.is_in(parents)),
        );
        if let Some(kind) = kind {
            q = q.filter(orm::Column::Kind.eq(kind.as_str()));
        }
        let models = q
            .order_by_asc(orm::Column::Issued)
            .order_by_asc(orm::Column::Id)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Self::decode(models)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_distribution_by_format(
        &self,
        scope: &OwnerScope,
        dataset_id: &Urn,
        format: &str,
    ) -> Outcome<Option<CatalogResource>> {
        Self::scoped(scope)
            .filter(orm::Column::Kind.eq(ResourceKind::Distribution.as_str()))
            .filter(orm::Column::DatasetId.eq(dataset_id.to_string()))
            .filter(Self::format_is(format))
            .order_by_asc(orm::Column::Issued)
            .one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .map(orm::Model::into_domain)
            .transpose()
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn insert(&self, resource: &CatalogResource) -> Outcome<CatalogResource> {
        let txn = self.db.begin().await.map_err(Self::fetch_err)?;
        let model = orm::Entity::insert(orm::ActiveModel::from_domain(resource))
            .exec_with_returning(&txn)
            .await
            .map_err(|e| Self::write_err(e, ResourceRepoErrors::Creating))?;
        Self::insert_links(&txn, resource)
            .await
            .map_err(|e| Self::write_err(e, ResourceRepoErrors::Creating))?;
        txn.commit()
            .await
            .map_err(|e| ResourceRepoErrors::Creating(Box::new(e)).into_errors())?;
        model.into_domain()
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn replace(
        &self,
        scope: &OwnerScope,
        resource: &CatalogResource,
        expected: i64,
    ) -> Outcome<CatalogResource> {
        let txn = self.db.begin().await.map_err(Self::fetch_err)?;
        let id = resource.id().to_string();
        let updated = orm::Entity::update_many()
            .set(orm::ActiveModel::from_domain(resource))
            .filter(orm::Column::Id.eq(id.as_str()))
            .filter(orm::Column::Version.eq(expected))
            .filter(scope.condition(
                orm::Column::UserId,
                orm::Column::UserRole,
                orm::Column::Visibility,
            ))
            .exec(&txn)
            .await
            .map_err(|e| Self::write_err(e, ResourceRepoErrors::Updating))?;
        if updated.rows_affected == 0 {
            return Err(CatalogErrors::stale(format!(
                "{id} is no longer at version {expected}"
            )));
        }
        links::Entity::delete_many()
            .filter(links::Column::Subject.eq(id.as_str()))
            .exec(&txn)
            .await
            .map_err(|e| Self::write_err(e, ResourceRepoErrors::Updating))?;
        Self::insert_links(&txn, resource)
            .await
            .map_err(|e| Self::write_err(e, ResourceRepoErrors::Updating))?;
        txn.commit()
            .await
            .map_err(|e| ResourceRepoErrors::Updating(Box::new(e)).into_errors())?;
        Ok(resource.clone())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete(&self, scope: &OwnerScope, id: &Urn) -> Outcome<CatalogResource> {
        let deleted = orm::Entity::delete_many()
            .filter(orm::Column::Id.eq(id.to_string()))
            .filter(scope.condition(
                orm::Column::UserId,
                orm::Column::UserRole,
                orm::Column::Visibility,
            ))
            .exec_with_returning(self.db.as_ref())
            .await
            .map_err(|e| Self::write_err(e, ResourceRepoErrors::Deleting))?;
        deleted
            .into_iter()
            .next()
            .ok_or_else(|| ResourceRepoErrors::NotFound.into_errors())?
            .into_domain()
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn referrers(&self, id: &Urn) -> Outcome<Vec<(String, String)>> {
        Ok(links::Entity::find()
            .filter(links::Column::Object.eq(id.to_string()))
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .into_iter()
            .map(|link| (link.subject, link.predicate))
            .collect())
    }
}
