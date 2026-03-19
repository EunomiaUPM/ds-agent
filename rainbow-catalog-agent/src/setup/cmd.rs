/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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
use crate::setup::boot::CatalogAgentBoot;
use crate::setup::db_migrations::CatalogAgentMigration;
use clap::{Parser, Subcommand};
use rainbow_common::boot::{BootstrapInit, BootstrapStepTrait};
use rainbow_common::config::services::CatalogConfig;
use rainbow_common::config::types::traits::ConfigLoader;
use rainbow_common::config::types::roles::RoleConfig;
use std::sync::Arc;
use tracing::{debug, info};
use ymir::services::vault::global::VaultService;

#[derive(Parser, Debug)]
#[command(name = "Rainbow Dataspace Connector Catalog Agent")]
#[command(version = "0.2")]
struct CatalogCli {
    #[clap(subcommand)]
    command: CatalogCliCommands,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum CatalogCliCommands {
    Start(CatalogCliArgs),
    Setup(CatalogCliArgs),
}

#[derive(Parser, Debug, PartialEq)]
pub struct CatalogCliArgs {
    #[arg(short, long)]
    env_file: String,
}

pub struct CatalogCommands {}

impl CatalogCommands {
    pub async fn init_command_line() -> anyhow::Result<()> {
        debug!("init_command_line - Initialize catalog commands");
        let cli = CatalogCli::parse();
        match cli.command {
            CatalogCliCommands::Start(args) => {
                let init = BootstrapInit::<CatalogAgentBoot>::new(args.env_file);
                let step1 = init.next_step().await?; // Init -> Config
                let step2 = step1.0.next_step().await?; // Config -> ServicesStarted (Background)
                let step3 = step2.0.next_step().await?; // Services -> Participant
                let step4 = step3.0.next_step().await?; // Participant -> Catalog
                let step5 = step4.0.next_step().await?; // Catalog -> DataService
                let step6 = step5.0.next_step().await?; // DataService -> PolicyTemplates
                let step_finalized = step6.0.next_step().await?;
                let _ = step_finalized.0.next_step().await?;
            }
            CatalogCliCommands::Setup(args) => {
                let config = CatalogConfig::load(args.env_file);
                let vault = Arc::new(VaultService::new());
                let table = json_to_table::json_to_table(&serde_json::to_value(&config)?)
                    .collapse()
                    .to_string();
                info!("Current Catalog Agent Config:\n{}", table);
                CatalogAgentMigration::run(&config, vault.clone()).await?;
            }
        }
        Ok(())
    }
}
