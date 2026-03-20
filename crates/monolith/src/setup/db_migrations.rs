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

use auth::data::migrator::get_auth_migrations;
use catalog_agent::get_catalog_migrations;
use connector::get_connector_migrations;
use dataplane::get_dataplane_migrations;
use events::data::migrations::get_events_migrations;
use negotiation_agent::get_negotiation_agent_migrations;
use sea_orm::DatabaseConnection;
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use transfer_agent::get_transfer_agent_migrations;
use ymir::errors::{Errors, Outcome};

pub struct CoreProviderMigration;

impl MigratorTrait for CoreProviderMigration {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations: Vec<Box<dyn MigrationTrait>> = vec![];
        let mut catalog_migrations = get_catalog_migrations();
        let mut connector_migrations = get_connector_migrations();
        let mut contract_negotiation_provider_migrations = get_negotiation_agent_migrations();
        let mut pub_sub_migrations = get_events_migrations();
        let mut auth_migrations = get_auth_migrations();
        let mut dataplane_migrations = get_dataplane_migrations();
        let mut transfer_agent_migrations = get_transfer_agent_migrations();

        migrations.append(&mut catalog_migrations);
        migrations.append(&mut connector_migrations);
        migrations.append(&mut contract_negotiation_provider_migrations);
        migrations.append(&mut pub_sub_migrations);
        migrations.append(&mut auth_migrations);
        migrations.append(&mut dataplane_migrations);
        migrations.append(&mut transfer_agent_migrations);
        migrations
    }
}

impl CoreProviderMigration {
    pub async fn run(db_connection: &DatabaseConnection) -> Outcome<()> {
        Self::refresh(db_connection)
            .await
            .map_err(|e| Errors::db("Error migrating data", Some(Box::new(e))))
    }
}
