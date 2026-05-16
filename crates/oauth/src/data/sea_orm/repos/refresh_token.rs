use std::sync::Arc;

use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repositories::refresh_token::{
    RefreshTokenRepository, RefreshTokenRepositoryError,
};
use crate::data::sea_orm::orm::refresh_token as orm;
use crate::entities::refresh_token::RefreshToken;

pub(crate) struct SeaOrmRefreshTokenRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmRefreshTokenRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl RefreshTokenRepository for SeaOrmRefreshTokenRepository {
    async fn create(&self, token: &RefreshToken) -> Outcome<RefreshToken> {
        orm::ActiveModel::from_domain(token)
            .insert(self.db.as_ref())
            .await
            .map_err(|e| RefreshTokenRepositoryError::Db(Box::new(e)).into_errors())
            .and_then(orm::Model::into_domain)
    }

    async fn get_by_jti(&self, jti: &str) -> Outcome<Option<RefreshToken>> {
        orm::Entity::find()
            .filter(orm::Column::Jti.eq(jti))
            .one(self.db.as_ref())
            .await
            .map_err(|e| RefreshTokenRepositoryError::Db(Box::new(e)).into_errors())?
            .map(orm::Model::into_domain)
            .transpose()
    }

    async fn revoke(&self, id: Uuid) -> Outcome<()> {
        let token = orm::Entity::find_by_id(id.to_string())
            .one(self.db.as_ref())
            .await
            .map_err(|e| RefreshTokenRepositoryError::Db(Box::new(e)).into_errors())?
            .ok_or_else(|| RefreshTokenRepositoryError::NotFound.into_errors())?;

        let mut active: orm::ActiveModel = token.into();
        active.revoked = Set(true);
        active
            .update(self.db.as_ref())
            .await
            .map_err(|e| RefreshTokenRepositoryError::Db(Box::new(e)).into_errors())?;
        Ok(())
    }

    async fn revoke_all_for_tenant(&self, tenant_id: &str) -> Outcome<()> {
        use sea_orm::sea_query::Expr;
        orm::Entity::update_many()
            .col_expr(orm::Column::Revoked, Expr::value(true))
            .filter(orm::Column::TenantId.eq(tenant_id))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| RefreshTokenRepositoryError::Db(Box::new(e)).into_errors())?;
        Ok(())
    }
}
