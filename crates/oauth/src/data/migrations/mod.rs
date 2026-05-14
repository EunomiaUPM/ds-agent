use sea_orm_migration::prelude::MigrationTrait;

mod m20260514_000001_users;
mod m20260514_000002_refresh_tokens;

pub fn get_oauth_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20260514_000001_users::Migration),
        Box::new(m20260514_000002_refresh_tokens::Migration),
    ]
}
