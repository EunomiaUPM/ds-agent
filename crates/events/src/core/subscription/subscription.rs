/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::core::subscription::subscription_err::SubscriptionErrors;
use crate::core::subscription::subscription_types::{
    RainbowEventsSubscriptionCreationRequest, RainbowEventsSubscriptionCreationResponse,
    SubscriptionEntities,
};
use crate::core::subscription::RainbowEventsSubscriptionTrait;
use crate::data::repo::{EditSubscription, EventsRepoFactory, NewSubscription};
use async_trait::async_trait;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome, RepoIntoErrors};

pub struct RainbowEventsSubscriptionService<T> {
    repo: Arc<T>,
}

impl<T> RainbowEventsSubscriptionService<T> {
    pub fn new(repo: Arc<T>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<T> RainbowEventsSubscriptionTrait for RainbowEventsSubscriptionService<T>
where
    T: EventsRepoFactory + Send + Sync + 'static,
{
    async fn get_all_subscriptions(
        &self,
    ) -> Outcome<Vec<RainbowEventsSubscriptionCreationResponse>> {
        let subscriptions = self
            .repo
            .get_all_subscriptions()
            .await
            .map_err(|e| e.into_errors())?;
        let subscriptions = subscriptions
            .iter()
            .map(|sub| RainbowEventsSubscriptionCreationResponse::try_from(sub.to_owned()).unwrap())
            .collect();
        Ok(subscriptions)
    }

    async fn get_subscription_by_id(
        &self,
        subscription_id: Urn,
    ) -> Outcome<RainbowEventsSubscriptionCreationResponse> {
        let subscription = self
            .repo
            .get_subscription_by_id(subscription_id.clone())
            .await
            .map_err(|e| e.into_errors())?
            .ok_or_else(|| {
                Errors::missing_resource(subscription_id.as_str(), "Subscription not found", None)
            })?;
        let subscription = RainbowEventsSubscriptionCreationResponse::try_from(subscription)?;
        Ok(subscription)
    }

    async fn get_subscription_by_callback_url(
        &self,
        callback_url: String,
    ) -> Outcome<RainbowEventsSubscriptionCreationResponse> {
        let subscription = self
            .repo
            .get_subscription_by_callback_string(callback_url)
            .await
            .map_err(|e| e.into_errors())?
            .ok_or_else(|| Errors::missing_resource("unknown", "Subscription not found", None))?;
        let subscription = RainbowEventsSubscriptionCreationResponse::try_from(subscription)?;
        Ok(subscription)
    }

    async fn put_subscription_by_id(
        &self,
        subscription_id: Urn,
        input: RainbowEventsSubscriptionCreationRequest,
    ) -> Outcome<RainbowEventsSubscriptionCreationResponse> {
        let subscription = self
            .repo
            .put_subscription_by_id(
                subscription_id,
                EditSubscription {
                    callback_address: Option::from(input.callback_address),
                    expiration_time: input.expiration_time,
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| e.into_errors())?;
        let subscription = RainbowEventsSubscriptionCreationResponse::try_from(subscription)?;
        Ok(subscription)
    }

    async fn create_subscription(
        &self,
        input: RainbowEventsSubscriptionCreationRequest,
        subscription_type: SubscriptionEntities,
    ) -> Outcome<RainbowEventsSubscriptionCreationResponse> {
        let subscription = self
            .repo
            .get_subscription_by_callback_string(input.callback_address.clone())
            .await
            .map_err(|e| e.into_errors())?;
        if let Some(existing) = subscription {
            return Err(Errors::parse(
                &SubscriptionErrors::SubscriptionCallbackAddressExists(existing.callback_address)
                    .to_string(),
                None,
            ));
        }

        let subscription = self
            .repo
            .create_subscription(NewSubscription {
                callback_address: input.callback_address,
                transfer_process: subscription_type == SubscriptionEntities::TransferProcess,
                contract_negotiation_process: subscription_type
                    == SubscriptionEntities::ContractNegotiationProcess,
                catalog: subscription_type == SubscriptionEntities::Catalog,
                data_plane: subscription_type == SubscriptionEntities::DataPlaneProcess,
                active: true,
                expiration_time: input.expiration_time,
            })
            .await
            .map_err(|e| e.into_errors())?;
        let subscription = RainbowEventsSubscriptionCreationResponse::try_from(subscription)?;
        Ok(subscription)
    }

    async fn delete_subscription_by_id(&self, subscription_id: Urn) -> Outcome<()> {
        let _ = self
            .repo
            .delete_subscription_by_id(subscription_id)
            .await
            .map_err(|e| e.into_errors())?;
        Ok(())
    }
}
