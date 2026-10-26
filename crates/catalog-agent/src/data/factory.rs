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

//! Access to the repositories, whatever backs them.

use std::sync::Arc;

use crate::data::repo::odrl_offer::OdrlOfferRepoTrait;
use crate::data::repo::policy_template::PolicyTemplateRepoTrait;
use crate::data::repo::resource::ResourceRepoTrait;

pub trait DataFactory: Send + Sync {
    fn resource_repo(&self) -> Arc<dyn ResourceRepoTrait>;
    fn odrl_offer_repo(&self) -> Arc<dyn OdrlOfferRepoTrait>;
    fn policy_template_repo(&self) -> Arc<dyn PolicyTemplateRepoTrait>;
}
