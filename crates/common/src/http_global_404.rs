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

use crate::errors::CommonErrors;
use axum::response::IntoResponse;

pub async fn global_handler_404(uri: axum::http::Uri) -> impl IntoResponse {
    tracing::info!("404 Not Found: {}", uri);
    CommonErrors::missing_resource_new(&uri.to_string(), "Route not found or Method not allowed")
        .into_response()
}
