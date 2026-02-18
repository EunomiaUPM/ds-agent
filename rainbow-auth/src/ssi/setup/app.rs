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

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{serve, Router};
use axum_server::tls_rustls::RustlsConfig;
use rainbow_common::config::services::SsiAuthConfig;
use rainbow_common::config::traits::CommonConfigTrait;
use tokio::net::TcpListener;
use tracing::info;
use ymir::config::traits::{ConnectionConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::services::client::basic::BasicClientService;
use ymir::services::issuer::basic::config::BasicIssuerConfig;
use ymir::services::issuer::basic::BasicIssuerService;
use ymir::services::issuer::IssuerTrait;
use ymir::services::vault::vault_rs::VaultService;
use ymir::services::vault::VaultTrait;
use ymir::services::verifier::basic::config::BasicVerifierConfig;
use ymir::services::verifier::basic::BasicVerifierService;
use ymir::services::wallet::walt_id::config::WaltIdConfig;
use ymir::services::wallet::walt_id::WaltIdService;
use ymir::services::wallet::WalletTrait;
use ymir::types::secrets::StringHelper;
use ymir::utils::expect_from_env;

use crate::ssi::core::AuthCore;
use crate::ssi::http::AuthRouter;
use crate::ssi::services::business::basic::config::BusinessConfig;
use crate::ssi::services::business::basic::BasicBusinessService;
use crate::ssi::services::callback::basic::BasicCallbackService;
use crate::ssi::services::gaia_self_issuer::basic::config::GaiaSelfIssuerConfig;
use crate::ssi::services::gaia_self_issuer::basic::BasicGaiaSelfIssuer;
use crate::ssi::services::gaia_self_issuer::GaiaOwnIssuerTrait;
use crate::ssi::services::gatekeeper::gnap::config::GnapGateKeeperConfig;
use crate::ssi::services::gatekeeper::gnap::GnapGateKeeperService;
use crate::ssi::services::onboarder::gnap::config::GnapOnboarderConfig;
use crate::ssi::services::onboarder::gnap::GnapOnboarderService;
use crate::ssi::services::repo::service::AuthRepoForSql;
use crate::ssi::services::vc_requester::basic::config::VCRequesterConfig;
use crate::ssi::services::vc_requester::basic::VCReqService;

pub struct AuthApplication {}

impl AuthApplication {
    pub async fn create_router(config: &SsiAuthConfig, vault: Arc<VaultService>) -> Router {
        // CONFIGS
        let db_connection = vault.get_db_connection(config.common()).await;
        let vc_req_config = VCRequesterConfig::from(config.clone());
        let onboarder_config = GnapOnboarderConfig::from(config.clone());
        let gatekeeper_config = GnapGateKeeperConfig::from(config.clone());
        let verifier_config = BasicVerifierConfig::from(config.clone());
        let business_config = BusinessConfig::from(config.clone());
        let core_config = Arc::new(config.clone());

        // SERVICES
        let client = Arc::new(BasicClientService::new());
        let vc_req = Arc::new(VCReqService::new(client.clone(), vault.clone(), vc_req_config));
        let onboarder = Arc::new(GnapOnboarderService::new(
            client.clone(),
            vault.clone(),
            onboarder_config,
        ));
        let callback = Arc::new(BasicCallbackService::new(client.clone()));
        let repo = Arc::new(AuthRepoForSql::create_repo(db_connection));
        let gatekeeper = Arc::new(GnapGateKeeperService::new(gatekeeper_config));
        let business = Arc::new(BasicBusinessService::new(business_config));
        let verifier = Arc::new(BasicVerifierService::new(client.clone(), verifier_config));

        let (gaia, issuer) =
            match config.is_gaia_active() {
                true => {
                    let issuer_config = BasicIssuerConfig::from(config.clone());
                    let gaia_config = GaiaSelfIssuerConfig::from(config.clone());

                    let gaia: Option<Arc<dyn GaiaOwnIssuerTrait>> = Some(Arc::new(
                        BasicGaiaSelfIssuer::new(vault.clone(), client.clone(), gaia_config),
                    ));

                    let issuer: Option<Arc<dyn IssuerTrait>> = Some(Arc::new(
                        BasicIssuerService::new(issuer_config, client.clone(), vault.clone()),
                    ));

                    (gaia, issuer)
                }
                false => (None, None),
            };

        let wallet: Option<Arc<dyn WalletTrait>> = match config.is_wallet_active() {
            true => {
                let walt_id_config = WaltIdConfig::from(config.clone());
                Some(Arc::new(WaltIdService::new(
                    walt_id_config,
                    client.clone(),
                    vault.clone(),
                )))
            }
            false => None,
        };

        // CORE
        let core = Arc::new(AuthCore::new(
            vc_req,
            onboarder,
            callback,
            business,
            gatekeeper,
            verifier,
            repo,
            core_config,
            wallet,
            issuer,
            gaia,
        ));

        AuthRouter::new(core).router()
    }

    pub async fn run_basic(
        config: SsiAuthConfig,
        vault_service: Arc<VaultService>,
    ) -> anyhow::Result<()> {
        let server_message = format!(
            "Starting Auth Consumer server in {}",
            config.common().get_host(HostType::Http)
        );
        info!("{}", server_message);

        let router = Self::create_router(&config, vault_service).await;

        let listener = match config.common().is_local() {
            true => {
                TcpListener::bind(format!(
                    "127.0.0.1{}",
                    config.common().get_weird_port(HostType::Http)
                ))
                .await?
            }
            false => {
                TcpListener::bind(format!(
                    "0.0.0.0{}",
                    config.common().get_weird_port(HostType::Http)
                ))
                .await?
            }
        };

        serve(listener, router).await?;

        Ok(())
    }

    pub async fn run_tls(config: &SsiAuthConfig, vault: Arc<VaultService>) -> anyhow::Result<()> {
        let cert = expect_from_env("VAULT_APP_ROOT_CLIENT_KEY");
        let pkey = expect_from_env("VAULT_APP_CLIENT_KEY");
        let cert: StringHelper = vault.read(None, &cert).await?;
        let pkey: StringHelper = vault.read(None, &pkey).await?;

        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Unable to install crypto utils");

        let tls_config = RustlsConfig::from_pem(
            cert.data().as_bytes().to_vec(),
            pkey.data().as_bytes().to_vec(),
        )
        .await?;

        let router = Self::create_router(config, vault).await;

        let port = config.common().hosts().get_tls_port(HostType::Http);
        let addr_str = if config.common().is_local() {
            format!("127.0.0.1:{}", port)
        } else {
            format!("0.0.0.0:{}", port)
        };
        let addr: SocketAddr = addr_str.parse()?;
        info!("Starting Authority server with TLS in {}", addr);

        axum_server::bind_rustls(addr, tls_config).serve(router.into_make_service()).await?;
        Ok(())
    }
    pub async fn run(config: SsiAuthConfig, vault: Arc<VaultService>) -> anyhow::Result<()> {
        if config.common().is_tls_enabled() {
            info!("Running with tls active");
            Self::run_tls(&config, vault).await
        } else {
            info!("Running with basic");
            Self::run_basic(config, vault).await
        }
    }
}
