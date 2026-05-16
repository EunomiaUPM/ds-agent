use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use ymir::errors::{Errors, Outcome};

use crate::entities::role::Role;
use crate::entities::user::User;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub tenant_id: String,
    pub email: String,
    pub password_hash: String,
    pub password_salt: String,
    pub role: String,
    pub created_at: DateTimeWithTimeZone,
    pub extra_fields: Json,
}

impl Model {
    pub(crate) fn into_domain(self) -> Outcome<User> {
        let role = self.role.parse::<Role>().map_err(|e| {
            Errors::crazy(
                "invalid role stored in database",
                Some(e.to_string().into()),
            )
        })?;
        Ok(User {
            tenant_id: self.tenant_id,
            email: self.email,
            password_hash: self.password_hash,
            password_salt: self.password_salt,
            role,
            created_at: self.created_at.with_timezone(&Utc),
            extra_fields: self.extra_fields,
        })
    }
}

impl ActiveModel {
    pub(crate) fn from_domain(u: &User) -> Self {
        Self {
            tenant_id: Set(u.tenant_id.clone()),
            email: Set(u.email.clone()),
            password_hash: Set(u.password_hash.clone()),
            password_salt: Set(u.password_salt.clone()),
            role: Set(u.role.to_string()),
            created_at: Set(u.created_at.into()),
            extra_fields: Set(u.extra_fields.clone()),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
