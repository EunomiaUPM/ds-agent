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

use crate::data::repo_traits::agreement_repo::AgreementRepoTrait;
use crate::data::repo_traits::negotiation_message_repo::NegotiationMessageRepoTrait;
use crate::data::repo_traits::negotiation_process_identifiers_repo::NegotiationIdentifierRepoTrait;
use crate::data::repo_traits::negotiation_process_repo::NegotiationProcessRepoTrait;
use crate::data::repo_traits::offer_repo::OfferRepoTrait;
use std::sync::Arc;

#[mockall::automock]
pub trait NegotiationAgentRepoTrait: Send + Sync + 'static {
    fn get_negotiation_process_repo(&self) -> Arc<dyn NegotiationProcessRepoTrait>;
    fn get_negotiation_message_repo(&self) -> Arc<dyn NegotiationMessageRepoTrait>;
    fn get_negotiation_process_identifiers_repo(&self) -> Arc<dyn NegotiationIdentifierRepoTrait>;
    fn get_offer_repo(&self) -> Arc<dyn OfferRepoTrait>;
    fn get_agreement_repo(&self) -> Arc<dyn AgreementRepoTrait>;
}
