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

use crate::SERVICE_BIG_NAME;
use crate::setup::boot::TransferBoot;
use crate::setup::db_migrations::TransferAgentRefMigration;
use clap::{Parser, Subcommand};
use common::boot::BootstrapInit;
use common::config::services::TransferConfig;
use common::config::types::traits::{CommonConfigTrait, ConfigLoader};
use std::sync::Arc;
use tracing::info;
use ymir::errors::Outcome;

#[derive(Parser, Debug)]
#[command(name = SERVICE_BIG_NAME)]
#[command(version)]
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
        let cli = TransferCli::parse();
        match cli.command {
            TransferCliCommands::Start(args) => {
                BootstrapInit::<TransferBoot>::new(args.env_file)
                    .run()
                    .await?;
            }
            TransferCliCommands::Setup(args) => {
                let config = TransferConfig::load(&args.env_file)?;
                let vault = common::vault_utils::vault(config.common())?;
                let table = json_to_table::json_to_table(&serde_json::to_value(&config)?)
                    .collapse()
                    .to_string();
                info!("Current Transfer Agent Ref Config:\n{}", table);
                TransferAgentRefMigration::run(&config, Arc::new(vault)).await?;
            }
        }
        Ok(())
    }
}
