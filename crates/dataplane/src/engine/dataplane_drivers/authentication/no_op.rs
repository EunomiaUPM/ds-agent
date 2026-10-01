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

use crate::engine::dataplane_drivers::DriverAuthenticatorTrait;
use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
use ymir::errors::Outcome;

#[derive(Debug)]
pub struct NoOpAuthenticator;

#[async_trait::async_trait]
impl DriverAuthenticatorTrait for NoOpAuthenticator {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "data-endpoint"))]
    async fn authenticate(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        Ok(context.clone())
    }
}
