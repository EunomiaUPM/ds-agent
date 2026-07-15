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

use crate::setup::boot::TransferBoot;
use crate::setup::db_migrations::TransferAgentMigration;
use clap::{Parser, Subcommand};
use common::boot::BootstrapInit;
use common::config::services::TransferConfig;
use common::config::types::traits::{CommonConfigTrait, ConfigLoader};
use std::sync::Arc;
use tracing::{debug, info};
use ymir::config::traits::ConnectionConfigTrait;
use ymir::errors::Outcome;
use ymir::services::vault::fake_vault::FakeVaultService;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::vault_rs::RealVaultService;

#[derive(Parser, Debug)]
#[command(name = "Eunomia DS-Agent Dataspace Connector Transfer Agent")]
#[command(version = "0.2")]
struct TransferCli {
    #[clap(subcommand)]
    command: TransferCliCommands,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum TransferCliCommands {
    Start(TransferCliArgs),
    Setup(TransferCliArgs),
}

#[derive(Parser, Debug, PartialEq)]
pub struct TransferCliArgs {
    #[arg(short, long)]
    env_file: String,
}

pub struct TransferCommands {}

impl TransferCommands {
    pub async fn init_command_line() -> Outcome<()> {
        debug!("init_command_line - Initialize transfer commands");
        let cli = TransferCli::parse();
        match cli.command {
            TransferCliCommands::Start(args) => {
                BootstrapInit::<TransferBoot>::new(args.env_file)
                    .run()
                    .await?;
            }
            TransferCliCommands::Setup(args) => {
                let config = TransferConfig::load(&*args.env_file)?;
                let vault = if config.common().is_vault_real() {
                    VaultService::Real(RealVaultService::new()?)
                } else {
                    VaultService::Fake(FakeVaultService::new()?)
                };
                let table = json_to_table::json_to_table(&serde_json::to_value(&config)?)
                    .collapse()
                    .to_string();
                info!("Current Transfer Agent Config:\n{}", table);
                TransferAgentMigration::run(&config, Arc::new(vault)).await?;
            }
        }
        Ok(())
    }
}
