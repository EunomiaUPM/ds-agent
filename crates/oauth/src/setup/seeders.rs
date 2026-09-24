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

use common::boot::seeders::{BootPhase, BootSeeder};
use common::config::services::CommonConfig;
use common::config::types::{AdminSeedConfig, ServiceClientConfig};
use sea_orm::DatabaseConnection;
use ymir::errors::Outcome;

use crate::services::admin_seeder::{ServiceClientSeeder, seed_admin_user};

/// Seeds the configured admin user and the service client agents authenticate with.
pub struct AdminSeeder {
    db: DatabaseConnection,
    admin: AdminSeedConfig,
    service_client: ServiceClientConfig,
}

impl AdminSeeder {
    pub fn new(db: DatabaseConnection, common: &CommonConfig) -> Self {
        Self {
            db,
            admin: common.admin_seed.clone(),
            service_client: common.service_client.clone(),
        }
    }
}

#[async_trait::async_trait]
impl BootSeeder for AdminSeeder {
    fn name(&self) -> &'static str {
        "oauth-admin"
    }

    /// Straight to the DB, so it precedes every seeder that calls the API as these clients.
    fn phase(&self) -> BootPhase {
        BootPhase::BeforeServe
    }

    async fn seed(&self) -> Outcome<()> {
        let admin = &self.admin;
        seed_admin_user(
            self.db.clone(),
            &admin.tenant_id,
            &admin.email,
            &admin.password,
        )
        .await?;
        ServiceClientSeeder::seed(self.db.clone(), &admin.tenant_id, &self.service_client).await
    }
}
