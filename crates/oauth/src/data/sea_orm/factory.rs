use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::data::factory::OAuthDataFactory;
use crate::data::repositories::refresh_token::RefreshTokenRepository;
use crate::data::repositories::user::UserRepository;
use crate::data::sea_orm::repos::refresh_token::SeaOrmRefreshTokenRepository;
use crate::data::sea_orm::repos::user::SeaOrmUserRepository;

pub(crate) struct SeaOrmDataFactory {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmDataFactory {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }
}

impl OAuthDataFactory for SeaOrmDataFactory {
    fn user_repository(&self) -> Arc<dyn UserRepository> {
        Arc::new(SeaOrmUserRepository::new(self.db.clone()))
    }

    fn refresh_token_repository(&self) -> Arc<dyn RefreshTokenRepository> {
        Arc::new(SeaOrmRefreshTokenRepository::new(self.db.clone()))
    }
}
