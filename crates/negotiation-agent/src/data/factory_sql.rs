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

use crate::data::factory_trait::NegotiationAgentRepoTrait;
use crate::data::repo_traits::agreement_repo::AgreementRepoTrait;
use crate::data::repo_traits::negotiation_message_repo::NegotiationMessageRepoTrait;
use crate::data::repo_traits::negotiation_process_identifiers_repo::NegotiationIdentifierRepoTrait;
use crate::data::repo_traits::negotiation_process_repo::NegotiationProcessRepoTrait;
use crate::data::repo_traits::offer_repo::OfferRepoTrait;
use crate::data::repos_sql::agreement_repo::AgreementRepoForSql;
use crate::data::repos_sql::negotiation_message_repo::NegotiationMessageRepoForSql;
use crate::data::repos_sql::negotiation_process_identifiers_repo::NegotiationProcessIdentifierRepoForSql;
use crate::data::repos_sql::negotiation_process_repo::NegotiationProcessRepoForSql;
use crate::data::repos_sql::offer_repo::OfferRepoForSql;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct NegotiationAgentRepoForSql {
    negotiation_process_repo: Arc<dyn NegotiationProcessRepoTrait>,
    negotiation_process_identifier_repo: Arc<dyn NegotiationIdentifierRepoTrait>,
    negotiation_message_repo: Arc<dyn NegotiationMessageRepoTrait>,
    offer_repo: Arc<dyn OfferRepoTrait>,
    agreement_repo: Arc<dyn AgreementRepoTrait>,
}

impl NegotiationAgentRepoForSql {
    pub fn create_repo(db_connection: DatabaseConnection) -> Self {
        Self {
            negotiation_process_repo: Arc::new(NegotiationProcessRepoForSql::new(
                db_connection.clone(),
            )),
            negotiation_process_identifier_repo: Arc::new(
                NegotiationProcessIdentifierRepoForSql::new(db_connection.clone()),
            ),
            negotiation_message_repo: Arc::new(NegotiationMessageRepoForSql::new(
                db_connection.clone(),
            )),
            offer_repo: Arc::new(OfferRepoForSql::new(db_connection.clone())),
            agreement_repo: Arc::new(AgreementRepoForSql::new(db_connection.clone())),
        }
    }
}

impl NegotiationAgentRepoTrait for NegotiationAgentRepoForSql {
    fn get_negotiation_process_repo(&self) -> Arc<dyn NegotiationProcessRepoTrait> {
        self.negotiation_process_repo.clone()
    }

    fn get_negotiation_message_repo(&self) -> Arc<dyn NegotiationMessageRepoTrait> {
        self.negotiation_message_repo.clone()
    }

    fn get_negotiation_process_identifiers_repo(&self) -> Arc<dyn NegotiationIdentifierRepoTrait> {
        self.negotiation_process_identifier_repo.clone()
    }

    fn get_offer_repo(&self) -> Arc<dyn OfferRepoTrait> {
        self.offer_repo.clone()
    }

    fn get_agreement_repo(&self) -> Arc<dyn AgreementRepoTrait> {
        self.agreement_repo.clone()
    }
}
