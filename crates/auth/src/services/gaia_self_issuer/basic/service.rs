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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::http::header::{ACCEPT, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue};
use chrono::{Duration, Utc};
use common::config::types::traits::{EntityClientTrait, GaiaConfigTrait};
use jsonwebtoken::{Algorithm, Header};
use serde_json::{json, Value};
use tracing::info;
use uuid::Uuid;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::issuing;
use ymir::errors::{Errors, Outcome, PetitionFailure};
use ymir::services::client::ClientTrait;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;
use ymir::types::http::Body;
use ymir::types::issuing::{GiveVC, IssuingToken};
use ymir::types::secrets::StringHelper;
use ymir::types::vcs::claims_v1::{VCClaimsV1, VCFromClaimsV1};
use ymir::types::vcs::claims_v2::VCClaimsV2;
use ymir::types::vcs::vc_issuer::VCIssuer;
use ymir::types::vcs::vc_specs::legal_person::LegalPersonCredentialSubject;
use ymir::types::vcs::vc_specs::terms_and_conds::TermsAndConditionsCredSub;
use ymir::types::vcs::{GaiaVP, VPDef, VcInsideGaiaVPBuilder, VcType, W3cDataModelVersion};
use ymir::types::wallet::MatchingVCs;
use ymir::utils::{expect_from_env, get_rsa_key, parse_to_value, sign_token, ResponseExt};

use super::super::GaiaOwnIssuerTrait;
use super::config::{GaiaGaiaSelfIssuerConfigTrait, GaiaSelfIssuerConfig};

pub struct BasicGaiaSelfIssuer {
    vault: Arc<VaultService>,
    client: Arc<dyn ClientTrait>,
    config: GaiaSelfIssuerConfig,
}

impl BasicGaiaSelfIssuer {
    pub fn new(
        vault: Arc<VaultService>,
        client: Arc<dyn ClientTrait>,
        config: GaiaSelfIssuerConfig,
    ) -> BasicGaiaSelfIssuer {
        BasicGaiaSelfIssuer {
            vault,
            client,
            config,
        }
    }
}

#[async_trait]
impl GaiaOwnIssuerTrait for BasicGaiaSelfIssuer {
    fn start_basic_vcs(&self) -> issuing::NewModel {
        info!("Starting retrieving basic gaia vcs");
        let id = uuid::Uuid::new_v4().to_string();
        let host = self.config.get_host(HostType::Http);
        let aud = match self.config.is_local() {
            true => host.replace("127.0.0.1", "host.docker.internal"),
            false => host,
        };

        let vc_type = format!("{}&{}", VcType::LegalPerson, VcType::TermsAndConditions);
        issuing::NewModel {
            id,
            name: self.config.get_clas_id().to_string(),
            vc_type,
            aud,
        }
    }

    fn get_token(&self) -> IssuingToken {
        info!("Giving token");
        IssuingToken::default()
    }

    fn get_did(&self) -> String {
        self.config.get_did().to_string()
    }

    async fn issue_cred(&self, did: &str, vc_type: &VcType, code: &str) -> Outcome<Value> {
        info!("Issuing cred");

        let legal_id = format!("urn:uuid:{}", uuid::Uuid::new_v4().to_string());
        let terms_id = format!("urn:uuid:{}", uuid::Uuid::new_v4().to_string());

        let legal_subj =
            parse_to_value(&LegalPersonCredentialSubject::new4gaia(did, vc_type, code)?)?;
        let terms_subj = parse_to_value(&TermsAndConditionsCredSub::random())?;

        let person_vc = self.build_vc(did, &legal_id, &VcType::LegalPerson, legal_subj)?;
        let terms_vc = self.build_vc(did, &terms_id, &VcType::TermsAndConditions, terms_subj)?;

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(did.to_string());

        let key = expect_from_env("VAULT_APP_PRIV_KEY");
        let key: StringHelper = self.vault.read(None, &key).await?;
        let key = get_rsa_key(key.data())?;

        let person_vc_jwt = sign_token(&header, &person_vc, &key)?;
        let terms_vc_jwt = sign_token(&header, &terms_vc, &key)?;

        Ok(json!({
            "credential_responses": vec![
                GiveVC { format: "jwt_vc_json".to_string(), credential: person_vc_jwt },
                GiveVC { format: "jwt_vc_json".to_string(), credential: terms_vc_jwt }
            ]
        }))
    }

    fn build_vc(&self, did: &str, id: &str, vc_type: &VcType, subject: Value) -> Outcome<Value> {
        let now = Utc::now();
        let issuer = VCIssuer {
            id: did.to_string(),
            name: None,
        };
        let context_v1 = vec!["https://www.w3.org/ns/credentials/v1".to_string()];
        let context_v2 = vec!["https://www.w3.org/ns/credentials/v2".to_string()];
        let types = vec!["VerifiableCredential".to_string(), vc_type.to_string()];
        let valid_until = Some(now + Duration::days(365));

        match self.config.get_data_model_version() {
            W3cDataModelVersion::V1 => parse_to_value(&VCClaimsV1 {
                exp: None,
                iat: None,
                jti: Some(id.to_string()),
                iss: Some(did.to_string()),
                sub: Some(did.to_string()),
                vc: VCFromClaimsV1 {
                    context: context_v1,
                    r#type: types,
                    id: id.to_string(),
                    credential_subject: subject,
                    issuer,
                    valid_from: Some(now),
                    valid_until,
                },
            }),
            W3cDataModelVersion::V2 => parse_to_value(&VCClaimsV2 {
                exp: None,
                iat: None,
                jti: Some(id.to_string()),
                iss: Some(did.to_string()),
                sub: Some(did.to_string()),
                context: context_v2,
                r#type: types,
                id: id.to_string(),
                credential_subject: subject,
                issuer,
                valid_from: Some(now),
                valid_until,
            }),
        }
    }

    async fn build_vp(&self, vcs: &[MatchingVCs], did: Option<&str>) -> Outcome<String> {
        info!("Building VP 4 GAIA");

        let did = did.unwrap_or_else(|| self.config.get_did());

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(did.to_string());
        let mut iss = HashMap::new();
        iss.insert("iss".to_string(), did.to_string());
        header.extras = iss;
        let priv_key = expect_from_env("VAULT_APP_PRIV_KEY");
        let priv_key: StringHelper = self.vault.read(None, &priv_key).await?;

        let key = get_rsa_key(priv_key.data())?;

        let now = Utc::now();

        let mut claims = GaiaVP {
            context: vec![],
            r#type: "VerifiablePresentation".to_string(),
            verifiable_credential: vec![],
            issuer: did.to_string(),
            valid_from: Some(now),
            valid_until: Some(now + Duration::days(1)),
        };

        let context;
        match self.config.get_data_model_version() {
            W3cDataModelVersion::V1 => {
                context = &["https://www.w3.org/ns/credentials/v1"];
            }
            W3cDataModelVersion::V2 => {
                context = &["https://www.w3.org/ns/credentials/v2"];
            }
        }

        let mut jwts = vec![];

        for vc in vcs {
            let jwt = VcInsideGaiaVPBuilder::default()
                .context(context)
                .id(vc.document.clone())
                .build();
            jwts.push(jwt);
        }

        let context: Vec<String> = context.iter().map(ToString::to_string).collect();
        claims.verifiable_credential = jwts;
        claims.context = context;

        let vc_jwt = sign_token(&header, &claims, &key)?;

        info!("{}", vc_jwt);
        Ok(vc_jwt)
    }

    async fn send_req(&self, body: &str) -> Outcome<String> {
        info!("Sending request to retrieve Gaia-x Compliance vc");

        let url = format!(
            "{}?urn:uuid:{}",
            self.config.get_gaia_api_host(),
            Uuid::new_v4().to_string()
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
        headers.insert(ACCEPT, HeaderValue::from_static("text/plain"));

        let res = self
            .client
            .post(&url, Some(headers), Body::Raw(body.to_string()))
            .await?;

        if res.status().is_success() {
            info!("Gaia Compliance Vc retrieved successfully");
            res.parse_text().await
        } else {
            Err(Errors::petition(
                &url,
                "POST",
                Some(res.status()),
                PetitionFailure::HttpStatus(res.status()),
                "Petition to retrieve gaia vc failed",
                None,
            ))
        }
    }

    fn get_vc_types(&self) -> Vec<VcType> {
        vec![VcType::LegalPerson, VcType::TermsAndConditions]
    }

    fn generate_vpds(&self, vc_types: &[VcType]) -> Vec<VPDef> {
        let data_model = self.config.get_data_model_version();
        let mut vpds: Vec<VPDef> = Vec::new();
        for vc_type in vc_types {
            let id = uuid::Uuid::new_v4().to_string();
            vpds.push(VPDef::new(&id, &vc_type.to_string(), data_model))
        }
        vpds
    }
}
