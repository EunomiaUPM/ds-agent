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

//! ConnectorTemplateService with mocked repositories, split into template validation on create
//! and tenant isolation.

mod isolation;
mod validation;

use common::paginated_spec::{Page, Sort};
use common::test_utils::scopes::TestScopes;
use connector::data::entities::connector_templates;
use connector::data::factory_trait::MockConnectorRepoTrait;
use connector::data::repo_traits::connector_repo_errors::ConnectorTemplateRepoErrors;
use connector::data::repo_traits::connector_template_repo::{
    ConnectorTemplateRepoTrait, MockConnectorTemplateRepoTrait,
};
use connector::entities::connector_template::ConnectorTemplateDto;
use connector::entities::filters::ConnectorTemplateFilter;
use connector::services::connector_template::service::ConnectorTemplateService;
use connector::services::connector_template::ConnectorTemplateServiceTrait;
use serde_json::json;
use std::sync::Arc;
use ymir::errors::RepoIntoErrors;

/// Service whose repo echoes created templates back; validation failures never reach it.
fn mock_entities() -> ConnectorTemplateService {
    let mut template_repo = MockConnectorTemplateRepoTrait::new();
    template_repo
        .expect_create_template()
        .returning(|m| Ok(echo_model(m)));

    let template_repo: Arc<dyn ConnectorTemplateRepoTrait> = Arc::new(template_repo);
    let template_repo_clone = Arc::clone(&template_repo);

    let mut repo = MockConnectorRepoTrait::new();
    repo.expect_get_templates_repo()
        .returning(move || Arc::clone(&template_repo_clone));

    ConnectorTemplateService::new(Arc::new(repo))
}

/// Echoes the inserted model back, as a successful insert would.
fn echo_model(
    m: &connector::data::entities::connector_templates::NewConnectorTemplateModel,
) -> connector_templates::Model {
    connector_templates::Model {
        name: m.name.clone().unwrap_or_default(),
        version: m.version.clone().unwrap_or_default(),
        tenant_id: m.tenant_id.clone(),
        author: m.author.clone().unwrap_or_default(),
        created_at: chrono::Utc::now().into(),
        spec: m.spec.clone(),
    }
}

fn valid_template_dto() -> ConnectorTemplateDto {
    let json_dto = json!({
        "name": "my-template",
        "version": "1.0.0",
        "author": "UPM",
        "authentication": {
            "type": "BASIC_AUTH",
            "username": "user",
            "password": { "type": "PLAIN", "content": "{{__PASSWORD__}}" }
        },
        "interaction": {
            "mode": "PULL",
            "dataAccess": {
                "protocol": "HTTP",
                "urlTemplate": "http://example.com/{{__URL__}}",
                "method": ["GET"],
                "headers": { "Content-Type": "application/json" }
            }
        },
        "parameters": [
            { "paramType": "STRING", "name": "PASSWORD", "title": "Password", "required": true },
            { "paramType": "STRING", "name": "URL", "title": "URL", "required": true }
        ]
    });
    serde_json::from_value(json_dto).unwrap()
}
