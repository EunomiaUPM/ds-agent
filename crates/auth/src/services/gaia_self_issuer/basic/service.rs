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
use common::config::types::traits::EntityClientTrait;
use jsonwebtoken::{Algorithm, Header};
use serde_json::{json, Value};
use tracing::info;
use uuid::Uuid;
use ymir::capabilities::{Did, Signer};
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::issuing;
use ymir::errors::{Errors, Outcome, PetitionFailure};
use ymir::services::client::ClientTrait;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;
use ymir::types::issuing::{GiveVC, IssuingToken};
use ymir::types::jwt::VcJwtClaimsBuilder;
use ymir::types::secrets::StringHelper;
use ymir::types::vcs::doc::VcDocumentBuilder;
use ymir::types::vcs::vc_specs::legal_person::LegalPersonCredentialSubject;
use ymir::types::vcs::vc_specs::terms_and_conds::TermsAndConditionsCredSub;
use ymir::types::vcs::VcIssuer;
use ymir::types::vcs::{GaiaVP, VPDef, VcInsideGaiaVPBuilder, VcType};
use ymir::types::wallet::waltid::MatchingVCs;
use ymir::utils::{expect_from_env, ResponseExt};

use super::super::GaiaOwnIssuerTrait;
use super::GaiaSelfIssuerConfig;

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
    fn start_basic_vcs(&self, id: &str, uri: &str) -> issuing::NewModel {
        info!("Starting retrieving basic gaia vcs");
        let host = self.config.get_host(HostType::Http);

        let vc_type = format!("{}&{}", VcType::LegalPerson, VcType::TermsAndConditions);
        issuing::NewModel {
            id: id.to_string(),
            name: self.config.get_clas_id().to_string(),
            vc_type,
            aud: host,
            uri: uri.to_string(),
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

        let legal_id = format!("urn:uuid:{}", Uuid::new_v4().to_string());
        let terms_id = format!("urn:uuid:{}", Uuid::new_v4().to_string());

        let legal_subj =
            serde_json::to_value(&LegalPersonCredentialSubject::new4gaia(did, vc_type, code)?)?;
        let terms_subj = serde_json::to_value(&TermsAndConditionsCredSub::random(did))?;

        let person_vc = self.build_vc(did, &legal_id, &VcType::LegalPerson, legal_subj)?;
        let terms_vc = self.build_vc(did, &terms_id, &VcType::TermsAndConditions, terms_subj)?;

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(did.to_string());

        let key = expect_from_env("VAULT_APP_PRIV_KEY");
        let key: StringHelper = self.vault.read(None, &key).await?;
        let key = Key::try_weird_from("", key.data())?;
        let did = Did::parse_from_kid(did)?;
        let ctx = SigningCtx::new(did, key);

        let person_vc_jwt =
            Signer::sign_enveloped("vc+ld+json+jwt", "vc+ld+json", &person_vc, &ctx)?;
        let terms_vc_jwt = Signer::sign_enveloped("vc+ld+json+jwt", "vc+ld+json", &terms_vc, &ctx)?;

        Ok(json!({
            "credential_responses": vec![
                GiveVC { format: "jwt_vc_json".to_string(), credential: person_vc_jwt.to_string() },
                GiveVC { format: "jwt_vc_json".to_string(), credential: terms_vc_jwt.to_string() }
            ]
        }))
    }

    fn build_vc(&self, did: &str, id: &str, vc_type: &VcType, subject: Value) -> Outcome<Value> {
        let now = Utc::now();

        let w3c_data_model = self.config.get_w3c_data_model();

        let doc = VcDocumentBuilder::new(&vc_type, &w3c_data_model)
            .id(id.to_string())
            .issuer(VcIssuer::Did(did.to_string()))
            .credential_subject(subject)
            .valid_from(now)
            .valid_until(now + Duration::days(365))
            .build();

        let vc = VcJwtClaimsBuilder::new(w3c_data_model)
            .iss(did.to_string())
            .sub(did.to_string())
            .jti(id.to_string())
            .iat(now)
            .exp(now + Duration::days(365))
            .vc(doc)
            .build();

        Ok(serde_json::to_value(&vc)?)
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

        let key = Key::try_weird_from("", priv_key.data())?;

        let now = Utc::now();

        let mut claims = GaiaVP {
            context: vec![self.config.get_w3c_data_model().to_string()],
            r#type: "VerifiablePresentation".to_string(),
            verifiable_credential: vec![],
            issuer: did.to_string(),
            valid_from: Some(now),
            valid_until: Some(now + Duration::days(1)),
        };

        let mut jwts = vec![];

        for vc in vcs {
            let jwt = VcInsideGaiaVPBuilder::default()
                .context(&[self.config.get_w3c_data_model().to_string().as_str()])
                .id(vc.document.clone())
                .build();
            jwts.push(jwt);
        }

        claims.verifiable_credential = jwts;

        let did = Did::parse_from_kid(did)?;
        let ctx = SigningCtx::new(did, key);
        let claims = serde_json::to_value(claims)?;

        let vc_jwt = Signer::sign_enveloped("vp+ld+json+jwt", "vp+ld+json", &claims, &ctx)?;

        info!("{}", vc_jwt);
        Ok(vc_jwt.to_string())
    }

    async fn send_req(&self, body: &str, url: &str) -> Outcome<String> {
        info!("Sending request to retrieve Gaia-x Compliance vc");

        let url = format!("{}?urn:uuid:{}", url, Uuid::new_v4().to_string());

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
        let data_model = self.config.get_w3c_data_model();
        let mut vpds: Vec<VPDef> = Vec::new();
        for vc_type in vc_types {
            let id = Uuid::new_v4().to_string();
            vpds.push(VPDef::new(&id, &[&vc_type.to_string()], data_model))
        }
        vpds
    }
}
