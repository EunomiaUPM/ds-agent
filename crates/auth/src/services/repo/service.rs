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

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use ymir::services::repo::postgres::received::{
    RecvGrantPostgresRepo, RecvInteractionPostgresRepo, RecvVerificationPostgresRepo,
};
use ymir::services::repo::postgres::sent::{
    SentGrantPostgresRepo, SentInteractionPostgresRepo, SentVerificationPostgresRepo,
};
use ymir::services::repo::postgres::shared::{ParticipantPostgresRepo, ResourceReqPostgresRepo};
use ymir::services::repo::traits::received::{
    RecvGrantRepoTrait, RecvInteractionRepoTrait, RecvVerificationRepoTrait,
};
use ymir::services::repo::traits::sent::{
    SentGrantRepoTrait, SentInteractionRepoTrait, SentVerificationRepoTrait,
};
use ymir::services::repo::traits::shared::{ParticipantRepoTrait, ResourceReqRepoTrait};

use crate::services::repo::repo_trait::AuthRepoTrait;

pub struct AuthRepoForSql {
    sent_grant_repo: Arc<dyn SentGrantRepoTrait>,
    sent_interaction_repo: Arc<dyn SentInteractionRepoTrait>,
    sent_verification_repo: Arc<dyn SentVerificationRepoTrait>,
    participant_repo: Arc<dyn ParticipantRepoTrait>,
    resource_req_repo: Arc<dyn ResourceReqRepoTrait>,
    recv_grant_repo: Arc<dyn RecvGrantRepoTrait>,
    recv_interaction_repo: Arc<dyn RecvInteractionRepoTrait>,
    recv_verification_repo: Arc<dyn RecvVerificationRepoTrait>,
}

impl AuthRepoForSql {
    pub fn create_repo(db_connection: DatabaseConnection) -> Self {
        Self {
            sent_grant_repo: Arc::new(SentGrantPostgresRepo::new(db_connection.clone())),
            sent_interaction_repo: Arc::new(SentInteractionPostgresRepo::new(
                db_connection.clone(),
            )),
            sent_verification_repo: Arc::new(SentVerificationPostgresRepo::new(
                db_connection.clone(),
            )),
            participant_repo: Arc::new(ParticipantPostgresRepo::new(db_connection.clone())),
            resource_req_repo: Arc::new(ResourceReqPostgresRepo::new(db_connection.clone())),
            recv_grant_repo: Arc::new(RecvGrantPostgresRepo::new(db_connection.clone())),
            recv_verification_repo: Arc::new(RecvVerificationPostgresRepo::new(
                db_connection.clone(),
            )),
            recv_interaction_repo: Arc::new(RecvInteractionPostgresRepo::new(
                db_connection.clone(),
            )),
        }
    }
}

impl AuthRepoTrait for AuthRepoForSql {
    fn sent_grant(&self) -> Arc<dyn SentGrantRepoTrait> {
        self.sent_grant_repo.clone()
    }

    fn sent_interaction(&self) -> Arc<dyn SentInteractionRepoTrait> {
        self.sent_interaction_repo.clone()
    }

    fn sent_verification(&self) -> Arc<dyn SentVerificationRepoTrait> {
        self.sent_verification_repo.clone()
    }

    fn participant(&self) -> Arc<dyn ParticipantRepoTrait> {
        self.participant_repo.clone()
    }

    fn resource_req(&self) -> Arc<dyn ResourceReqRepoTrait> {
        self.resource_req_repo.clone()
    }

    fn recv_grant(&self) -> Arc<dyn RecvGrantRepoTrait> {
        self.recv_grant_repo.clone()
    }

    fn recv_interaction(&self) -> Arc<dyn RecvInteractionRepoTrait> {
        self.recv_interaction_repo.clone()
    }

    fn recv_verification(&self) -> Arc<dyn RecvVerificationRepoTrait> {
        self.recv_verification_repo.clone()
    }
}
