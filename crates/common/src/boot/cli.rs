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

//! Command line shared by every agent binary: `start` serves, `setup` provisions the database.

use std::marker::PhantomData;

use clap::{Args, CommandFactory, FromArgMatches, Parser, Subcommand};
use ymir::errors::Outcome;

use crate::boot::bootstrapper::Bootstrapper;
use crate::boot::BootstrapServiceTrait;
use crate::info_banner::banner;
use crate::telemetry;

#[derive(Parser, Debug)]
struct CliArgs {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    /// Boot the agent and serve until SIGINT/SIGTERM.
    Start(EnvArgs),
    /// Write vault secrets and apply database migrations.
    Setup(SetupArgs),
}

#[derive(Args, Debug)]
struct EnvArgs {
    #[arg(short, long)]
    env_file: String,
}

#[derive(Args, Debug)]
struct SetupArgs {
    #[command(flatten)]
    env: EnvArgs,
    /// Roll back every applied migration before migrating (destroys data).
    #[arg(long)]
    reset: bool,
}

pub struct AgentCli<S>(PhantomData<S>);

impl<S: BootstrapServiceTrait> AgentCli<S> {
    /// Whole body of an agent's `main`: telemetry, banner, then the requested subcommand.
    pub async fn run(service_name: &'static str, big_name: &'static str) -> Outcome<()> {
        telemetry::init(service_name);
        tracing::info!("{}", banner(big_name));
        let matches = CliArgs::command()
            .name(big_name)
            .version(env!("CARGO_PKG_VERSION"))
            .get_matches();
        let args = CliArgs::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
        match args.command {
            CliCommand::Start(env) => Bootstrapper::<S>::start(&env.env_file).await,
            CliCommand::Setup(setup) => {
                Bootstrapper::<S>::setup(&setup.env.env_file, setup.reset).await
            }
        }
    }
}
