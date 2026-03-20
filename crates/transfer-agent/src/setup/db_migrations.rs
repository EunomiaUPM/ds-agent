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
use crate::data::migrations::get_transfer_agent_migrations;
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use dataplane::get_dataplane_migrations;
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;

pub struct TransferAgentMigration;

impl MigratorTrait for TransferAgentMigration {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations: Vec<Box<dyn MigrationTrait>> = vec![];
        let mut transfer_agent_migrations = get_transfer_agent_migrations();
        let mut data_plane_migrations = get_dataplane_migrations();

        migrations.append(&mut transfer_agent_migrations);
        migrations.append(&mut data_plane_migrations);
        migrations
    }
}

impl TransferAgentMigration {
    pub async fn run(config: &TransferConfig, vault: Arc<VaultService>) -> Outcome<()> {
        // db_connection
        let db_connection = vault.get_db_connection(config.common()).await;
        // run migration
        Self::refresh(&db_connection)
            .await
            .map_err(|e| Errors::crazy("Not able to run migration", Some(Box::new(e))))?;
        Ok(())
    }
}
