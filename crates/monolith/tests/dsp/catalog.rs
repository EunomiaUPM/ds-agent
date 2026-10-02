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

//! Catalog over DSP: a consumer asks the provider's catalog through its own RPC endpoint.

use serde_json::json;

use crate::support::dataspace::{Dataspace, PROVIDER_DID};

/// A fresh provider has a main catalog and data service, and the consumer reads them over DSP.
#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn consumer_reads_the_provider_catalog() {
    let ds = Dataspace::start().await;

    let main = ds.provider.get("/api/v1/catalog-agent/catalogs/main").await;
    assert!(main["id"].is_string(), "{main}");

    let found = ds
        .consumer
        .post(
            "/dsp/current/catalog/rpc/setup-catalog-request",
            json!({"associatedAgentPeer": PROVIDER_DID, "filter": [], "noCache": true}),
        )
        .await;
    assert!(found["response"]["service"].is_object(), "{found}");
}
