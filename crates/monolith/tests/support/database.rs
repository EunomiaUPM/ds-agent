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

//! A throwaway database per participant on the server `DATABASE_URL` points at.

use common::boot::migrations::SetupMigrator;
use monolith::setup::CoreBoot;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use uuid::Uuid;

/// Database `test_<uuid>` with every monolith migration applied; dropped with the value.
pub struct TestDatabase {
    pub connection: DatabaseConnection,
    server_url: String,
    name: String,
}

impl TestDatabase {
    /// Server URL from `DATABASE_URL`, such as `postgres://user:pass@localhost:1400/postgres`.
    pub fn server_url() -> String {
        std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must point at a Postgres server for integration tests")
    }

    pub async fn create() -> Self {
        let server_url = Self::server_url();
        let name = format!("test_{}", Uuid::new_v4().simple());
        let admin = Database::connect(&server_url)
            .await
            .expect("connect to DATABASE_URL");
        admin
            .execute_unprepared(&format!("CREATE DATABASE {name}"))
            .await
            .expect("create test database");
        admin.close().await.ok();

        let connection = Database::connect(Self::url_of(&server_url, &name))
            .await
            .expect("connect to test database");
        SetupMigrator::<CoreBoot>::run(&connection, false)
            .await
            .expect("migrate test database");
        Self {
            connection,
            server_url,
            name,
        }
    }

    // Same server and credentials, another database.
    fn url_of(server_url: &str, name: &str) -> String {
        let base = server_url
            .rsplit_once('/')
            .map_or(server_url, |(base, _)| base);
        format!("{base}/{name}")
    }
}

impl Drop for TestDatabase {
    /// Drops the database on a separate runtime, so it also runs when a test panics.
    fn drop(&mut self) {
        let (server_url, name) = (self.server_url.clone(), self.name.clone());
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            // FORCE ends the participant's pooled connections, which are never closed here.
            rt.block_on(async move {
                let dropped = match Database::connect(&server_url).await {
                    Ok(admin) => admin
                        .execute_unprepared(&format!("DROP DATABASE IF EXISTS {name} WITH (FORCE)"))
                        .await
                        .map(|_| ()),
                    Err(e) => Err(e),
                };
                if let Err(e) = dropped {
                    eprintln!("could not drop test database {name}: {e}");
                }
            });
        })
        .join()
        .ok();
    }
}
