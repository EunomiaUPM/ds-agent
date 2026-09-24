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

use async_trait::async_trait;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;

pub mod mates_facade;
pub mod ssi_auth_facade;

#[mockall::automock]
#[async_trait]
/// Resolves a peer's GNAP token to its participant record in whichever tenant holds it.
pub trait SSIAuthFacadeTrait: Send + Sync {
    async fn verify_token(&self, token: String) -> Outcome<Mates>;
}

#[mockall::automock]
#[async_trait]
/// Participants of a tenant, read from ssi-auth with the service token.
pub trait MatesFacadeTrait: Send + Sync {
    async fn get_mate_by_id(&self, tenant_id: String, mate_id: String) -> Outcome<Mates>;
    async fn get_me_mate(&self, tenant_id: String) -> Outcome<Mates>;
    async fn get_all_mates(&self, tenant_id: String) -> Outcome<Vec<Mates>>;
}
