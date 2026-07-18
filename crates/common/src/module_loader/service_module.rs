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

use axum::Router;
use sea_orm_migration::MigrationTrait;
use tonic::service::RoutesBuilder;

/// One composable slice of an agent. Every hook has a no-op default:
/// implement only the planes the module actually has.
pub trait ServiceModuleTrait<Ctx>: Send + Sync {
    /// Stable identifier, used for logging/diagnostics.
    fn name(&self) -> &'static str;

    /// The DB migrations this module owns. Context-free by design (see
    /// module docs). Order within the module is preserved.
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        vec![]
    }

    /// The module's HTTP surface: absolute mount prefix + router.
    /// `None` when the module has no HTTP plane.
    fn http(&self, _ctx: &Ctx) -> Option<(String, Router)> {
        None
    }

    /// Register the module's gRPC services into the shared builder.
    /// Default: contributes nothing.
    fn grpc(&self, _ctx: &Ctx, _routes: &mut RoutesBuilder) {}
}
