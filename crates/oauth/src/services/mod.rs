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

use crate::data::factory::OAuthDataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::entities::role::RbacRole;
use crate::entities::user::User;
use chrono::Utc;

pub(crate) mod password;
pub mod token_service;
pub mod user_service;

/// Admin user seeder.
/// On boot procedures, an admin taken by config info is seeded into the database.
/// This admin has token against, applications via REST-API can perform actions
pub async fn seed_admin_user(
    db: sea_orm::DatabaseConnection,
    tenant_id: &str,
    email: &str,
    default_password: &str,
) -> ymir::errors::Outcome<()> {
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
        role: RbacRole::Admin,
        created_at: Utc::now(),
        extra_fields: serde_json::Value::Object(Default::default()),
    };
    user_repo.create(&user).await?;

    tracing::info!("Admin user '{}' seeded successfully.", email);
    Ok(())
}
