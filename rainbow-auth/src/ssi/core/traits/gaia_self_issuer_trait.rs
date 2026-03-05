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

use async_trait::async_trait;
use serde_json::Value;
use ymir::errors::{Errors, Outcome};
use ymir::services::issuer::IssuerTrait;
use ymir::services::wallet::WalletTrait;
use ymir::types::issuing::{
    AuthServerMetadata, CredentialRequest, CredentialRequestsss, IssuerMetadata, IssuingToken,
    TokenRequest, VCCredOffer,
};

use crate::ssi::services::gaia_self_issuer::GaiaOwnIssuerTrait;
use crate::ssi::services::repo::repo_trait::AuthRepoTrait;

#[async_trait]
pub trait CoreGaiaSelfIssuerTrait: Send + Sync + 'static {
    fn issuer(&self) -> Arc<dyn IssuerTrait>;
    fn gaia(&self) -> Arc<dyn GaiaOwnIssuerTrait>;
    fn wallet(&self) -> Option<Arc<dyn WalletTrait>>;
    fn repo(&self) -> Arc<dyn AuthRepoTrait>;
    async fn generate_gaia_vcs(&self) -> Outcome<Option<String>> {
        let model = self.gaia().start_basic_vcs();
        let model = self.repo().issuing().create(model).await?;
        let uri = self.issuer().generate_issuing_uri(&model.id, Some("gaia"));

        match self.wallet() {
            Some(wallet) => {
                wallet.process_oidc4vci(&uri).await?;
                Ok(None)
            }
            None => Ok(Some(uri)),
        }
    }
    async fn get_cred_offer_data(&self, id: String) -> Outcome<VCCredOffer> {
        let mut model = self.repo().issuing().get_by_id(&id).await?;

        let data = self.issuer().get_cred_offer_data(&model, Some("gaia"))?;

        if model.step {
            model.step = false;
            self.repo().issuing().update(model).await?;
        };

        Ok(data)
    }
    fn issuer_metadata(&self) -> IssuerMetadata {
        let vcs = self.gaia().get_vc_types();
        self.issuer().get_issuer_data(Some("gaia"), Some(&vcs))
    }
    fn oauth_server_metadata(&self) -> AuthServerMetadata {
        let vcs = self.gaia().get_vc_types();
        self.issuer().get_oauth_server_data(Some("gaia"), Some(&vcs))
    }
    async fn get_token(&self, payload: TokenRequest) -> Outcome<IssuingToken> {
        let model =
            self.repo().issuing().get_by_pre_auth_code(&payload.pre_authorized_code).await?;

        self.issuer().validate_token_req(&model, &payload)?;

        let response = self.issuer().get_token(&model);
        Ok(response)
    }
    async fn issue_cred(&self, payload: CredentialRequest, token: String) -> Outcome<Value> {
        let did = match self.wallet() {
            Some(wallet) => wallet.get_did().await?,
            None => self.gaia().get_did(),
        };

        let mut iss_model = self.repo().issuing().get_by_token(&token).await?;
        self.issuer().validate_cred_req(&mut iss_model, &payload, &token, Some(&did)).await?;
        self.repo().issuing().update(iss_model).await?;

        self.gaia().issue_cred(&did).await
    }

    async fn issue_some_cred(
        &self,
        _payload: CredentialRequestsss,
        _token: String,
    ) -> Outcome<Value> {
        let did = match self.wallet() {
            Some(wallet) => wallet.get_did().await?,
            None => self.gaia().get_did(),
        };

        self.gaia().issue_cred(&did).await
    }

    async fn request_gaia_vc(&self) -> Outcome<()> {
        let wallet = self
            .wallet()
            .ok_or_else(|| Errors::not_impl("Not implemented if wallet is not connected", None))?;

        let vcs = wallet.retrieve_wallet_credentials().await?;

        let did = wallet.get_did().await?;
        let body = self.gaia().build_vp(&vcs, Some(&did)).await?;
        let _vc = self.gaia().send_req(&body).await?;

        Ok(())
    }
}
