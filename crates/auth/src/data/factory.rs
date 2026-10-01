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

use ymir::services::repo::traits::received::{
    RecvGrantRepoTrait, RecvInteractionRepoTrait, RecvVerificationRepoTrait,
};
use ymir::services::repo::traits::sent::{
    SentGrantRepoTrait, SentInteractionRepoTrait, SentVerificationRepoTrait,
};
use ymir::services::repo::traits::shared::{ParticipantRepoTrait, ResourceReqRepoTrait};

pub trait AuthRepoTrait: Send + Sync + 'static {
    fn sent_grant(&self) -> Arc<dyn SentGrantRepoTrait>;
    fn sent_interaction(&self) -> Arc<dyn SentInteractionRepoTrait>;
    fn sent_verification(&self) -> Arc<dyn SentVerificationRepoTrait>;
    fn participant(&self) -> Arc<dyn ParticipantRepoTrait>;
    fn resource_req(&self) -> Arc<dyn ResourceReqRepoTrait>;
    fn recv_grant(&self) -> Arc<dyn RecvGrantRepoTrait>;
    fn recv_interaction(&self) -> Arc<dyn RecvInteractionRepoTrait>;
    fn recv_verification(&self) -> Arc<dyn RecvVerificationRepoTrait>;
}
