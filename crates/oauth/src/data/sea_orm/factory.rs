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

use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::data::factory::OAuthDataFactory;
use crate::data::repositories::token::TokenRepository;
use crate::data::repositories::user::UserRepository;
use crate::data::sea_orm::repos::token::SeaOrmTokenRepository;
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

    fn token_repository(&self) -> Arc<dyn TokenRepository> {
        Arc::new(SeaOrmTokenRepository::new(self.db.clone()))
    }
}
