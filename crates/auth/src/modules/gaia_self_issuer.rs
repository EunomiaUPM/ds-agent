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

use std::str::FromStr;

use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;
use ymir::errors::{BadFormat, Errors, MissingAction, Outcome};
use ymir::services::issuer::IssuerTrait;
use ymir::services::wallet::WalletTrait;
use ymir::types::issuing::{AuthServerMetadata, IssuerMetadata, IssuingToken, TokenRequest};
use ymir::types::vcs::VcType;
use ymir::types::wallet::waltid::MatchingVCs;

use crate::services::gaia_self_issuer::GaiaOwnIssuerTrait;
use crate::services::repo::repo_trait::AuthRepoTrait;

#[async_trait]
pub trait GaiaSelfIssuerModule: Send + Sync + 'static {
    // fn issuer(&self) -> Arc<dyn IssuerTrait>;
    // fn gaia(&self) -> Arc<dyn GaiaOwnIssuerTrait>;
    // fn wallet(&self) -> Arc<dyn WalletTrait>;
    // fn repo(&self) -> Arc<dyn AuthRepoTrait>;
    // async fn generate_gaia_vcs(&self) -> Outcome<Option<String>> {
    //     let id = Uuid::new_v4().to_string();
    //     let uri = self.issuer().generate_issuing_uri(&id, Some("gaia"));
    //     let model = self.gaia().start_basic_vcs(&id, &uri);
    //     let _model = self.repo().issuing().create(model).await?;
    // 
    //     self.wallet().process_oidc4vci(&uri).await?;
    //     Ok(None)
    // }
    // async fn get_cred_offer_data(&self, id: String) -> Outcome<VCCredOffer> {
    //     let mut model = self.repo().issuing().get_by_id(&id).await?;
    // 
    //     let data = self.issuer().get_cred_offer_data(&model)?;
    // 
    //     if model.step {
    //         model.step = false;
    //         self.repo().issuing().update(model).await?;
    //     };
    // 
    //     Ok(data)
    // }
    // fn issuer_metadata(&self) -> IssuerMetadata {
    //     let vcs = self.gaia().get_vc_types();
    //     self.issuer().get_issuer_data(Some("/gaia"), Some(&vcs))
    // }
    // fn oauth_server_metadata(&self) -> AuthServerMetadata {
    //     let vcs = self.gaia().get_vc_types();
    //     self.issuer()
    //         .get_oauth_server_data(Some("/gaia"), Some(&vcs))
    // }
    // async fn get_token(&self, payload: TokenRequest) -> Outcome<IssuingToken> {
    //     let model = self
    //         .repo()
    //         .issuing()
    //         .get_by_pre_auth_code(&payload.pre_authorized_code)
    //         .await?;
    // 
    //     self.issuer().validate_token_req(&model, &payload)?;
    // 
    //     let response = self.issuer().get_token(&model);
    //     Ok(response)
    // }
    // 
    // async fn issue_some_cred(
    //     &self,
    //     _payload: CredentialRequestsss,
    //     _token: String,
    // ) -> Outcome<Value> {
    //     let did = self.wallet().get_did().await?;
    //     let vc_types = vec![
    //         VcType::Euid,
    //         VcType::Eori,
    //         VcType::LeiCode,
    //         VcType::LocalRegistrationNumber,
    //         VcType::TaxId,
    //         VcType::VatId,
    //     ];
    //     let vc_data = self.match_vcs(&vc_types).await?;
    // 
    //     let vc_type = get_real_vc_type(&vc_data.parsed_document)?;
    //     let vc_type = VcType::from_str(&vc_type)?;
    //     let code = get_claim(
    //         &vc_data.parsed_document,
    //         &["CredentialSubject", vc_type.to_gaia_weird()?],
    //     )?;
    // 
    //     self.gaia().issue_cred(&did, &vc_type, &code).await
    // }
    // 
    // async fn request_gaia_vc(&self, url: &str) -> Outcome<()> {
    //     let reg_number = self
    //         .match_vcs(&[
    //             VcType::Euid,
    //             VcType::Eori,
    //             VcType::LeiCode,
    //             VcType::LocalRegistrationNumber,
    //             VcType::TaxId,
    //             VcType::VatId,
    //         ])
    //         .await?;
    //     let legal_person = self.match_vcs(&[VcType::LegalPerson]).await?;
    //     let terms_conds = self.match_vcs(&[VcType::TermsAndConditions]).await?;
    // 
    //     let vcs = vec![reg_number, legal_person, terms_conds];
    //     let did = self.wallet().get_did().await?;
    //     let body = self.gaia().build_vp(&vcs, Some(&did)).await?;
    //     let vc = self.gaia().send_req(&body, url).await?;
    // 
    //     println!("{:#?}", vc);
    //     Ok(())
    // }
    // 
    // async fn match_vcs(&self, vc_types: &[VcType]) -> Outcome<MatchingVCs> {
    //     let vpds = self.gaia().generate_vpds(&vc_types);
    //     let mut vc: Option<MatchingVCs> = None;
    //     for vpd in &vpds {
    //         let vpd = serde_json::to_value(vpd)?;
    //         let vec = self.wallet().match_vc4vp(vpd).await?;
    //         if !vec.is_empty() {
    //             vc = vec.first().cloned();
    //             break;
    //         }
    //     }
    //     vc.ok_or_else(|| {
    //         Errors::missing_action(
    //             MissingAction::Credentials,
    //             "Wallet does not have a registration number vc",
    //             None,
    //         )
    //     })
    // }
}

fn get_real_vc_type(claims: &Value) -> Outcome<String> {
    let types = claims
        .get("type")
        .and_then(|v| v.as_array())
        .ok_or_else(|| Errors::format(BadFormat::Received, "Field 'type' is not an array", None))?;

    let real_type = types
        .iter()
        .filter_map(|v| v.as_str())
        .find(|t| *t != "VerifiableCredential")
        .ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "No VC type found different from 'VerifiableCredential'",
                None,
            )
        })?;

    Ok(real_type.to_string())
}
