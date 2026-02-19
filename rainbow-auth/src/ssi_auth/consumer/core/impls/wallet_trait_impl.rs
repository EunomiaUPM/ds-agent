/*
 *
 *  * Copyright (C) 2024 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */
use crate::ssi_auth::common::errors::AuthErrors;
use crate::ssi_auth::common::traits::RainbowSSIAuthWalletTrait;
use crate::ssi_auth::common::types::jwt::AuthJwtClaims;
use crate::ssi_auth::common::types::oidc::{CredentialOfferResponse, MatchVCsRequest, Vpd};
use crate::ssi_auth::common::types::ssi::dids::DidsInfo;
use crate::ssi_auth::common::types::ssi::keys::KeyDefinition;
use crate::ssi_auth::common::types::ssi::other::{MatchingVCs, RedirectResponse};
use crate::ssi_auth::common::types::ssi::wallet::{WalletInfo, WalletInfoResponse, WalletLoginResponse};
use crate::ssi_auth::consumer::core::traits::consumer_trait::RainbowSSIAuthConsumerManagerTrait;
use crate::ssi_auth::consumer::core::Manager;
use anyhow::bail;
use async_trait::async_trait;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::HeaderMap;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rainbow_common::config::consumer_config::ApplicationConsumerConfigTrait;
use rainbow_common::errors::helpers::{BadFormat, MissingAction};
use rainbow_common::errors::{CommonErrors, ErrorLog};
use rainbow_db::auth_consumer::entities::mates;
use rainbow_db::auth_consumer::repo_factory::factory_trait::AuthRepoFactoryTrait;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};
use url::Url;
use urlencoding::decode;

#[mockall::automock]
#[async_trait]
impl<T> RainbowSSIAuthWalletTrait for Manager<T>
where
    T: AuthRepoFactoryTrait + Send + Sync + Clone + 'static,
{
    // BASIC ----------------------------------------------------------------------------------------------->
    async fn register_wallet(&self) -> anyhow::Result<()> {
        info!("Registering in web wallet");
        let url = format!(
            "{}/wallet-api/auth/register",
            self.config.get_wallet_portal_url()
        );
        let wallet_data = self.config.get_wallet_data();

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);

        let res = match self.client.post(&url).headers(headers).json(&wallet_data).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            201 => {
                info!("Wallet account registration successful");
            }
            409 => {
                warn!("Wallet account has already registered");
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "POST".to_string(),
                    res.status().as_u16(),
                    Some("Petition to register Wallet failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }
        Ok(())
    }

    async fn login_wallet(&self) -> anyhow::Result<()> {
        info!("Login into web wallet");
        let url = format!(
            "{}/wallet-api/auth/login",
            self.config.get_wallet_portal_url()
        );

        let mut wallet_data = self.config.get_wallet_data();
        wallet_data.as_object_mut().map(|obj| obj.remove("name"));

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);

        let res = match self.client.post(&url).headers(headers).json(&wallet_data).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            200 => {
                info!("Wallet login successful");

                let json_res: WalletLoginResponse = res.json().await?;

                let mut wallet_session = self.wallet_session.lock().await;
                wallet_session.account_id = Some(json_res.id);
                wallet_session.token = Some(json_res.token.clone());

                let jwt_parts: Vec<&str> = json_res.token.split('.').collect();

                if jwt_parts.len() != 3 {
                    let error = CommonErrors::format_new(
                        BadFormat::Sent,
                        Some("The jwt does not have the correct format".to_string()),
                    );
                    error!("{}", error.log());
                    bail!(error);
                }

                let decoded = URL_SAFE_NO_PAD.decode(jwt_parts[1])?;

                let claims: AuthJwtClaims = serde_json::from_slice(&decoded)?;

                wallet_session.token_exp = Some(claims.exp);

                Ok(())
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "POST".to_string(),
                    res.status().as_u16(),
                    Some("Petition to login into Wallet failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    async fn logout_wallet(&self) -> anyhow::Result<()> {
        info!("Login out of web wallet");
        let url = format!(
            "{}/wallet-api/auth/logout",
            self.config.get_wallet_portal_url()
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);

        let res = match self.client.post(&url).headers(headers).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            200 => {
                info!("Wallet logout successful");
                let mut wallet_session = self.wallet_session.lock().await;
                wallet_session.token = None;
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "POST".to_string(),
                    res.status().as_u16(),
                    Some("Petition to logout from Wallet failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }

        Ok(())
    }

    async fn onboard_wallet(&self) -> anyhow::Result<()> {
        info!("Onboarding into web wallet");
        if !self.wallet_onboard {
            self.register_wallet().await?
        }
        self.login_wallet().await?;
        self.retrieve_wallet_info().await?;
        self.retrieve_keys().await?;
        self.retrieve_wallet_dids().await?;

        let wallet = self.get_wallet().await?;
        let key_data = self.get_key().await?;
        let did_info = match wallet.dids.first() {
            Some(data) => data.clone(),
            None => {
                bail!("Something unexpected happened");
            }
        };

        self.delete_did(did_info).await?;
        self.delete_key(key_data).await?;

        self.register_key().await?;
        self.retrieve_keys().await?;

        self.register_did().await?;

        self.retrieve_wallet_info().await?;
        self.retrieve_wallet_dids().await?;
        self.set_default_did().await?;

        let wallet = self.get_wallet().await?;
        let did_info = match wallet.dids.first() {
            Some(data) => data.clone(),
            None => {
                bail!("Something unexpected happened");
            }
        };

        let mate = mates::NewModel {
            participant_id: did_info.did,
            participant_slug: "Myself".to_string(),
            participant_type: "Agent".to_string(),
            base_url: self.config.get_ssi_auth_host_url().unwrap(),
            token: None,
            is_me: true,
        };

        self.save_mate(mate).await?;

        info!("Onboarding completed successfully");
        Ok(())
    }

    async fn partial_onboard(&self) -> anyhow::Result<()> {
        info!("Initializing partial onboarding");

        self.login_wallet().await?;
        self.retrieve_wallet_info().await?;
        self.retrieve_keys().await?;
        self.retrieve_wallet_dids().await?;

        info!("Initialization successful");
        Ok(())
    }

    // GET FROM MANAGER ------------------------------------------------------------------------------------>
    // It gives a cloned Value, not a reference
    async fn get_wallet(&self) -> anyhow::Result<WalletInfo> {
        info!("Getting wallet data");
        let wallet_session = self.wallet_session.lock().await;

        match wallet_session.wallets.first() {
            Some(data) => Ok(data.clone()),
            None => {
                let error = CommonErrors::missing_action_new(
                    "There is no wallet associated to this session".to_string(),
                    MissingAction::Wallet,
                    Some("There is no wallet to retrieve dids from".to_string()),
                );
                error!("{}", error.log());
                bail!(error)
            }
        }
    }

    async fn get_did(&self) -> anyhow::Result<String> {
        info!("Getting Did");
        let wallet = self.get_wallet().await?;

        match wallet.dids.first() {
            Some(did_entry) => Ok(did_entry.did.clone()),
            None => {
                let error = CommonErrors::missing_action_new(
                    "A DID is needed".to_string(),
                    MissingAction::Did,
                    Some("No DIDs found in wallet".to_string()),
                );
                error!("{}", error.log());
                bail!(error)
            }
        }
    }

    async fn get_token(&self) -> anyhow::Result<String> {
        info!("Getting token");
        let wallet_session = self.wallet_session.lock().await;

        match &wallet_session.token {
            Some(token) => Ok(token.clone()),
            None => {
                let error = CommonErrors::missing_action_new(
                    "There is no token associated to this session".to_string(),
                    MissingAction::Token,
                    Some("There is no token available for use".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    async fn get_did_doc(&self) -> anyhow::Result<Value> {
        info!("Getting Did Document");

        let wallet = self.get_wallet().await?;

        let did = match wallet.dids.first() {
            Some(did_entry) => did_entry.document.clone(),
            None => {
                let error = CommonErrors::missing_action_new(
                    "A DID is needed".to_string(),
                    MissingAction::Did,
                    Some("No DIDs found in wallet".to_string()),
                );
                error!("{}", error.log());
                bail!(error)
            }
        };

        let json: Value = serde_json::from_str(did.as_str())?;
        Ok(json)
    }

    async fn get_key(&self) -> anyhow::Result<KeyDefinition> {
        info!("Getting key data");

        let key_data = self.key_data.lock().await;
        match key_data.first() {
            Some(data) => Ok(data.clone()),
            None => {
                let error =
                    CommonErrors::missing_action_new("Retrieve keys first".to_string(), MissingAction::Key, None);
                error!("{}", error.log());
                bail!(error)
            }
        }
    }

    // RETRIEVE FROM WALLET ------------------------------------------------------------------------------->
    async fn retrieve_wallet_info(&self) -> anyhow::Result<()> {
        info!("Retrieving wallet info from web wallet");
        let url = format!(
            "{}/wallet-api/wallet/accounts/wallets",
            self.config.get_wallet_portal_url()
        );

        let token = self.get_token().await?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let res = match self.client.get(&url).headers(headers).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "GET".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            200 => {
                let weird_wallets = res.json::<WalletInfoResponse>().await?.wallets;
                let mut wallets = Vec::<WalletInfo>::new();
                for wallet in weird_wallets {
                    let wallet = wallet.to_normal();
                    if !wallets.contains(&wallet) {
                        wallets.push(wallet);
                    }
                }
                let mut wallet_session = self.wallet_session.lock().await;
                for wallet in wallets {
                    if !wallet_session.wallets.contains(&wallet) {
                        wallet_session.wallets.push(wallet);
                    }
                }
                info!("Wallet data loaded successfully");
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "GET".to_string(),
                    res.status().as_u16(),
                    Some("Petition to retrieve Wallet information failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }

        Ok(())
    }

    async fn retrieve_keys(&self) -> anyhow::Result<()> {
        info!("Retrieving keys from web wallet");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/keys",
            self.config.get_wallet_portal_url(),
            &wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/plain".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let res = match self.client.get(&url).headers(headers).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "GET".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            200 => {
                info!("Keys retrieved successfully");
                let res = res.text().await?;
                let keys: Vec<KeyDefinition> = serde_json::from_str(&res)?;
                let mut key_data = self.key_data.lock().await;
                for key in keys {
                    if !key_data.contains(&key) {
                        key_data.push(key);
                    }
                }
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "GET".to_string(),
                    res.status().as_u16(),
                    Some("Petition to retrieve keys failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }

        Ok(())
    }

    async fn retrieve_wallet_dids(&self) -> anyhow::Result<()> {
        info!("Retrieving dids from web wallet");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/dids",
            self.config.get_wallet_portal_url(),
            &wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let res = match self.client.get(&url).headers(headers).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "GET".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            200 => {
                let dids: Vec<DidsInfo> = res.json().await?;
                let mut wallet_session = self.wallet_session.lock().await;

                let wallet = match wallet_session.wallets.first_mut() {
                    Some(data) => data,
                    None => {
                        let error = CommonErrors::missing_action_new(
                            "No wallet available".to_string(),
                            MissingAction::Wallet,
                            None,
                        );
                        error!("{}", error.log());
                        bail!(error)
                    }
                };

                for did in dids {
                    if !wallet.dids.contains(&did) {
                        wallet.dids.push(did)
                    }
                }

                info!("Wallet Dids data loaded successfully");
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "GET".to_string(),
                    res.status().as_u16(),
                    Some("Petition to retrieve Wallet DIDs failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }

        Ok(())
    }

    // REGISTER STUFF IN WALLET ----------------------------------------------------------------------------->
    async fn register_key(&self) -> anyhow::Result<()> {
        info!("Registering key in web wallet");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;
        let body = self.config.get_priv_key();

        let url = format!(
            "{}/wallet-api/wallet/{}/keys/import",
            self.config.get_wallet_portal_url(),
            &wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/plain".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let res = match self.client.post(&url).headers(headers).body(body).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            201 => {
                info!("Key registered successfully");
                let res = res.text().await?;
                debug!("{}", res);
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "POST".to_string(),
                    res.status().as_u16(),
                    Some("Petition to register key failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }

        Ok(())
    }

    async fn register_did(&self) -> anyhow::Result<()> {
        info!("Registering did in web wallet");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;
        let key_data = self.get_key().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/dids/create/jwk?keyId={}&alias=privatekey",
            self.config.get_wallet_portal_url(),
            &wallet.id,
            key_data.key_id.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let res = match self.client.post(&url).headers(headers).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            200 => {
                info!("Did registered successfully");
                let res = res.text().await?;
                debug!("{:#?}", res);
            }
            409 => {
                warn!("Did already exists");
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "POST".to_string(),
                    res.status().as_u16(),
                    Some("Petition to register key failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }

        Ok(())
    }

    async fn set_default_did(&self) -> anyhow::Result<()> {
        info!("Setting default did in web wallet");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;
        let did = self.get_did().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/dids/default?did={}",
            self.config.get_wallet_portal_url(),
            &wallet.id,
            did
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let res = match self.client.post(&url).headers(headers).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            202 => {
                info!("Did has been set as default");
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "POST".to_string(),
                    res.status().as_u16(),
                    Some("Petition to set did as default failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }

        Ok(())
    }

    // DELETE STUFF FROM WALLET --------------------------------------------------------------------------->
    async fn delete_key(&self, key_id: KeyDefinition) -> anyhow::Result<()> {
        info!("Deleting key in web wallet and from internal data");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/keys/{}",
            self.config.get_wallet_portal_url(),
            &wallet.id,
            key_id.key_id.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/plain".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let res = match self.client.delete(&url).headers(headers).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "DELETE".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            202 => {
                info!("Key deleted successfully from web wallet");
                let mut keys_data = self.key_data.lock().await;
                keys_data.retain(|key| *key != key_id);
                info!("Key deleted successfully from internal data");
                Ok(())
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "DELETE".to_string(),
                    res.status().as_u16(),
                    Some("Petition to delete key failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    async fn delete_did(&self, did_info: DidsInfo) -> anyhow::Result<()> {
        info!("Deleting did from web wallet and from internal data");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/dids/{}",
            self.config.get_wallet_portal_url(),
            &wallet.id,
            did_info.did
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/plain".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let res = match self.client.delete(&url).headers(headers).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "DELETE".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            202 => {
                info!("Did deleted successfully from web wallet");
                let mut wallet_session = self.wallet_session.lock().await;

                let wallet = match wallet_session.wallets.first_mut() {
                    Some(data) => data,
                    None => {
                        let error = CommonErrors::missing_action_new(
                            "No wallet available".to_string(),
                            MissingAction::Wallet,
                            None,
                        );
                        error!("{}", error.log());
                        bail!(error)
                    }
                };

                wallet.dids.retain(|did| *did != did_info);
                info!("Did deleted successfully from internal data");
                Ok(())
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "DELETE".to_string(),
                    res.status().as_u16(),
                    Some("Petition to delete key failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    async fn resolve_credential_offer(&self, uri: String) -> anyhow::Result<CredentialOfferResponse> {
        info!("Resolving credential offer");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/exchange/resolveCredentialOffer",
            self.config.get_wallet_portal_url(),
            &wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/plain;charset=UTF-8".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let res = match self.client.post(&url).headers(headers).body(uri).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            200 => {
                let data: CredentialOfferResponse = res.json().await?;
                info!("Credential offer resolved successfully");
                debug!("{:#?}", data);
                Ok(data)
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "POST".to_string(),
                    res.status().as_u16(),
                    Some("Petition to resolve credential offer failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    async fn resolve_credential_issuer(&self, issuer_uri: String) -> anyhow::Result<()> {
        info!("Resolving credential issuer data");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/exchange/resolveIssuerOpenIDMetadata?issuer={}",
            self.config.get_wallet_portal_url(),
            &wallet.id,
            issuer_uri
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let res = match self.client.get(&url).headers(headers).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "GET".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            200 => {
                // let data: Value = res.json().await?;
                info!("Credential issuer resolved successfully");
                // debug!("{:#?}", data);
                Ok(())
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "GET".to_string(),
                    res.status().as_u16(),
                    Some("Petition resolve credential issuer failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    async fn use_offer_req(&self, uri: String, pin: String) -> anyhow::Result<()> {
        info!("Accepting credential");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;
        let did = self.get_did().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/exchange/useOfferRequest?did={}&requireUserInput=false&pinOrTxCode={}",
            self.config.get_wallet_portal_url(),
            &wallet.id,
            did,
            pin
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let res = match self.client.post(&url).headers(headers).body(uri).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            200 => {
                let data: Value = res.json().await?;
                info!("Credential accepted successfully");
                debug!("{:#?}", data);
                Ok(())
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "POST".to_string(),
                    res.status().as_u16(),
                    Some("Petition accept credential issuer failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    async fn join_exchange(&self, exchange_url: String) -> anyhow::Result<String> {
        info!("Joining exchange");
        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/exchange/resolvePresentationRequest",
            self.config.get_wallet_portal_url(),
            &wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "text/plain".parse()?);
        headers.insert(ACCEPT, "text/plain".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let res = match self.client.post(&url).headers(headers).body(exchange_url).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            200 => {
                info!("Joined the exchange successful");
                let data = res.text().await?;
                Ok(data)
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "POST".to_string(),
                    res.status().as_u16(),
                    Some("Error joining the exchange".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    async fn parse_vpd(&self, vpd_as_string: String) -> anyhow::Result<Vpd> {
        info!("Parsing Verifiable Presentation Definition");
        let url = Url::parse(decode(&vpd_as_string).unwrap().as_ref())?;

        if let Some((_, vpd_json)) = url.query_pairs().find(|(key, _)| key == "presentation_definition") {
            match serde_json::from_str::<Vpd>(&vpd_json) {
                Ok(data) => {
                    debug!("{:#?}", data);
                    Ok(data)
                }
                Err(err) => {
                    let error = CommonErrors::format_new(
                        BadFormat::Received,
                        Some(format!(
                            "Error parsing the credential -> {}",
                            err.to_string()
                        )),
                    );
                    error!("{}", error.log());
                    bail!(error)
                }
            }
        } else {
            let error = CommonErrors::format_new(
                BadFormat::Received,
                Some("Invalid presentation definition".to_string()),
            );
            error!("{}", error.log());
            bail!(error)
        }
    }

    async fn get_matching_vcs(&self, vpd: Vpd) -> anyhow::Result<Vec<String>> {
        info!("Matching Verifiable Credentials for OIDC4VP");
        let mut vcs_id = Vec::new();
        for descriptor in vpd.input_descriptors {
            let n_vpd = Vpd { id: "temporal_id".to_string(), input_descriptors: vec![descriptor] };
            let vcs = self.match_vc4vp(serde_json::to_value(n_vpd)?).await?;
            let vc_id = match vcs.first() {
                Some(vc) => vc.id.clone(),
                None => {
                    let error = CommonErrors::forbidden_new(Some(
                        "There are no VCs that match the specified input descriptor".to_string(),
                    ));
                    error!("{}", error.log());
                    bail!(error)
                }
            };
            vcs_id.push(vc_id);
        }
        Ok(vcs_id)
    }

    async fn match_vc4vp(&self, vp_def: Value) -> anyhow::Result<Vec<MatchingVCs>> {
        info!("Matching for an specific descriptor");
        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/exchange/matchCredentialsForPresentationDefinition",
            self.config.get_wallet_portal_url(),
            wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let res = match self.client.post(&url).headers(headers).json(&vp_def).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            200 => {
                info!("Credentials matched successfully");
                let vc_json: Vec<MatchingVCs> = res.json().await?;
                Ok(vc_json)
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "POST".to_string(),
                    res.status().as_u16(),
                    Some("Petition to match credentials failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    async fn present_vp(&self, preq: String, creds: Vec<String>) -> anyhow::Result<RedirectResponse> {
        info!("Presenting Verifiable Presentation");
        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;
        let did = self.get_did().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/exchange/usePresentationRequest",
            self.config.get_wallet_portal_url(),
            wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

        let body = MatchVCsRequest { did, presentation_request: preq, selected_credentials: creds };

        let res = match self.client.post(&url).headers(headers).json(&body).send().await {
            Ok(data) => data,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        match res.status().as_u16() {
            200 => {
                info!("Credentials presented successfully");
                let data: RedirectResponse = res.json().await?;
                Ok(data)
            }
            _ => {
                let error = AuthErrors::wallet_new(
                    url,
                    "POST".to_string(),
                    res.status().as_u16(),
                    Some("Petition to present credentials failed".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    // OTHER ----------------------------------------------------------------------------------------->
    async fn token_expired(&self) -> anyhow::Result<bool> {
        info!("Checking if token is expired");
        let wallet_session = self.wallet_session.lock().await;

        match wallet_session.token_exp {
            Some(expiration_time) => {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();

                if now >= expiration_time {
                    info!("Token expired");
                    return Ok(true);
                };
                Ok(false)
            }
            None => {
                let error = CommonErrors::unauthorized_new(Some("There is no token".to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    async fn update_token(&self) -> anyhow::Result<()> {
        info!("Updating token");
        match self.login_wallet().await {
            Ok(()) => {
                info!("Token updated successfully");
                Ok(())
            }
            Err(e) => {
                error!("Token update failed: {}", e);
                bail!("Error updating token: {}", e);
            }
        }
    }
    async fn ok(&self) -> anyhow::Result<()> {
        if self.token_expired().await? {
            self.update_token().await?;
        }
        Ok(())
    }
}
