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
use crate::data::migrations::get_negotiation_agent_migrations;
use rainbow_common::config::services::ContractsConfig;
use rainbow_common::config::types::traits::CommonConfigTrait;
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use std::sync::Arc;
use ymir::services::vault::VaultTrait;
use ymir::services::vault::global::VaultService;

pub struct NegotiationAgentMigration;

impl MigratorTrait for NegotiationAgentMigration {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations: Vec<Box<dyn MigrationTrait>> = vec![];
        let mut negotiation_agent_migrations = get_negotiation_agent_migrations();

        migrations.append(&mut negotiation_agent_migrations);
        migrations
    }
}

impl NegotiationAgentMigration {
    pub async fn run(config: &ContractsConfig, vault: Arc<VaultService>) -> anyhow::Result<()> {
        // db_connection
        let db_connection = vault.get_db_connection(config.common()).await;
        // run migration
        Self::refresh(&db_connection).await?;
        Ok(())
    }
}
