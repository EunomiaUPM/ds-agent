pub(crate) mod password;
pub mod token_service;
pub mod user_service;

// Admin seeding ─────────────────────────────────────────────────────────────

/// Ensures an admin user with the given credentials exists in the database.
/// Idempotent: does nothing if a user with `email` already exists.
pub async fn seed_admin_user(
    db: sea_orm::DatabaseConnection,
    tenant_id: &str,
    email: &str,
    default_password: &str,
) -> ymir::errors::Outcome<()> {
    use crate::data::factory::OAuthDataFactory;
    use crate::data::sea_orm::factory::SeaOrmDataFactory;
    use crate::entities::role::Role;
    use crate::entities::user::User;
    use chrono::Utc;

    let factory = SeaOrmDataFactory::new(db);
    let user_repo = factory.user_repository();

    if user_repo.get_by_email(email).await?.is_some() {
        tracing::info!("Admin user '{}' already exists — skipping seed.", email);
        return Ok(());
    }

    let (password_hash, password_salt) = password::hash_password(default_password)?;
    let user = User {
        tenant_id: tenant_id.to_string(),
        email: email.to_string(),
        password_hash,
        password_salt,
        role: Role::Admin,
        created_at: Utc::now(),
        extra_fields: serde_json::Value::Object(Default::default()),
    };
    user_repo.create(&user).await?;
    tracing::info!("Admin user '{}' seeded successfully.", email);
    Ok(())
}
