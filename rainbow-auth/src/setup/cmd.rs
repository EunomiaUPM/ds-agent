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
use rainbow_common::config::types::traits::{CommonConfigTrait, ConfigLoader};
use rainbow_common::utils::show_table;
use tracing::debug;
use ymir::config::traits::ConnectionConfigTrait;
use ymir::errors::Outcome;
use ymir::services::vault::fake_vault::FakeVaultService;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::vault_rs::RealVaultService;
use ymir::services::vault::VaultTrait;

use super::app::AuthApplication;
use crate::setup::migrations::AuthMigrator;

#[derive(Parser, Debug)]
#[command(name = "Rainbow Dataspace Aut Server")]
#[command(version = "0.1")]
struct AuthCli {
    #[command(subcommand)]
    command: AuthCliCommands
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum AuthCliCommands {
    Start(AuthCliArgs),
    Setup(AuthCliArgs)
}

#[derive(Parser, Debug, PartialEq)]
pub struct AuthCliArgs {
    #[arg(short, long)]
    env_file: String
}

pub struct AuthCommands;

impl AuthCommands {
    pub async fn init_command_line() -> Outcome<()> {
        // parse command line
        debug!("Init the command line application");
        let cli = AuthCli::parse();

        match cli.command {
            AuthCliCommands::Start(args) => {
                let (config, vault) = Self::bootstrap(args)?;
                AuthApplication::run(config, Arc::new(vault)).await?;
            }
            AuthCliCommands::Setup(args) => {
                let (config, vault) = Self::bootstrap(args)?;
                match config.common().is_prod() {
                    true => vault.write_all_secrets(None).await?,
                    false => vault.write_local_secrets(None).await?
                }
                let connection = vault.get_db_connection(config.common()).await;
                AuthMigrator::run(&connection).await?;
            }
        }

        Ok(())
    }

    fn bootstrap(args: AuthCliArgs) -> Outcome<(SsiAuthConfig, VaultService)> {
        let config = SsiAuthConfig::load(&args.env_file)?;
        let vault = if config.common().is_vault_real() {
            VaultService::Real(RealVaultService::new())
        } else {
            VaultService::Fake(FakeVaultService::new())
        };
        show_table(&config)?;
        Ok((config, vault))
    }
}
