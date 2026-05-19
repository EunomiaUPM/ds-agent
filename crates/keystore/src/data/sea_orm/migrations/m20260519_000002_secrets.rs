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

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260519_000002_keystore_secrets"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(KeystoreSecrets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(KeystoreSecrets::Key)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    // value is stored as plaintext; callers are responsible for
                    // application-level encryption before passing to the repo.
                    .col(
                        ColumnDef::new(KeystoreSecrets::Value)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(KeystoreSecrets::Version)
                            .big_integer()
                            .not_null()
                            .default(1i64),
                    )
                    .col(ColumnDef::new(KeystoreSecrets::Description).string())
                    .col(
                        ColumnDef::new(KeystoreSecrets::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(KeystoreSecrets::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(KeystoreSecrets::CreatedBy)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(KeystoreSecrets::UpdatedBy)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .col(ColumnDef::new(KeystoreSecrets::DeletedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(KeystoreSecrets::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum KeystoreSecrets {
    Table,
    Key,
    Value,
    Version,
    Description,
    CreatedAt,
    UpdatedAt,
    CreatedBy,
    UpdatedBy,
    DeletedAt,
}
