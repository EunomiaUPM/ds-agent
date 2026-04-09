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

use crate::facades::Mates;
use async_trait::async_trait;
use ymir::errors::Outcome;

pub mod mates_facade;
pub mod ssi_auth_facade;

#[mockall::automock]
#[async_trait]
pub trait SSIAuthFacadeTrait: Send + Sync {
    async fn verify_token(&self, token: String) -> Outcome<Mates>;
}

#[mockall::automock]
#[async_trait]
pub trait MatesFacadeTrait: Send + Sync {
    async fn get_mate_by_id(&self, mate_id: String) -> Outcome<Mates>;
    async fn get_mate_by_slug(&self, mate_slug: String) -> Outcome<Mates>;
    async fn get_me_mate(&self) -> Outcome<Mates>;
}
