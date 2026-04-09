/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
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

use crate::setup::boot::GatewayBoot;
use crate::subscriptions::subscriptions::GatewaySubscriptions;
use crate::subscriptions::MicroserviceSubscriptionKey;
use clap::{Parser, Subcommand};
use common::boot::BootstrapServiceTrait;
use common::config::services::GatewayConfig;
use common::config::types::traits::{CommonConfigTrait, ConfigLoader};
use fs_extra::dir::{copy, CopyOptions};
use std::cmp::PartialEq;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tracing::debug;
use ymir::config::traits::ConnectionConfigTrait;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::fake_vault::FakeVaultService;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::vault_rs::RealVaultService;

#[derive(Parser, Debug)]
#[command(name = "Eunomia Dataspace Connector Gateway Server")]
#[command(version = "0.2")]
struct GatewayCli {
    #[command(subcommand)]
    command: GatewayCliCommands,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum GatewayCliCommands {
    Start(GatewayCliArgs),
    Subscribe(GatewayCliArgs),
    Build(GatewayCliArgs),
}

#[derive(Parser, Debug, PartialEq)]
pub struct GatewayCliArgs {
    #[arg(short, long)]
    env_file: String,
}

pub struct GatewayCommands;

impl GatewayCommands {
    pub async fn init_command_line() -> Outcome<()> {
        // parse command line
        debug!("Init the command line application");
        let cli = GatewayCli::parse();

        // run scripts
        match cli.command {
            GatewayCliCommands::Start(args) => {
                let config = GatewayBoot::load_config(args.env_file).await?;
                let vault = if config.common().is_vault_real() {
                    VaultService::Real(RealVaultService::new())
                } else {
                    VaultService::Fake(FakeVaultService::new())
                };
                GatewayBoot::start_services_background(&config, Arc::new(vault)).await?;
            }
            GatewayCliCommands::Subscribe(args) => {
                let config = GatewayConfig::load(&*args.env_file)?;
                let microservices_subs = GatewaySubscriptions::new(config.clone());
                microservices_subs
                    .subscribe_to_microservice(MicroserviceSubscriptionKey::Catalog)
                    .await?;
                // TODO when pubsub refactor
                // microservices_subs.subscribe_to_microservice(MicroserviceSubscriptionKey::ContractNegotiation).await?;
                // microservices_subs.subscribe_to_microservice(MicroserviceSubscriptionKey::TransferControlPlane).await?;
            }
            GatewayCliCommands::Build(args) => {
                Self::build_frontend(args.env_file)?;
            }
        };

        Ok(())
    }

    fn build_frontend(_env_file: String) -> Outcome<()> {
        let cwd = "./../gui/admin".to_string();

        // 1. Build react application
        let mut cmd = Command::new("npm")
            .current_dir(&cwd)
            .args(["run", "build", "-w", "admin"])
            .spawn()
            .map_err(|e| {
                Errors::parse(&format!("Failed to spawn npm build process: {}", e), None)
            })?;

        cmd.wait()
            .map_err(|e| Errors::parse(&format!("Failed to wait for npm build: {}", e), None))?;
        debug!("Build command finished successfully");

        // 2. Routes
        let origin = format!("{}/dist", cwd);
        let destination = "./src/static/admin".to_string();
        let dest_path = Path::new(&destination);

        // 3. Clean
        if dest_path.exists() {
            debug!("Cleaning content of: {}", destination);
            for entry in fs::read_dir(dest_path).map_err(|e| Errors::parse(&e.to_string(), None))? {
                let entry = entry.map_err(|e| Errors::parse(&e.to_string(), None))?;
                let path = entry.path();
                if path.is_dir() {
                    fs::remove_dir_all(&path).map_err(|e| Errors::parse(&e.to_string(), None))?;
                } else {
                    fs::remove_file(&path).map_err(|e| Errors::parse(&e.to_string(), None))?;
                }
            }
        } else {
            fs::create_dir_all(dest_path).map_err(|e| Errors::parse(&e.to_string(), None))?;
        }

        // 4. Copy content
        let mut options = CopyOptions::new();
        options.overwrite = true;
        options.copy_inside = true;
        let _ = copy(&origin, &destination, &options)
            .map_err(|e| Errors::parse(&e.to_string(), None))?;

        debug!("Copy command finished successfully");

        Ok(())
    }
}
