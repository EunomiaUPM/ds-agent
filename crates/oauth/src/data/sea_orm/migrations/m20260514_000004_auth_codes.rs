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
        "m20260514_000004_oauth_auth_codes"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OauthAuthCodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OauthAuthCodes::Code)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OauthAuthCodes::ClientId).string().not_null())
                    .col(ColumnDef::new(OauthAuthCodes::RedirectUri).string())
                    .col(ColumnDef::new(OauthAuthCodes::TenantId).string().not_null())
                    .col(ColumnDef::new(OauthAuthCodes::Role).string().not_null())
                    .col(
                        ColumnDef::new(OauthAuthCodes::Scopes)
                            .json_binary()
                            .not_null()
                            .default("[]"),
                    )
                    .col(
                        ColumnDef::new(OauthAuthCodes::CodeChallenge)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OauthAuthCodes::CodeChallengeMethod)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OauthAuthCodes::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OauthAuthCodes::Used)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OauthAuthCodes::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum OauthAuthCodes {
    Table,
    Code,
    ClientId,
    RedirectUri,
    TenantId,
    Role,
    Scopes,
    CodeChallenge,
    CodeChallengeMethod,
    ExpiresAt,
    Used,
}
