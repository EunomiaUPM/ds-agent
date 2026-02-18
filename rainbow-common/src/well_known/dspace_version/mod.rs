use crate::dsp_common::well_known_types::{
    Auth, AuthProtocolTypes, DSPBindings, DSPIdentifierTypes, DSPProtocolVersions, Version,
    VersionResponse,
};
use axum::routing::get;
use axum::{Json, Router};
use reqwest::StatusCode;
use std::sync::Arc;
use urn::UrnBuilder;
use uuid::Uuid;

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

    fn get_dspace_version(&self) -> anyhow::Result<VersionResponse> {
        let protocol_version = VersionResponse {
            protocol_versions: vec![Version {
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
            }],
        };

        Ok(protocol_version)
    }

    fn get_dspace_version_str(&self, str: &String) -> anyhow::Result<Version> {
        if str != "2025-1" {
            return Err(anyhow::anyhow!("invalid dspace version"));
        }
        Ok(Version {
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
        })
    }

    fn get_router(&self) -> anyhow::Result<Router> {
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
