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

use std::sync::Arc;

use axum::routing::get;
use axum::{Json, Router};
use reqwest::StatusCode;
use urn::UrnBuilder;
use uuid::Uuid;
use ymir::errors::{Errors, Outcome};

use crate::dsp_common::well_known_types::{
    Auth, AuthProtocolTypes, DSPBindings, DSPIdentifierTypes, DSPProtocolVersions, Version,
    VersionResponse,
};

pub mod dspace_version;

pub trait WellKnownDSpaceVersionTrait: Send + Sync + 'static {
    fn dspace_path(&self) -> String;
    fn dspace_service_id(&self) -> String {
        let path = self.dspace_path();
        let deterministic_uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, path.as_bytes());
        UrnBuilder::new("dsp-service-id", deterministic_uuid.to_string().as_str())
            .build()
            .expect("Not able to create Service ID")
            .to_string()
    }

    fn get_dspace_version(&self) -> Outcome<VersionResponse> {
        let protocol_version = VersionResponse {
            protocol_versions: vec![self.get_base_dspace_version()],
        };

        Ok(protocol_version)
    }

    fn get_dspace_version_str(&self, str: &String) -> Outcome<Version> {
        if str != "2025-1" {
            return Err(Errors::crazy("invalid dspace version", None));
        }
        Ok(self.get_base_dspace_version())
    }

    fn get_base_dspace_version(&self) -> Version {
        Version {
            binding: DSPBindings::HTTPS,
            path: self.dspace_path(),
            version: DSPProtocolVersions::V2025_1,
            auth: Some(Auth {
                protocol: AuthProtocolTypes::Gnap,
                version: "1".to_string(),
                profile: None,
            }),
            identifier_type: Some(DSPIdentifierTypes::DidJWK),
            service_id: Option::from(self.dspace_service_id()),
        }
    }

    fn get_router(&self) -> Outcome<Router> {
        let version_response = Arc::new(self.get_dspace_version()?);
        Ok(Router::new().route(
            "/dspace-version",
            get(move || {
                let res = version_response.clone();
                async move { (StatusCode::OK, Json(res)) }
            }),
        ))
    }
}
