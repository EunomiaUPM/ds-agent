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

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{serve, Router};
use axum_server::tls_rustls::RustlsConfig;
use common::config::services::traits::SsiAuthConfigTrait;
use common::config::services::SsiAuthConfig;
use common::config::types::traits::CommonConfigTrait;
use tokio::net::TcpListener;
use tracing::info;
use ymir::config::traits::{
    ApiConfigTrait, ConnectionConfigTrait, DidConfigTrait, HostsConfigTrait, WalletConfigTrait,
};
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};
use ymir::services::client::ClientService;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;
use ymir::services::verifier::basic::BasicVerifierConfig;
use ymir::services::verifier::basic::BasicVerifierService;
use ymir::services::wallet::fafnir::FafnirConfig;
use ymir::services::wallet::fafnir::FafnirService;
use ymir::services::wallet::walt_id::WaltIdConfig;
use ymir::services::wallet::walt_id::WaltIdService;
use ymir::services::wallet::WalletTrait;
use ymir::types::dids::{DidService, DidServiceType};
use ymir::types::participants::ParticipantType;
use ymir::types::secrets::StringHelper;
use ymir::types::wallet::WalletInstance;
use ymir::utils::expect_from_env;

use crate::core::AuthCore;
use crate::http::AuthRouter;
use crate::services::callback::BasicCallbackService;
use crate::services::gaia_self_issuer::basic::BasicGaiaSelfIssuer;
use crate::services::gaia_self_issuer::basic::GaiaSelfIssuerConfig;
use crate::services::gaia_self_issuer::GaiaOwnIssuerTrait;
use crate::services::gatekeeper::gnap::GnapGateKeeperConfig;
use crate::services::gatekeeper::gnap::GnapGateKeeperService;
use crate::services::peer_connector::gnap::GnapPeerConnectorConfig;
use crate::services::peer_connector::gnap::GnapPeerConnectorService;
use crate::services::repo::service::AuthRepoForSql;
use crate::services::vc_requester::basic::VCReqService;
use crate::services::vc_requester::basic::VCRequesterConfig;

pub struct AuthApplication {}

impl AuthApplication {
    pub async fn create_router(
        config: &SsiAuthConfig,
        vault: Arc<VaultService>,
    ) -> Outcome<Router> {
        // =========================================CONFIGS=========================================
        let db_connection = vault.get_db_connection(config.common()).await?;
        let vc_req_config = VCRequesterConfig::from(config);
        let peer_connector_config = GnapPeerConnectorConfig::from(config);
        let gatekeeper_config = GnapGateKeeperConfig::from(config.clone());
        let verifier_config = BasicVerifierConfig::from(config.clone());
        let core_config = Arc::new(config.clone());

        // ========================================SERVICES=========================================
        let vc_requester = Arc::new(VCReqService::new(vault.clone(), vc_req_config));
        let peer_connector = Arc::new(GnapPeerConnectorService::new(
            vault.clone(),
            peer_connector_config,
        ));
        let callback = Arc::new(BasicCallbackService::new(vault.clone()));
        let repo = Arc::new(AuthRepoForSql::create_repo(db_connection));
        let gatekeeper = Arc::new(GnapGateKeeperService::new(gatekeeper_config));
        let verifier = Arc::new(BasicVerifierService::new(verifier_config));

        // let (gaia, issuer) = match config.is_gaia_active() {
        //     true => {
        //         let issuer_config = BasicIssuerConfig::from(config.clone());
        //         let gaia_config = GaiaSelfIssuerConfig::from(config.clone());
        //
        //         let gaia: Option<Arc<dyn GaiaOwnIssuerTrait>> = Some(Arc::new(
        //             BasicGaiaSelfIssuer::new(vault.clone(), client.clone(), gaia_config),
        //         ));
        //
        //         let issuer: Option<Arc<dyn IssuerTrait>> = Some(Arc::new(BasicIssuerService::new(
        //             issuer_config,
        //             vault.clone(),
        //         )));
        //
        //         (gaia, issuer)
        //     }
        //     false => (None, None),
        // };

        let services = vec![DidService::basic(
            DidServiceType::AuthorizationServer,
            format!(
                "{}{}/gate/access",
                config.common().get_host(HostType::Http),
                config.common().get_api_version()
            ),
        )];

        let wallet: Arc<dyn WalletTrait> = match config.get_wallet() {
            WalletInstance::WaltId => {
                let walt_id_config = WaltIdConfig::from(config);
                Arc::new(WaltIdService::new(
                    walt_id_config,
                    vault.clone(),
                    services,
                    ParticipantType::Agent,
                ))
            }
            WalletInstance::Fafnir => {
                let fafnir_config = FafnirConfig::from(config);
                Arc::new(FafnirService::new(
                    fafnir_config,
                    vault.clone(),
                    services,
                    ParticipantType::Agent,
                ))
            }
        };

        // CORE
        let core = Arc::new(AuthCore::new(
            vc_requester,
            peer_connector,
            callback,
            gatekeeper,
            verifier,
            repo,
            wallet,
            core_config,
        ));

        Ok(AuthRouter::new(core).router())
    }

    pub async fn run_basic(config: SsiAuthConfig, vault_service: Arc<VaultService>) -> Outcome<()> {
        let router = Self::create_router(&config, vault_service).await;

        let port = config.common().hosts().get_internal_port(HostType::Http);
        let addr = format!("0.0.0.0:{}", port);
        info!("Starting Eunomia DS-Agent Auth server in {}", addr);

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| Errors::crazy("Error with tcp listener", Some(Box::new(e))))?;

        serve(listener, router)
            .await
            .map_err(|e| Errors::crazy("Error while running basic server", Some(Box::new(e))))
    }

    pub async fn run_tls(config: &SsiAuthConfig, vault: Arc<VaultService>) -> Outcome<()> {
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
        .await
        .map_err(|e| Errors::crazy("Errors parsing certificate stuff", Some(Box::new(e))))?;

        let router = Self::create_router(config, vault).await?;

        let port = config.common().hosts().get_internal_port(HostType::Http);
        let addr_str = format!("0.0.0.0:{}", port);
        let addr: SocketAddr = addr_str
            .parse()
            .map_err(|e| Errors::crazy("Errors with socker address", Some(Box::new(e))))?;
        info!("Starting Eunomia DS-Agent Auth server with TLS in {}", addr);

        axum_server::bind_rustls(addr, tls_config)
            .serve(router.into_make_service())
            .await
            .map_err(|e| Errors::crazy("Error while running basic server", Some(Box::new(e))))?;
        Ok(())
    }
    pub async fn run(config: SsiAuthConfig, vault: Arc<VaultService>) -> Outcome<()> {
        if config.common().is_prod() && !config.common().has_tls_proxy() {
            info!("Running with tls active");
            Self::run_tls(&config, vault).await
        } else {
            info!("Running with basic");
            Self::run_basic(config, vault).await
        }
    }
}
