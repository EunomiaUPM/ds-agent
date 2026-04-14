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

use crate::data::factory_trait::ConnectorRepoTrait;
use crate::data::repo_sql::connector_distro_relation_repo::ConnectorDistroRelationRepoForSql;
use crate::data::repo_sql::connector_instance_repo::ConnectorInstanceRepoForSql;
use crate::data::repo_sql::connector_template_repo::ConnectorTemplateRepoForSql;
use crate::data::repo_traits::connector_distro_relation_repo::ConnectorDistroRelationRepoTrait;
use crate::data::repo_traits::connector_instance_repo::ConnectorInstanceRepoTrait;
use crate::data::repo_traits::connector_template_repo::ConnectorTemplateRepoTrait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct ConnectorRepoForSql {
    templates_repo: Arc<dyn ConnectorTemplateRepoTrait>,
    instances_repo: Arc<dyn ConnectorInstanceRepoTrait>,
    relation_repo: Arc<dyn ConnectorDistroRelationRepoTrait>,
}

impl ConnectorRepoForSql {
    pub fn create_repo(db_connection: DatabaseConnection) -> Self {
        Self {
            templates_repo: Arc::new(ConnectorTemplateRepoForSql::new(db_connection.clone())),
            instances_repo: Arc::new(ConnectorInstanceRepoForSql::new(db_connection.clone())),
            relation_repo: Arc::new(ConnectorDistroRelationRepoForSql::new(
                db_connection.clone(),
            )),
        }
    }
}

impl ConnectorRepoTrait for ConnectorRepoForSql {
    fn get_templates_repo(&self) -> Arc<dyn ConnectorTemplateRepoTrait> {
        self.templates_repo.clone()
    }

    fn get_instances_repo(&self) -> Arc<dyn ConnectorInstanceRepoTrait> {
        self.instances_repo.clone()
    }

    fn get_distro_relation_repo(&self) -> Arc<dyn ConnectorDistroRelationRepoTrait> {
        self.relation_repo.clone()
    }
}
