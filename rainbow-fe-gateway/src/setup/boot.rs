use crate::setup::http_worker::GatewayHttpWorker;
use rainbow_common::boot::BootstrapServiceTrait;
use rainbow_common::config::services::GatewayConfig;
use rainbow_common::config::types::traits::ConfigLoader;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tokio_util::sync::CancellationToken;
use ymir::services::vault::vault_rs::VaultService;

pub struct GatewayBoot;

#[async_trait::async_trait]
impl BootstrapServiceTrait for GatewayBoot {
    type Config = GatewayConfig;

    async fn load_config(env_file: String) -> anyhow::Result<Self::Config> {
        let config = Self::Config::load(env_file);
        let table =
            json_to_table::json_to_table(&serde_json::to_value(&config)?).collapse().to_string();
        tracing::info!("Current Catalog Agent Config:\n{}", table);
        Ok(config)
    }

    async fn start_services_background(
        config: &Self::Config,
        _vault_service: Arc<VaultService>,
    ) -> anyhow::Result<Sender<()>> {
        // thread control
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        let cancel_token = CancellationToken::new();

        // workers
        tracing::info!("Spawning HTTP subsystem...");
        let http_handle = GatewayHttpWorker::spawn(config, &cancel_token).await?;

        // non-blocking thread
        let token_clone = cancel_token.clone();
        tokio::spawn(async move {
            tokio::select! {
                // ctrl+c
                _ = shutdown_rx.recv() => {
                    tracing::info!("Shutdown command received from Main Pipeline.");
                }
                _ = async { http_handle.await } => {
                    tracing::error!("HTTP subsystem failed or stopped unexpectedly!");
                }
            }

            tracing::info!("Initiating internal graceful shutdown sequence...");
            token_clone.cancel();
            tracing::info!("Background services stopped.");
        });

        Ok(shutdown_tx)
    }
}
