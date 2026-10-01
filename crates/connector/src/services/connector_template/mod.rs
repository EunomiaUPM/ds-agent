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

//! Connector template use cases: CRUD over reusable, parameterised blueprints.

pub mod service;

use crate::entities::connector_template::ConnectorTemplateDto;
use crate::entities::filters::ConnectorTemplateFilter;
use common::auth::AccessScope;
use common::paginated_spec::{Page, Paginated, Sort};
use ymir::errors::Outcome;

/// Service interface for connector template CRUD operations.
#[mockall::automock]
#[async_trait::async_trait]
pub trait ConnectorTemplateServiceTrait: Send + Sync {
    /// Page of templates matching the filters.
    async fn get_all_templates(
        &self,
        scope: &AccessScope,
        filters: &ConnectorTemplateFilter,
        page: &Page,
        sort: Sort,
    ) -> Outcome<Paginated<ConnectorTemplateDto>>;
    /// Every version of the template named `template_id`.
    async fn get_templates_by_id(
        &self,
        scope: &AccessScope,
        template_id: &str,
    ) -> Outcome<Vec<ConnectorTemplateDto>>;
    async fn get_template_by_name_and_version(
        &self,
        scope: &AccessScope,
        name: &str,
        version: &str,
    ) -> Outcome<Option<ConnectorTemplateDto>>;
    /// Checks that every placeholder is declared, then stores the template in the caller's tenant.
    async fn create_template(
        &self,
        scope: &AccessScope,
        new_template: &mut ConnectorTemplateDto,
    ) -> Outcome<ConnectorTemplateDto>;
    async fn delete_template_by_name_and_version(
        &self,
        scope: &AccessScope,
        name: &str,
        version: &str,
    ) -> Outcome<()>;
}
