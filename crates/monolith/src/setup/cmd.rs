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

use std::cmp::PartialEq;

use clap::{Parser, Subcommand};
use common::boot::{BootstrapInit, BootstrapStepTrait};
use common::config::types::traits::CommonConfigTrait;
use common::config::ApplicationConfig;
use tracing::{debug, info};
use ymir::config::traits::ConnectionConfigTrait;
use ymir::errors::Outcome;
use ymir::services::vault::fake_vault::FakeVaultService;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::vault_rs::RealVaultService;
use ymir::services::vault::VaultTrait;

use crate::setup::boot::CoreBoot;
use crate::setup::db_migrations::CoreProviderMigration;

#[derive(Parser, Debug)]
#[command(name = "Eunomia DS-Agent Dataspace Connector Core Server")]
#[command(version = "0.2")]
struct CoreCli {
    #[command(subcommand)]
    command: CoreCliCommands,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum CoreCliCommands {
    Start(CoreCliArgs),
    Setup(CoreCliArgs),
}

#[derive(Parser, Debug, PartialEq)]
pub struct CoreCliArgs {
    #[arg(short, long)]
    env_file: String,
}

pub struct CoreCommands;

impl CoreCommands {
    pub async fn init_command_line() -> Outcome<()> {
        // parse command line
        debug!("Init the command line application");
        let cli = CoreCli::parse();

        // run scripts
        match cli.command {
            CoreCliCommands::Start(args) => {
                let init = BootstrapInit::<CoreBoot>::new(args.env_file);
                let step1 = init.next_step().await?; // Init -> Config
                let step2 = step1.0.next_step().await?; // Config -> ServicesStarted (Background)
                let step3 = step2.0.next_step().await?; // Services -> Participant
                let step4 = step3.0.next_step().await?; // Participant -> Catalog
                let step5 = step4.0.next_step().await?; // Catalog -> DataService
                let step6 = step5.0.next_step().await?; // DataService -> PolicyTemplates
                let step_finalized = step6.0.next_step().await?;
                let _ = step_finalized.0.next_step().await?;
            }
            CoreCliCommands::Setup(args) => {
                let config = ApplicationConfig::load(&args.env_file)?;
                let vault = if config.monolith().common().is_vault_real() {
                    VaultService::Real(RealVaultService::new())
                } else {
                    VaultService::Fake(FakeVaultService::new())
                };
                let table =
                    json_to_table::json_to_table(&serde_json::to_value(&config)?)
                        .collapse()
                        .to_string();
                info!("Current Config:\n{}", &table);

                if config.monolith().common().is_prod() {
                    vault.write_all_secrets(None).await?;
                } else {
                    vault.write_local_secrets(None).await?;
                }

                let db_connection = vault.get_db_connection(config.monolith().common()).await;

                CoreProviderMigration::run(&db_connection).await?;
            }
        };

        Ok(())
    }
}
