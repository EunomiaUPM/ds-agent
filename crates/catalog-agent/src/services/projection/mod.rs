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

//! A resource in one profile, as its owner sees it: JSON-LD or any RDF syntax, cut to a number
//! of datasets for a preview.

pub mod service;
pub mod views;

use common::rdf::RdfSyntax;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

use crate::entities::resource_kind::ResourceKind;
use crate::services::projection::views::Projection;

/// What a projection is asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionRequest {
    pub profile: String,
    /// Datasets and series kept; none keeps them all.
    pub limit: Option<usize>,
    pub syntax: RdfSyntax,
}

/// Projecting stored resources through the registered profiles.
#[mockall::automock]
#[async_trait::async_trait]
pub trait ProjectionServiceTrait: Send + Sync {
    /// `id`, of `kind`, in the requested profile and syntax; 404 when the caller does not see it.
    async fn project(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        id: &Urn,
        request: &ProjectionRequest,
    ) -> Outcome<Projection>;
}
