/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use ymir::services::repo::postgres::repos::{
    BusinessMatesRepo, IssuingRepo, MatesRepo, RecvInteractionRepo, RecvRequestRepo,
    RecvVerificationRepo, ReqInteractionRepo, ReqRequestRepo, ReqVcRepo, ReqVerificationRepo,
    TokenRequirementsRepo
};
use ymir::services::repo::subtraits::{
    BusinessMatesRepoTrait, IssuingTrait, MatesTrait, RecvInteractionTrait, RecvRequestTrait,
    RecvVerificationTrait, ReqInteractionTrait, ReqRequestTrait, ReqVcTrait, ReqVerificationTrait,
    TokenRequirementsTrait
};

use crate::ssi::services::repo::repo_trait::AuthRepoTrait;

pub struct AuthRepoForSql {
    req_request_repo: Arc<dyn ReqRequestTrait>,
    recv_request_repo: Arc<dyn RecvRequestTrait>,
    req_interaction_repo: Arc<dyn ReqInteractionTrait>,
    recv_interaction_repo: Arc<dyn RecvInteractionTrait>,
    req_verification_repo: Arc<dyn ReqVerificationTrait>,
    recv_verification_repo: Arc<dyn RecvVerificationTrait>,
    req_vc_repo: Arc<dyn ReqVcTrait>,
    token_repo: Arc<dyn TokenRequirementsTrait>,
    mates_repo: Arc<dyn MatesTrait>,
    business_mates: Arc<dyn BusinessMatesRepoTrait>,
    issuing: Arc<dyn IssuingTrait>
}

impl AuthRepoForSql {
    pub fn create_repo(db_connection: DatabaseConnection) -> Self {
        Self {
            req_request_repo: Arc::new(ReqRequestRepo::new(db_connection.clone())),
            recv_request_repo: Arc::new(RecvRequestRepo::new(db_connection.clone())),
            req_interaction_repo: Arc::new(ReqInteractionRepo::new(db_connection.clone())),
            recv_interaction_repo: Arc::new(RecvInteractionRepo::new(db_connection.clone())),
            req_verification_repo: Arc::new(ReqVerificationRepo::new(db_connection.clone())),
            recv_verification_repo: Arc::new(RecvVerificationRepo::new(db_connection.clone())),
            token_repo: Arc::new(TokenRequirementsRepo::new(db_connection.clone())),
            mates_repo: Arc::new(MatesRepo::new(db_connection.clone())),
            req_vc_repo: Arc::new(ReqVcRepo::new(db_connection.clone())),
            business_mates: Arc::new(BusinessMatesRepo::new(db_connection.clone())),
            issuing: Arc::new(IssuingRepo::new(db_connection.clone()))
        }
    }
}

impl AuthRepoTrait for AuthRepoForSql {
    fn request_req(&self) -> Arc<dyn ReqRequestTrait> { self.req_request_repo.clone() }

    fn request_rcv(&self) -> Arc<dyn RecvRequestTrait> { self.recv_request_repo.clone() }

    fn interaction_req(&self) -> Arc<dyn ReqInteractionTrait> { self.req_interaction_repo.clone() }

    fn interaction_rcv(&self) -> Arc<dyn RecvInteractionTrait> {
        self.recv_interaction_repo.clone()
    }

    fn verification_req(&self) -> Arc<dyn ReqVerificationTrait> {
        self.req_verification_repo.clone()
    }

    fn verification_rcv(&self) -> Arc<dyn RecvVerificationTrait> {
        self.recv_verification_repo.clone()
    }

    fn token_requirements(&self) -> Arc<dyn TokenRequirementsTrait> { self.token_repo.clone() }

    fn mates(&self) -> Arc<dyn MatesTrait> { self.mates_repo.clone() }

    fn business_mates(&self) -> Arc<dyn BusinessMatesRepoTrait> { self.business_mates.clone() }

    fn vc_req(&self) -> Arc<dyn ReqVcTrait> { self.req_vc_repo.clone() }

    fn issuing(&self) -> Arc<dyn IssuingTrait> { self.issuing.clone() }
}
