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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

pub mod shutdown;

use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use tokio::sync::broadcast;
use ymir::config::traits::ConnectionConfigTrait;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::fake_vault::FakeVaultService;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::vault_rs::RealVaultService;

#[async_trait::async_trait]
pub trait BootstrapServiceTrait: Send + Sync {
    type Config: ConnectionConfigTrait + Debug + Clone + Send + Sync;

    async fn load_config(env_file: String) -> Outcome<Self::Config>;

    fn enable_participant() -> bool { true }
    async fn create_participant(_config: &Self::Config) -> Outcome<String> {
        Err(Errors::crazy(
            "This service does not support creation of participants.",
            None
        ))
    }

    fn enable_catalog() -> bool { true }
    async fn load_catalog(
        _participant_id: &Option<String>,
        _config: &Self::Config
    ) -> Outcome<String> {
        Err(Errors::crazy(
            "This service does not support creation of catalogs.",
            None
        ))
    }

    fn enable_dataservice() -> bool { true }
    async fn load_dataservice(
        _catalog_id: &Option<String>,
        _config: &Self::Config
    ) -> Outcome<String> {
        Err(Errors::crazy(
            "This service does not support creation of data services.",
            None
        ))
    }

    fn enable_policy_templates() -> bool { true }

    async fn load_policy_templates(_config: &Self::Config) -> Outcome<()> {
        Err(Errors::crazy(
            "This service does not support creation of policy templates.",
            None
        ))
    }

    async fn cleanup_cache(_config: &Self::Config) -> Outcome<()> { Ok(()) }

    async fn start_services_background(
        config: &Self::Config,
        vault_service: Arc<VaultService>
    ) -> Outcome<broadcast::Sender<()>>;
}

#[async_trait::async_trait]
pub trait BootstrapStepTrait: Send + Sync {
    type NextState;
    async fn next_step(self) -> Outcome<Self::NextState>;
}

pub struct BootstrapCurrentState<S: BootstrapStepTrait>(pub S);

pub struct BootstrapInit<S: BootstrapServiceTrait> {
    pub _marker: PhantomData<S>,
    pub env_file: String
}

impl<S: BootstrapServiceTrait> BootstrapInit<S> {
    pub fn new(env_file: String) -> Self { Self { _marker: PhantomData, env_file } }
}

pub struct BootstrapConfigLoaded<S: BootstrapServiceTrait> {
    pub _marker: PhantomData<S>,
    pub env_file: String
}

pub struct BootstrapServicesStarted<S: BootstrapServiceTrait> {
    pub config: S::Config,
    pub shutdown_tx: broadcast::Sender<()>
}

pub struct BootstrapSelfParticipantOnBoarded<S: BootstrapServiceTrait> {
    pub config: S::Config,
    pub shutdown_tx: broadcast::Sender<()>,
    pub participant_id: Option<String>
}

pub struct BootstrapCatalogLoaded<S: BootstrapServiceTrait> {
    pub config: S::Config,
    pub shutdown_tx: broadcast::Sender<()>,
    pub participant_id: Option<String>,
    pub catalog_id: Option<String>
}

pub struct BootstrapDataServiceLoaded<S: BootstrapServiceTrait> {
    pub config: S::Config,
    pub shutdown_tx: broadcast::Sender<()>,
    pub participant_id: Option<String>,
    pub catalog_id: Option<String>,
    pub dataservice_id: Option<String>
}

pub struct BootstrapPolicyTemplateLoaded<S: BootstrapServiceTrait> {
    pub config: S::Config,
    pub shutdown_tx: broadcast::Sender<()>
}

pub struct BootstrapFinalized<S: BootstrapServiceTrait> {
    pub _marker: PhantomData<S>,
    pub shutdown_tx: broadcast::Sender<()>
}

pub struct BootstrapTerminated<S: BootstrapServiceTrait>(PhantomData<S>);

#[async_trait::async_trait]
impl<S: BootstrapServiceTrait> BootstrapStepTrait for BootstrapInit<S> {
    type NextState = BootstrapCurrentState<BootstrapConfigLoaded<S>>;

    async fn next_step(self) -> Outcome<Self::NextState> {
        tracing::info!("Step [1/8]: Init bootstrap configuration");
        Ok(BootstrapCurrentState(BootstrapConfigLoaded {
            _marker: PhantomData,
            env_file: self.env_file
        }))
    }
}

#[async_trait::async_trait]
impl<S: BootstrapServiceTrait> BootstrapStepTrait for BootstrapConfigLoaded<S> {
    type NextState = BootstrapCurrentState<BootstrapServicesStarted<S>>;

    async fn next_step(self) -> Outcome<Self::NextState> {
        tracing::info!("Step [2/8]: Configuration loading");
        let config = S::load_config(self.env_file).await?;

        let vault = match config.is_vault_real() {
            true => VaultService::Real(RealVaultService::new()),
            false => VaultService::Fake(FakeVaultService::new())
        };
        let vault = Arc::new(vault);

        tracing::info!("Step [3/8]: Starting Services in Background");
        S::cleanup_cache(&config).await?;
        let shutdown_tx = S::start_services_background(&config, vault.clone()).await?;

        // waiting for port setup
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        Ok(BootstrapCurrentState(BootstrapServicesStarted {
            config,
            shutdown_tx
        }))
    }
}

#[async_trait::async_trait]
impl<S: BootstrapServiceTrait> BootstrapStepTrait for BootstrapServicesStarted<S> {
    type NextState = BootstrapCurrentState<BootstrapSelfParticipantOnBoarded<S>>;

    async fn next_step(self) -> Outcome<Self::NextState> {
        tracing::info!("Step [4/8]: Creating self participant");

        let participant_id = if S::enable_participant() {
            Some(S::create_participant(&self.config).await?)
        } else {
            None
        };

        Ok(BootstrapCurrentState(BootstrapSelfParticipantOnBoarded {
            config: self.config,
            shutdown_tx: self.shutdown_tx,
            participant_id
        }))
    }
}

#[async_trait::async_trait]
impl<S: BootstrapServiceTrait> BootstrapStepTrait for BootstrapSelfParticipantOnBoarded<S> {
    type NextState = BootstrapCurrentState<BootstrapCatalogLoaded<S>>;

    async fn next_step(self) -> Outcome<Self::NextState> {
        tracing::info!("Step [5/8]: Loading main catalog");

        let catalog_id = if S::enable_catalog() {
            Some(S::load_catalog(&self.participant_id, &self.config).await?)
        } else {
            None
        };

        Ok(BootstrapCurrentState(BootstrapCatalogLoaded {
            config: self.config,
            shutdown_tx: self.shutdown_tx,
            participant_id: self.participant_id,
            catalog_id
        }))
    }
}

#[async_trait::async_trait]
impl<S: BootstrapServiceTrait> BootstrapStepTrait for BootstrapCatalogLoaded<S> {
    type NextState = BootstrapCurrentState<BootstrapDataServiceLoaded<S>>;

    async fn next_step(self) -> Outcome<Self::NextState> {
        tracing::info!("Step [6/8]: Loading main dataservice");

        let dataservice_id = if S::enable_dataservice() {
            Some(S::load_dataservice(&self.catalog_id, &self.config).await?)
        } else {
            None
        };

        Ok(BootstrapCurrentState(BootstrapDataServiceLoaded {
            config: self.config,
            shutdown_tx: self.shutdown_tx,
            participant_id: self.participant_id,
            catalog_id: self.catalog_id,
            dataservice_id
        }))
    }
}

#[async_trait::async_trait]
impl<S: BootstrapServiceTrait> BootstrapStepTrait for BootstrapDataServiceLoaded<S> {
    type NextState = BootstrapCurrentState<BootstrapPolicyTemplateLoaded<S>>;

    async fn next_step(self) -> Outcome<Self::NextState> {
        tracing::info!("Step [7/8]: Loading policy templates.");

        if S::enable_policy_templates() {
            S::load_policy_templates(&self.config).await?
        }

        Ok(BootstrapCurrentState(BootstrapPolicyTemplateLoaded {
            config: self.config,
            shutdown_tx: self.shutdown_tx
        }))
    }
}

#[async_trait::async_trait]
impl<S: BootstrapServiceTrait> BootstrapStepTrait for BootstrapPolicyTemplateLoaded<S> {
    type NextState = BootstrapCurrentState<BootstrapFinalized<S>>;

    async fn next_step(self) -> Outcome<Self::NextState> {
        tracing::info!("Step [8/8]: Bootstrap sequence completed. Services UP.");

        Ok(BootstrapCurrentState(BootstrapFinalized {
            _marker: PhantomData,
            shutdown_tx: self.shutdown_tx
        }))
    }
}

#[async_trait::async_trait]
impl<S: BootstrapServiceTrait> BootstrapStepTrait for BootstrapFinalized<S> {
    type NextState = BootstrapCurrentState<BootstrapTerminated<S>>;

    async fn next_step(self) -> Outcome<Self::NextState> {
        //
        tracing::info!("System is RUNNING. Waiting for termination signal (Ctrl+C)...");
        match tokio::signal::ctrl_c().await {
            Ok(()) => tracing::info!("Shutdown signal received."),
            Err(err) => tracing::error!("Unable to listen for shutdown signal: {}", err)
        }
        tracing::info!("Sending shutdown signal to background services...");
        let _ = self.shutdown_tx.send(());

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        Ok(BootstrapCurrentState(BootstrapTerminated(PhantomData)))
    }
}

#[async_trait::async_trait]
impl<S: BootstrapServiceTrait> BootstrapStepTrait for BootstrapTerminated<S> {
    type NextState = BootstrapCurrentState<BootstrapTerminated<S>>;

    async fn next_step(self) -> Outcome<Self::NextState> {
        tracing::info!("Terminating process.");
        Ok(BootstrapCurrentState(BootstrapTerminated(PhantomData)))
    }
}
