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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! SSI auth agent as a composable module: wallet, GNAP gatekeeper, verifier and issuer.

use std::sync::Arc;

use crate::core::AuthCore;
use crate::data::migrations::get_auth_migrations;
use crate::data::sea_orm::factory::AuthRepoForSql;
use crate::http::AuthRouter;
use crate::services::callback::BasicCallbackService;
use crate::services::gaia_self_attester::{GaiaSelfAttester, GaiaSelfAttesterTrait};
use crate::services::gatekeeper::gnap::GnapGateKeeperConfig;
use crate::services::gatekeeper::gnap::GnapGateKeeperService;
use crate::services::peer_connector::gnap::GnapPeerConnectorConfig;
use crate::services::peer_connector::gnap::GnapPeerConnectorService;
use crate::services::vc_requester::basic::VCReqService;
use crate::services::vc_requester::basic::VCRequesterConfig;
use crate::SERVICE_NAME;
use axum::Router;
use common::config::services::SsiAuthConfig;
use common::config::types::traits::{CommonConfigTrait, GaiaConfigTrait};
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait, WalletConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};
use ymir::services::issuer::{oid4vci_1_0, IssuerTrait};
use ymir::services::vault::global::VaultService;
use ymir::services::verifier::oid4vp_draft20;
use ymir::services::wallet::fafnir::FafnirConfig;
use ymir::services::wallet::fafnir::FafnirService;
use ymir::services::wallet::WalletTrait;
use ymir::types::dids::{DidService, DidServiceType};
use ymir::types::wallet::WalletInstance;

/// The whole auth HTTP plane, built once with its wallet and core.
pub struct AuthModule {
    router: Router,
}

impl AuthModule {
    pub async fn compose(config: &SsiAuthConfig, root: &RootContext) -> Outcome<Self> {
        // ======================================== CONFIGS ========================================
        let vault = root.vault.clone();
        let db_connection = root.db.clone();
        let validator = root.validator.clone();
        let vc_req_config = VCRequesterConfig::from(config);
        let peer_connector_config = GnapPeerConnectorConfig::from(config);
        let gatekeeper_config = GnapGateKeeperConfig::from(config);
        let verifier_config = oid4vp_draft20::VerifierConfig::from(config);
        let core_config = Arc::new(config.clone());

        // ======================================== WALLET =========================================
        let wallet = Self::wallet(config, vault.clone()).await?;
        let arc_identity = wallet.get_identity();

        // ======================================= SERVICES ========================================
        let vc_requester = Arc::new(VCReqService::new(vault.clone(), vc_req_config));
        let peer_connector = Arc::new(GnapPeerConnectorService::new(
            vault.clone(),
            peer_connector_config,
        ));
        let callback = Arc::new(BasicCallbackService::new(vault.clone()));
        let repo = Arc::new(AuthRepoForSql::create_repo(db_connection));
        let gatekeeper = Arc::new(GnapGateKeeperService::new(gatekeeper_config));
        let verifier = Arc::new(oid4vp_draft20::VerifierService::new(verifier_config));

        let (gaia, issuer) = match config.gaia_config() {
            Some(gaia_config) => {
                let issuer_config = oid4vci_1_0::IssuerConfig::from(config);

                let gaia: Arc<dyn GaiaSelfAttesterTrait> = Arc::new(GaiaSelfAttester::new(
                    gaia_config.clone(),
                    arc_identity.clone(),
                ));

                let issuer: Arc<dyn IssuerTrait> = Arc::new(oid4vci_1_0::IssuerService::new(
                    issuer_config,
                    vault.clone(),
                    arc_identity.clone(),
                ));

                (Some(gaia), Some(issuer))
            }
            None => (None, None),
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
            gaia,
            issuer,
            core_config,
        ));

        Ok(Self {
            router: AuthRouter::new(core, validator).router(),
        })
    }

    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        get_auth_migrations()
    }

    async fn wallet(
        config: &SsiAuthConfig,
        vault: Arc<VaultService>,
    ) -> Outcome<Arc<dyn WalletTrait>> {
        // Tenants share this DID, so peers append `/{tenant}/access` to the advertised gate.
        let services = vec![DidService::basic(
            DidServiceType::AuthorizationServer,
            format!(
                "{}{}/gate",
                config.common().get_host(HostType::Http),
                config.common().get_api_version()
            ),
        )];

        match config.get_wallet() {
            WalletInstance::WaltId => {
                Err(Errors::not_impl("Waltid is a legacy option", None))
                // let walt_id_config = WaltIdConfig::from(config);
                // let wallet = WaltIdService::new(
                //     walt_id_config,
                //     vault.clone(),
                //     services,
                //     ParticipantType::Authority,
                // )
                // .await?;
                //
                // Ok(Arc::new(wallet))
            }
            WalletInstance::Fafnir => {
                let fafnir_config = FafnirConfig::from(config);
                let wallet = FafnirService::new(fafnir_config, vault.clone(), services).await?;
                Ok(Arc::new(wallet))
            }
        }
    }
}

impl ServiceModuleTrait for AuthModule {
    fn name(&self) -> &'static str {
        SERVICE_NAME
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        Some((String::new(), self.router.clone()))
    }
}
