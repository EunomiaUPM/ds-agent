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

pub mod remote;

#[mockall::automock]
#[async_trait]
/// Participants of the connector, read from ssi-auth with the service token.
pub trait MatesFacadeTrait: Send + Sync {
    /// Participant `mate_id`.
    async fn get_mate_by_id(&self, mate_id: String) -> Outcome<Mates>;
    /// The connector's own participant record.
    async fn get_me_mate(&self) -> Outcome<Mates>;
    /// Every participant the connector knows.
    async fn get_all_mates(&self) -> Outcome<Vec<Mates>>;
}
