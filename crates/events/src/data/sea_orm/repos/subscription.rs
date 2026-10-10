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

//! Subscription repository.

use async_trait::async_trait;
use common::oauth::{Owner, OwnerScope};
use chrono::Utc;
use common::paginated_spec::{Page, Sort};
use common::secret::Secret;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryTrait,
};
use uuid::Uuid;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::repo::EventSubscriptionRepo;
use crate::data::sea_orm::orm::subscription;
use crate::data::sea_orm::repos::listing::NaiveKeyset;
use crate::entities::commands::{CreateSubscriptionDto, UpdateSubscriptionDto};
use crate::entities::queries::SubscriptionFilter;
use crate::entities::subscription::SubscriptionRecord;
use crate::entities::topic::Topic;
use crate::entities::topic_pattern::TopicPattern;

#[derive(Clone)]
pub struct SeaOrmSubscriptionRepo {
    db: DatabaseConnection,
}

impl SeaOrmSubscriptionRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EventSubscriptionRepo for SeaOrmSubscriptionRepo {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn create_subscription(
        &self,
        owner: Owner,
        dto: CreateSubscriptionDto,
    ) -> Outcome<SubscriptionRecord> {
        let id = format!("urn:uuid:{}", Uuid::new_v4());
        let pattern = TopicPattern::new(&dto.topic_pattern)
            .map_err(|e| Errors::format(BadFormat::Received, e, None))?;

        let headers_val = dto
            .headers
            .as_ref()
            .map(|h| serde_json::to_value(h).unwrap_or_default());

        let active = subscription::ActiveModel {
            id: ActiveValue::Set(id.clone()),
            user_id: ActiveValue::Set(owner.user_id.clone()),
            user_role: ActiveValue::Set(owner.role.clone()),
            visibility: ActiveValue::Set(owner.visibility.clone()),
            callback_address: ActiveValue::Set(dto.callback_address.clone()),
            topic_pattern: ActiveValue::Set(dto.topic_pattern),
            secret: ActiveValue::Set(dto.secret.clone().map(Secret::into_exposed)),
            headers: ActiveValue::Set(headers_val),
            retry_limit: ActiveValue::Set(dto.retry_limit.map(|r| r as i32)),
            active: ActiveValue::Set(true),
            created_at: ActiveValue::Set(Utc::now().naive_utc()),
            updated_at: ActiveValue::Set(None),
            expiration_time: ActiveValue::Set(dto.expiration_time.map(|e| e.naive_utc())),
        };

        active
            .insert(&self.db)
            .await
            .map_err(|e| Errors::db("failed to create subscription", Some(Box::new(e))))?;

        Ok(SubscriptionRecord {
            id,
            owner,
            callback_address: dto.callback_address,
            topic_pattern: pattern,
            secret: dto.secret,
            headers: dto.headers,
            retry_limit: dto.retry_limit,
            active: true,
            created_at: Utc::now(),
            updated_at: None,
            expiration_time: dto.expiration_time,
        })
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_subscription(
        &self,
        scope: &OwnerScope,
        id: &str,
    ) -> Outcome<Option<SubscriptionRecord>> {
        let model = subscription::Entity::find()
            .filter(subscription::Column::Id.eq(id))
            .filter(scope.condition(subscription::Column::UserId, subscription::Column::UserRole, subscription::Column::Visibility))
            .one(&self.db)
            .await
            .map_err(|e| Errors::db("failed to query subscription", Some(Box::new(e))))?;

        match model {
            Some(m) => Ok(Some(m.into_domain()?)),
            None => Ok(None),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn list_subscriptions(
        &self,
        scope: &OwnerScope,
        filter: &SubscriptionFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<SubscriptionRecord>, u64)> {
        let query = subscription::Entity::find()
            .filter(scope.condition(subscription::Column::UserId, subscription::Column::UserRole, subscription::Column::Visibility))
            .apply_if(filter.active, |q, a| {
                q.filter(subscription::Column::Active.eq(a))
            });

        let total = query
            .clone()
            .count(&self.db)
            .await
            .map_err(|e| Errors::db("failed to count subscriptions", Some(Box::new(e))))?;
        let models = NaiveKeyset::apply(
            query,
            page,
            sort,
            subscription::Column::CreatedAt,
            subscription::Column::Id,
        )
        .all(&self.db)
        .await
        .map_err(|e| Errors::db("failed to list subscriptions", Some(Box::new(e))))?;

        let results = models
            .into_iter()
            .map(subscription::Model::into_domain)
            .collect::<Outcome<Vec<_>>>()?;
        Ok((results, total))
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn update_subscription(
        &self,
        scope: &OwnerScope,
        id: &str,
        dto: UpdateSubscriptionDto,
    ) -> Outcome<SubscriptionRecord> {
        let model = subscription::Entity::find()
            .filter(subscription::Column::Id.eq(id))
            .filter(scope.condition(subscription::Column::UserId, subscription::Column::UserRole, subscription::Column::Visibility))
            .one(&self.db)
            .await
            .map_err(|e| Errors::db("failed to find subscription", Some(Box::new(e))))?
            .ok_or_else(|| Errors::missing_resource(id, "subscription not found", None))?;

        let mut active: subscription::ActiveModel = model.into();
        if let Some(addr) = dto.callback_address {
            active.callback_address = ActiveValue::Set(addr);
        }
        if let Some(pat) = dto.topic_pattern {
            TopicPattern::new(&pat).map_err(|e| Errors::format(BadFormat::Received, e, None))?;
            active.topic_pattern = ActiveValue::Set(pat);
        }
        if dto.secret.is_some() {
            active.secret = ActiveValue::Set(dto.secret.map(Secret::into_exposed));
        }
        if let Some(headers) = dto.headers {
            active.headers =
                ActiveValue::Set(Some(serde_json::to_value(headers).unwrap_or_default()));
        }
        if dto.retry_limit.is_some() {
            active.retry_limit = ActiveValue::Set(dto.retry_limit.map(|r| r as i32));
        }
        if let Some(act) = dto.active {
            active.active = ActiveValue::Set(act);
        }
        if dto.expiration_time.is_some() {
            active.expiration_time = ActiveValue::Set(dto.expiration_time.map(|e| e.naive_utc()));
        }
        active.updated_at = ActiveValue::Set(Some(Utc::now().naive_utc()));

        let updated = active
            .update(&self.db)
            .await
            .map_err(|e| Errors::db("failed to update subscription", Some(Box::new(e))))?;

        updated.into_domain()
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete_subscription(&self, scope: &OwnerScope, id: &str) -> Outcome<()> {
        subscription::Entity::delete_many()
            .filter(subscription::Column::Id.eq(id))
            .filter(scope.condition(subscription::Column::UserId, subscription::Column::UserRole, subscription::Column::Visibility))
            .exec(&self.db)
            .await
            .map_err(|e| Errors::db("failed to delete subscription", Some(Box::new(e))))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_matching_subscriptions(
        &self,
        owner: &Owner,
        topic: &Topic,
    ) -> Outcome<Vec<SubscriptionRecord>> {
        // Patterns and subscribers live in the rows, so both are matched against each active
        // one here.
        let models = subscription::Entity::find()
            .filter(subscription::Column::Active.eq(true))
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("failed to list subscriptions", Some(Box::new(e))))?;
        let mut matching = Vec::new();
        for model in models {
            let record = model.into_domain()?;
            if record.matches(topic) && OwnerScope::seeing(&record.owner).admits_owner(owner) {
                matching.push(record);
            }
        }
        Ok(matching)
    }
}
