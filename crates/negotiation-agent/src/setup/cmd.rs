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

use crate::setup::boot::NegotiationAgentBoot;
use crate::setup::db_migrations::NegotiationAgentMigration;
use clap::{Parser, Subcommand};
use common::boot::BootstrapInit;
use common::config::services::ContractsConfig;
use common::config::types::traits::{CommonConfigTrait, ConfigLoader};
use std::sync::Arc;
use tracing::{debug, info};
use ymir::config::traits::ConnectionConfigTrait;
use ymir::errors::Outcome;
use ymir::services::vault::fake_vault::FakeVaultService;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::vault_rs::RealVaultService;

#[derive(Parser, Debug)]
#[command(name = "Eunomia DS-Agent Negotiation Agent")]
#[command(version = "0.2")]
struct NegotiationCli {
    #[clap(subcommand)]
    command: NegotiationCliCommands,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum NegotiationCliCommands {
    Start(NegotiationCliArgs),
    Setup(NegotiationCliArgs),
}

#[derive(Parser, Debug, PartialEq)]
pub struct NegotiationCliArgs {
    #[arg(short, long)]
    env_file: String,
}

pub struct NegotiationCommands {}

impl NegotiationCommands {
    pub async fn init_command_line() -> Outcome<()> {
        debug!("init_command_line - Initialize negotiation commands");
        let cli = NegotiationCli::parse();
        match cli.command {
            NegotiationCliCommands::Start(args) => {
                BootstrapInit::<NegotiationAgentBoot>::new(args.env_file)
                    .run()
                    .await?;
            }
            NegotiationCliCommands::Setup(args) => {
                let config = ContractsConfig::load(&*args.env_file)?;
                let vault = if config.common().is_vault_real() {
                    VaultService::Real(RealVaultService::new()?)
                } else {
                    VaultService::Fake(FakeVaultService::new()?)
                };
                let table = json_to_table::json_to_table(&serde_json::to_value(&config)?)
                    .collapse()
                    .to_string();
                info!("Current Negotiations Agent Config:\n{}", table);
                NegotiationAgentMigration::run(&config, Arc::new(vault)).await?;
            }
        }
        Ok(())
    }
}
