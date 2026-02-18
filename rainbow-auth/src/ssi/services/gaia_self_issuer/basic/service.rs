/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use std::sync::Arc;

use anyhow::bail;
use async_trait::async_trait;
use axum::http::header::ACCEPT;
use axum::http::HeaderMap;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, Header};
use serde_json::{json, Value};
use tracing::{error, info};
use ymir::config::traits::{HostsConfigTrait, SingleHostTrait};
use ymir::config::types::HostType;
use ymir::data::entities::issuing;
use ymir::errors::{ErrorLogTrait, Errors};
use ymir::services::client::ClientTrait;
use ymir::services::vault::vault_rs::VaultService;
use ymir::services::vault::VaultTrait;
use ymir::types::http::Body;
use ymir::types::issuing::{GiveVC, IssuingToken};
use ymir::types::secrets::StringHelper;
use ymir::types::vcs::claims_v1::{VCClaimsV1, VCFromClaimsV1};
use ymir::types::vcs::claims_v2::VCClaimsV2;
use ymir::types::vcs::vc_issuer::VCIssuer;
use ymir::types::vcs::vc_specs::legal_person::LegalPersonCredentialSubject;
use ymir::types::vcs::vc_specs::terms_and_conds::TermsAndConditionsCredSub;
use ymir::types::vcs::{GaiaVP, VcInsideGaiaVPBuilder, VcType, W3cDataModelVersion};
use ymir::types::wallet::WalletCredentials;
use ymir::utils::{expect_from_env, get_rsa_key, sign_token};

use super::super::GaiaOwnIssuerTrait;
use super::config::{GaiaGaiaSelfIssuerConfigTrait, GaiaSelfIssuerConfig};

pub struct BasicGaiaSelfIssuer {
    vault: Arc<VaultService>,
    client: Arc<dyn ClientTrait>,
    config: GaiaSelfIssuerConfig
}

impl BasicGaiaSelfIssuer {
    pub fn new(
        vault: Arc<VaultService>,
        client: Arc<dyn ClientTrait>,
        config: GaiaSelfIssuerConfig
    ) -> BasicGaiaSelfIssuer {
        BasicGaiaSelfIssuer { vault, client, config }
    }
}

#[async_trait]
impl GaiaOwnIssuerTrait for BasicGaiaSelfIssuer {
    fn start_basic_vcs(&self) -> issuing::NewModel {
        info!("Starting retrieving basic gaia vcs");
        let id = uuid::Uuid::new_v4().to_string();
        let host = format!(
            "{}{}/gaia",
            self.config.hosts().get_host(HostType::Http),
            self.config.get_api_path()
        );
        let aud = match self.config.is_local() {
            true => host.replace("127.0.0.1", "host.docker.internal"),
            false => host
        };

        let vc_type = format!("{}&{}", VcType::LegalPerson, VcType::TermsAndConditions);
        issuing::NewModel {
            id,
            name: self.config.get_client_config().class_id.clone(),
            vc_type,
            aud
        }
    }

    fn get_token(&self) -> IssuingToken {
        info!("Giving token");
        IssuingToken::default()
    }

    fn get_did(&self) -> String { self.config.get_did() }

    async fn issue_cred(&self, did: &str) -> anyhow::Result<Value> {
        info!("Issuing cred");

        let legal_id = uuid::Uuid::new_v4().to_string();
        let terms_id = uuid::Uuid::new_v4().to_string();

        let legal_person_subj =
            serde_json::to_value(LegalPersonCredentialSubject::default4gaia(did))?;
        let terms_subj = serde_json::to_value(TermsAndConditionsCredSub::new_gaia(did))?;
        let now = Utc::now();
        let person_vc = match self.config.get_data_model_version() {
            W3cDataModelVersion::V1 => serde_json::to_value(VCClaimsV1 {
                exp: None,
                iat: None,
                jti: Some(legal_id.clone()),
                iss: Some(did.to_string()),
                sub: Some(did.to_string()),
                vc: VCFromClaimsV1 {
                    context: vec!["https://www.w3.org/ns/credentials/v1".to_string()],
                    r#type: vec![
                        "VerifiableCredential".to_string(),
                        VcType::LegalPerson.to_string(),
                    ],
                    id: legal_id,
                    credential_subject: legal_person_subj,
                    issuer: VCIssuer { id: did.to_string(), name: None },
                    valid_from: Some(now),
                    valid_until: Some(now + Duration::days(365))
                }
            })?,
            W3cDataModelVersion::V2 => serde_json::to_value(VCClaimsV2 {
                exp: None,
                jti: Some(legal_id.clone()),
                iat: None,
                iss: Some(did.to_string()),
                sub: Some(did.to_string()),
                context: vec!["https://www.w3.org/ns/credentials/v2".to_string()],
                r#type: vec!["VerifiableCredential".to_string(), VcType::LegalPerson.to_string()],
                id: legal_id,
                credential_subject: legal_person_subj,
                issuer: VCIssuer { id: did.to_string(), name: None },
                valid_from: Some(now),
                valid_until: Some(now + Duration::days(365))
            })?
        };

        let terms_vc = match self.config.get_data_model_version() {
            W3cDataModelVersion::V1 => serde_json::to_value(VCClaimsV1 {
                exp: None,
                iat: None,
                jti: Some(terms_id.clone()),
                iss: Some(did.to_string()),
                sub: Some(did.to_string()),
                vc: VCFromClaimsV1 {
                    context: vec!["https://www.w3.org/ns/credentials/v1".to_string()],
                    r#type: vec![
                        "VerifiableCredential".to_string(),
                        VcType::TermsAndConditions.to_string(),
                    ],
                    id: terms_id,
                    credential_subject: terms_subj,
                    issuer: VCIssuer { id: did.to_string(), name: None },
                    valid_from: Some(now),
                    valid_until: Some(now + Duration::days(365))
                }
            })?,
            W3cDataModelVersion::V2 => serde_json::to_value(VCClaimsV2 {
                exp: None,
                iat: None,
                jti: Some(terms_id.clone()),
                iss: Some(did.to_string()),
                sub: Some(did.to_string()),
                context: vec!["https://www.w3.org/ns/credentials/v2".to_string()],
                r#type: vec![
                    "VerifiableCredential".to_string(),
                    VcType::TermsAndConditions.to_string(),
                ],
                id: terms_id,
                credential_subject: terms_subj,
                issuer: VCIssuer { id: did.to_string(), name: None },
                valid_from: Some(now),
                valid_until: Some(now + Duration::days(365))
            })?
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(did.to_string());

        let key = expect_from_env("VAULT_APP_PRIV_KEY");
        let key: StringHelper = self.vault.read(None, &key).await?;

        let key = get_rsa_key(key.data())?;

        let person_vc_jwt = sign_token(&header, &person_vc, &key)?;

        let terms_vc_jwt = sign_token(&header, &terms_vc, &key)?;

        Ok(json!({
            "credential_responses": vec![
                GiveVC {
                    format: "jwt_vc_json".to_string(),
                    credential: person_vc_jwt,
                },
                GiveVC {
                    format: "jwt_vc_json".to_string(),
                    credential: terms_vc_jwt,
                }
            ]
        }))
    }

    async fn build_vp(
        &self,
        vcs: Vec<WalletCredentials>,
        did: Option<String>
    ) -> anyhow::Result<String> {
        info!("Building VP 4 GAIA");

        let did = did.unwrap_or_else(|| self.config.get_did());

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(did.clone());
        let priv_key = expect_from_env("VAULT_APP_PRIV_KEY");
        let priv_key: StringHelper = self.vault.read(None, &priv_key).await?;

        let key = get_rsa_key(priv_key.data())?;

        let now = Utc::now();

        let mut claims = GaiaVP {
            context: vec![],
            r#type: "VerifiablePresentation".to_string(),
            verifiable_credential: vec![],
            issuer: did,
            valid_from: Some(now),
            valid_until: Some(now + Duration::days(1))
        };

        let context;
        match self.config.get_data_model_version() {
            W3cDataModelVersion::V1 => {
                context = vec!["https://www.w3.org/ns/credentials/v1".to_string()];
            }
            W3cDataModelVersion::V2 => {
                context = vec!["https://www.w3.org/ns/credentials/v2".to_string()];
            }
        }

        let mut jwts = vec![];

        for vc in vcs {
            let jwt =
                VcInsideGaiaVPBuilder::default().context(context.clone()).id(vc.document).build();
            jwts.push(jwt);
        }

        claims.verifiable_credential = jwts;
        claims.context = context;

        let vc_jwt = sign_token(&header, &claims, &key)?;

        info!("{}", vc_jwt);
        Ok(vc_jwt)
    }

    async fn send_req(&self, body: String) -> anyhow::Result<String> {
        info!("Sending request to retrieve Gaia-x Compliance vc");

        let url = format!("{}", self.config.gaia_api().get_host());

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse()?);

        let res = self.client.post(&url, Some(headers), Body::Raw(body)).await?;

        match res.status().as_u16() {
            201 => {
                info!("Gaia Compliance Vc retrieved successfully");
                let res = res.text().await?;
                Ok(res)
            }
            _ => {
                let error = Errors::petition_new(
                    &url,
                    "POST",
                    Some(res.status().as_u16()),
                    "Petition to retrieve gaia vc failed"
                );
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    fn get_vc_types(&self) -> Vec<VcType> {
        vec![VcType::LegalPerson, VcType::TermsAndConditions]
    }
}
