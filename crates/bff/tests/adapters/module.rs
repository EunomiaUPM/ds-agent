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

//! BffModule as a ServiceModuleTrait: name, mount point and the routes it serves.

use std::sync::Arc;

use axum::http::StatusCode;
use bff::{AppContext, BffModule};
use common::module_loader::service_module::ServiceModuleTrait;
use ymir::services::client::ClientTrait;
use ymir::utils::http_client;

use crate::support::fixtures::{gateway_config, serve};

/// The module is named `gateway`, mounts at the root and serves the frontend config.
#[tokio::test]
async fn module_mounts_at_root_and_serves_frontend_config() {
    let app_ctx = Arc::new(AppContext::new(gateway_config(8080), None));
    let module = BffModule::new(app_ctx);

    assert_eq!(module.name(), "gateway");
    let (prefix, router) = module.http().expect("http routes present");
    assert_eq!(prefix, "");
    let port = serve(router).await;

    let resp = http_client()
        .get(
            &format!("http://127.0.0.1:{port}/admin/api/fe-config"),
            None,
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
