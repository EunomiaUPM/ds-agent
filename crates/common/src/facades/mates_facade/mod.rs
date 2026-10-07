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

//! Participants known by the connector.

use async_trait::async_trait;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

pub mod remote;

#[mockall::automock]
#[async_trait]
/// Participants of the connector, as `user` sees them; tokens live in the grants facade.
pub trait MatesFacadeTrait: Send + Sync {
    /// Participant `mate_id`, if `user` sees it; missing-resource error otherwise.
    async fn get_mate_by_id(&self, user: &UserInfo, mate_id: String) -> Outcome<Mates>;
    /// The connector's own participant record, the same for every user.
    async fn get_me_mate(&self) -> Outcome<Mates>;
    /// Every participant `user` sees.
    async fn get_all_mates(&self, user: &UserInfo) -> Outcome<Vec<Mates>>;
}
