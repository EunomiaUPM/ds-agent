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

use crate::services::gaia_self_attester::GaiaSelfAttesterTrait;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use common::config::types::GaiaConfig;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use ymir::errors::Outcome;
use ymir::types::jwt::{VCJwtClaims, VcJwtClaimsBuilder};
use ymir::types::vcs::doc::VcDocumentBuilder;
use ymir::types::vcs::vc_specs::legal_person::{
    Address, LegalPersonCredentialSubject, TypedRegistrationNumber,
};
use ymir::types::vcs::vc_specs::terms_and_conds::TermsAndConditionsCredSub;
use ymir::types::vcs::VcType;
use ymir::types::vcs::{VcIssuer, W3cDataModelVersion};
use ymir::types::wallet::Identity;

pub struct GaiaSelfAttester {
    config: GaiaConfig,
    identity: Arc<RwLock<Identity>>,
}

impl GaiaSelfAttester {
    pub fn new(config: GaiaConfig, identity: Arc<RwLock<Identity>>) -> GaiaSelfAttester {
        GaiaSelfAttester { config, identity }
    }
}

#[async_trait]
impl GaiaSelfAttesterTrait for GaiaSelfAttester {
    async fn generate_terms_cons_vc(&self) -> Outcome<VCJwtClaims> {
        let identity = self.identity.read().await;
        let holder_did = identity.did().id().to_string();
        let cred_subj = TermsAndConditionsCredSub {
            id: holder_did,
            uri: "https://gaia-x.eu/.well-known/terms-and-conditions.json#cs".to_string(),
            hash: "4bd7554097444c960292b4726c2efa1373485e8a5565d94d41195214c5e0ceb3".to_string(),
        };
        let value = serde_json::to_value(cred_subj)?;
        Ok(self.build(value, VcType::TermsAndConditions).await)
    }

    async fn generate_legal_person(&self) -> Outcome<VCJwtClaims> {
        let identity = self.identity.read().await;
        let holder_did = identity.did().id().to_string();

        let lp = &self.config.legal_person;
        let cred_subj = LegalPersonCredentialSubject {
            id: holder_did,
            gx_registration_number: TypedRegistrationNumber {
                id: None,
                gx_registration_number_type: lp.registration_number.kind.to_string(),
                gx_registration_number_value: lp.registration_number.value.clone(),
            },
            gx_legal_address: Address {
                id: None,
                r#type: "gx:Address".to_string(),
                country_code: lp.legal_address.country_code.clone(),
                country_name: lp.legal_address.country_name.clone(),
                locality: lp.legal_address.locality.clone(),
                postal_code: lp.legal_address.postal_code.clone(),
                street_address: lp.legal_address.street_address.clone(),
            },
            gx_headquarters_address: Address {
                id: None,
                r#type: "gx:Address".to_string(),
                country_code: lp.headquarters_address.country_code.clone(),
                country_name: lp.headquarters_address.country_name.clone(),
                locality: lp.headquarters_address.locality.clone(),
                postal_code: lp.headquarters_address.postal_code.clone(),
                street_address: lp.headquarters_address.street_address.clone(),
            },
            schema_name: lp.name.clone(),
            schema_description: lp.description.clone(),
        };

        let value = serde_json::to_value(cred_subj)?;
        Ok(self.build(value, VcType::LegalPerson).await)
    }
}

impl GaiaSelfAttester {
    async fn build(&self, cred_subj: Value, vc_type: VcType) -> VCJwtClaims {
        let identity = self.identity.read().await;
        let holder_did = identity.did().id();

        let now = Utc::now();
        let credential_id = format!("urn:uuid:{}", Uuid::new_v4().to_string());
        let doc = VcDocumentBuilder::new(&vc_type, W3cDataModelVersion::default())
            .id(&credential_id)
            .issuer(VcIssuer::Did(holder_did.to_string()))
            .credential_subject(cred_subj)
            .valid_from(now)
            .valid_until(now + Duration::days(365))
            .build();

        VcJwtClaimsBuilder::new(W3cDataModelVersion::default())
            .iss(holder_did)
            .sub(holder_did)
            .jti(&credential_id)
            .iat(now)
            .exp(now + Duration::days(365))
            .vc(doc)
            .build()
    }
}
