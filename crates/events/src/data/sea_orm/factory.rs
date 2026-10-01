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

use crate::data::factory::DataFactory;
use crate::data::repo::{
    EventDeadLetterRepo, EventDeliveryRepo, EventStoreRepo, EventSubscriptionRepo,
};
use crate::data::sea_orm::repos::{
    SeaOrmDeadLetterRepo, SeaOrmDeliveryRepo, SeaOrmEventRepo, SeaOrmSubscriptionRepo,
};

// Concrete SeaORM data factory constructing repository trait instances.
#[derive(Clone)]
pub struct SeaOrmDataFactory {
    db: DatabaseConnection,
}

impl SeaOrmDataFactory {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl DataFactory for SeaOrmDataFactory {
    fn event_repository(&self) -> Arc<dyn EventStoreRepo> {
        Arc::new(SeaOrmEventRepo::new(self.db.clone()))
    }

    fn subscription_repository(&self) -> Arc<dyn EventSubscriptionRepo> {
        Arc::new(SeaOrmSubscriptionRepo::new(self.db.clone()))
    }

    fn delivery_repository(&self) -> Arc<dyn EventDeliveryRepo> {
        Arc::new(SeaOrmDeliveryRepo::new(self.db.clone()))
    }

    fn dlq_repository(&self) -> Arc<dyn EventDeadLetterRepo> {
        Arc::new(SeaOrmDeadLetterRepo::new(self.db.clone()))
    }
}
