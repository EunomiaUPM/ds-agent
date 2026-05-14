use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260514_000002_oauth_refresh_tokens"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OauthRefreshTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OauthRefreshTokens::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OauthRefreshTokens::TenantId).string().not_null())
                    // JWT ID stored for revocation — unique per issued token.
                    .col(ColumnDef::new(OauthRefreshTokens::Jti).string().not_null().unique_key())
                    .col(
                        ColumnDef::new(OauthRefreshTokens::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OauthRefreshTokens::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OauthRefreshTokens::Revoked)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(OauthRefreshTokens::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum OauthRefreshTokens {
    Table,
    Id,
    TenantId,
    Jti,
    ExpiresAt,
    CreatedAt,
    Revoked,
}
