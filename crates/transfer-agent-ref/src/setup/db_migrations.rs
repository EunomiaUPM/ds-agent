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
use crate::data::sea_orm::migrations::get_migrations;
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use oauth::get_oauth_migrations;
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::VaultTrait;
use ymir::services::vault::global::VaultService;

pub struct TransferAgentRefMigration;

impl MigratorTrait for TransferAgentRefMigration {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = get_oauth_migrations();
        migrations.append(&mut get_migrations());
        migrations
    }
}

impl TransferAgentRefMigration {
    pub async fn run(config: &TransferConfig, vault: Arc<VaultService>) -> Outcome<()> {
        let db_connection = vault.get_db_connection(config.common()).await;
        Self::refresh(&db_connection)
            .await
            .map_err(|e| Errors::crazy("Not able to run migration", Some(Box::new(e))))?;
        Ok(())
    }
}
