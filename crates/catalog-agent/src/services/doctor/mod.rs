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

//! Whether a stored resource conforms to each registered profile, and why not.

pub mod service;
pub mod views;

use urn::Urn;
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

use crate::entities::resource_kind::ResourceKind;
use crate::services::doctor::views::DoctorReport;

/// Diagnosing resources against the profiles the composition registered.
#[mockall::automock]
#[async_trait::async_trait]
pub trait DoctorServiceTrait: Send + Sync {
    /// Every profile, or only `profile`; `deep` checks the whole closure, not the resource alone.
    async fn diagnose(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        id: &Urn,
        profile: Option<String>,
        deep: bool,
    ) -> Outcome<DoctorReport>;
}
