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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::cmp::PartialEq;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use rainbow_common::config::services::SsiAuthConfig;
use rainbow_common::config::traits::{CommonConfigTrait, ConfigLoader};
use tracing::{debug, info};
use ymir::config::traits::{ConnectionConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::data::seeders::MateSeeder;
use ymir::services::vault::vault_rs::VaultService;
use ymir::services::vault::VaultTrait;

use super::app::AuthApplication;
use crate::ssi::setup::migrations::AuthMigrator;

#[derive(Parser, Debug)]
#[command(name = "Rainbow Dataspace Aut Server")]
#[command(version = "0.1")]
struct AuthCli {
    #[command(subcommand)]
    command: AuthCliCommands,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum AuthCliCommands {
    Start(AuthCliArgs),
    Setup(AuthCliArgs),
    Vault,
}

#[derive(Parser, Debug, PartialEq)]
pub struct AuthCliArgs {
    #[arg(short, long)]
    env_file: String,
}

pub struct AuthCommands;

impl AuthCommands {
    pub async fn init_command_line() -> anyhow::Result<()> {
        // parse command line
        debug!("Init the command line application");
        let cli = AuthCli::parse();
        let vault = Arc::new(VaultService::new());

        match cli.command {
            AuthCliCommands::Start(args) => {
                let config = SsiAuthConfig::load(args.env_file);
                let table = json_to_table::json_to_table(&serde_json::to_value(&config)?)
                    .collapse()
                    .to_string();
                info!("Current Auth Config:\n{}", table);
                AuthApplication::run(config, vault.clone()).await?;
            }
            AuthCliCommands::Setup(args) => {
                let config = SsiAuthConfig::load(args.env_file);
                let table = json_to_table::json_to_table(&serde_json::to_value(&config)?)
                    .collapse()
                    .to_string();
                info!("Current Auth Config:\n{}", table);
                if config.common().is_tls_enabled() {
                    vault.write_all_secrets(None).await?;
                } else {
                    vault.write_local_secrets(None).await?;
                }

                let connection = vault.get_db_connection(config.common()).await;
                AuthMigrator::run(&connection).await?;

                let did = config.did_config().did;
                let url = config.common().hosts().get_host(HostType::Http);
                MateSeeder::seed(&connection, did, url).await?
            }
            AuthCliCommands::Vault => {}
        }

        Ok(())
    }
}
