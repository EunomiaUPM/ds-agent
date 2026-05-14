use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260514_000001_oauth_users"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OauthUsers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(OauthUsers::TenantId).string().not_null().primary_key())
                    .col(ColumnDef::new(OauthUsers::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(OauthUsers::PasswordHash).string().not_null())
                    .col(ColumnDef::new(OauthUsers::Role).string().not_null())
                    .col(
                        ColumnDef::new(OauthUsers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OauthUsers::ExtraFields)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(OauthUsers::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum OauthUsers {
    Table,
    TenantId,
    Email,
    PasswordHash,
    Role,
    CreatedAt,
    ExtraFields,
}
