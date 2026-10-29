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

//! The Postgres data factory.

use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::data::factory::DataFactory;
use crate::data::repo::odrl_offer::OdrlOfferRepoTrait;
use crate::data::repo::policy_template::PolicyTemplateRepoTrait;
use crate::data::repo::resource::ResourceRepoTrait;
use crate::data::repo::schema::SchemaRepoTrait;
use crate::data::sea_orm::repos::odrl_offer::SeaOrmOdrlOfferRepo;
use crate::data::sea_orm::repos::policy_template::SeaOrmPolicyTemplateRepo;
use crate::data::sea_orm::repos::resource::SeaOrmResourceRepo;
use crate::data::sea_orm::repos::schema::SeaOrmSchemaRepo;

/// Every repository over one connection pool.
pub struct SeaOrmDataFactory {
    resources: Arc<dyn ResourceRepoTrait>,
    offers: Arc<dyn OdrlOfferRepoTrait>,
    templates: Arc<dyn PolicyTemplateRepoTrait>,
    schemas: Arc<dyn SchemaRepoTrait>,
}

impl SeaOrmDataFactory {
    pub fn new(db: DatabaseConnection) -> Self {
        let db = Arc::new(db);
        Self {
            resources: Arc::new(SeaOrmResourceRepo::new(db.clone())),
            offers: Arc::new(SeaOrmOdrlOfferRepo::new(db.clone())),
            templates: Arc::new(SeaOrmPolicyTemplateRepo::new(db.clone())),
            schemas: Arc::new(SeaOrmSchemaRepo::new(db)),
        }
    }
}

impl DataFactory for SeaOrmDataFactory {
    fn resource_repo(&self) -> Arc<dyn ResourceRepoTrait> {
        self.resources.clone()
    }

    fn odrl_offer_repo(&self) -> Arc<dyn OdrlOfferRepoTrait> {
        self.offers.clone()
    }

    fn policy_template_repo(&self) -> Arc<dyn PolicyTemplateRepoTrait> {
        self.templates.clone()
    }

    fn schema_repo(&self) -> Arc<dyn SchemaRepoTrait> {
        self.schemas.clone()
    }
}
