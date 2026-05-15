use chrono::Utc;
use uuid::Uuid;
use ymir::errors::{Errors, Outcome};

use crate::data::sea_orm::orm::refresh_token as rt_orm;
use crate::data::sea_orm::orm::user as user_orm;
use crate::entities::refresh_token::RefreshToken;
use crate::entities::role::Role;
use crate::entities::user::User;

pub(crate) fn user_from_orm(m: user_orm::Model) -> Outcome<User> {
    let role = m.role.parse::<Role>().map_err(|e| {
        Errors::crazy("invalid role stored in database", Some(e.to_string().into()))
    })?;
    Ok(User {
        tenant_id: m.tenant_id,
        email: m.email,
        password_hash: m.password_hash,
        password_salt: m.password_salt,
        role,
        created_at: m.created_at.with_timezone(&Utc),
        extra_fields: m.extra_fields,
    })
}

pub(crate) fn user_to_active_model(u: &User) -> user_orm::ActiveModel {
    use sea_orm::ActiveValue::Set;
    user_orm::ActiveModel {
        tenant_id: Set(u.tenant_id.clone()),
        email: Set(u.email.clone()),
        password_hash: Set(u.password_hash.clone()),
        password_salt: Set(u.password_salt.clone()),
        role: Set(u.role.to_string()),
        created_at: Set(u.created_at.into()),
        extra_fields: Set(u.extra_fields.clone()),
    }
}

pub(crate) fn refresh_token_from_orm(m: rt_orm::Model) -> Outcome<RefreshToken> {
    let id = Uuid::parse_str(&m.id)
        .map_err(|e| Errors::crazy("invalid UUID in refresh token", Some(Box::new(e))))?;
    Ok(RefreshToken {
        id,
        tenant_id: m.tenant_id,
        jti: m.jti,
        expires_at: m.expires_at.with_timezone(&Utc),
        created_at: m.created_at.with_timezone(&Utc),
        revoked: m.revoked,
    })
}

pub(crate) fn refresh_token_to_active_model(rt: &RefreshToken) -> rt_orm::ActiveModel {
    use sea_orm::ActiveValue::Set;
    rt_orm::ActiveModel {
        id: Set(rt.id.to_string()),
        tenant_id: Set(rt.tenant_id.clone()),
        jti: Set(rt.jti.clone()),
        expires_at: Set(rt.expires_at.into()),
        created_at: Set(rt.created_at.into()),
        revoked: Set(rt.revoked),
    }
}
