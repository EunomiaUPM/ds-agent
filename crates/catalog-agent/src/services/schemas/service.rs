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

//! The schema service over its repository and the resources that point to schemas.

use std::sync::Arc;

use common::config::types::traits::CommonConfigTrait;
use common::errors::NotFoundExt;
use common::json_schema::{JsonSchemaCheck, JsonSchemaInference, SchemaIssue};
use common::oauth::{OwnedTrait, Owner, OwnerScope};
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use common::routes::catalog as routes;
use serde_json::Value;
use sha2::{Digest, Sha256};
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::services::client::ClientExt;
use ymir::types::oauth::UserInfo;
use ymir::utils::http_client;

use crate::entities::commands::NewSchemaVersionCommand;
use crate::entities::errors::CatalogErrors;
use crate::entities::filters::{ResourceFilter, SchemaFilter};
use crate::entities::schemas::{
    CheckSchemaUrlDto, DataSchema, InferSchemaDto, NewSchemaDto, NewSchemaVersionDto,
    ValidateSchemaDto,
};
use crate::profiles::shared::public::PublicReader;
use crate::services::schemas::views::{
    SchemaUrlCheckView, SchemaUsageView, SchemaValidationView, SchemaVersionView, SchemaView,
};
use crate::services::schemas::SchemaServiceTrait;
use crate::setup::context::AppContext;
use crate::SERVICE_NAME;

pub struct SchemaService {
    ctx: Arc<AppContext>,
}

impl SchemaService {
    /// Samples one inference or validation takes.
    const MAX_SAMPLES: usize = 200;
    /// Resources listed as users of a version.
    const MAX_USAGE: u32 = 100;

    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    fn view(&self, schema: DataSchema) -> SchemaView {
        let url = self.url(&schema.id, schema.version);
        SchemaView::assemble(schema, url)
    }

    fn sha256(content: &Value) -> String {
        format!("{:x}", Sha256::digest(content.to_string().as_bytes()))
    }

    /// The schema as stored: checked, and without the `$id` its URL will give it.
    fn checked(schema: &Value) -> Outcome<Value> {
        let issues = JsonSchemaCheck::schema(schema);
        if let Some(issue) = issues.first() {
            return Err(Self::bad(format!(
                "not a valid JSON Schema at {}: {}",
                issue.path, issue.message
            )));
        }
        let mut content = schema.clone();
        if let Value::Object(object) = &mut content {
            object.remove("$id");
        }
        Ok(content)
    }

    fn samples(samples: &[Value]) -> Outcome<&[Value]> {
        if samples.len() > Self::MAX_SAMPLES {
            return Err(Self::bad(format!("at most {} samples", Self::MAX_SAMPLES)));
        }
        Ok(samples)
    }

    fn bad(reason: impl Into<String>) -> Errors {
        Errors::format(BadFormat::Received, reason.into(), None)
    }

    /// Resources the caller sees whose own `schema` is `url`.
    async fn usage(&self, user: &UserInfo, url: &str) -> Outcome<Vec<SchemaUsageView>> {
        let filter = ResourceFilter {
            schema: Some(url.to_string()),
            ..Default::default()
        };
        let page = Page::new(Self::MAX_USAGE, None);
        let resources = self
            .ctx
            .repos
            .resource_repo()
            .get_page(&OwnerScope::seeing(user), &filter, &page, &Sort::default())
            .await?;
        Ok(resources
            .iter()
            .map(|r| SchemaUsageView {
                id: r.id().to_string(),
                kind: r.kind().as_str().to_string(),
                title: r.title(),
            })
            .collect())
    }

    /// Fails with a 409 when any resource, whoever owns it, uses one of `versions`.
    async fn ensure_unused(&self, versions: &[DataSchema]) -> Outcome<()> {
        let resources = self.ctx.repos.resource_repo();
        for schema in versions {
            let url = self.url(&schema.id, schema.version);
            let filter = ResourceFilter {
                schema: Some(url.clone()),
                ..Default::default()
            };
            let users = resources.count(&OwnerScope::All, &filter).await?;
            if users > 0 {
                return Err(CatalogErrors::conflict(format!(
                    "{url} is the schema of {users} resource(s); point them elsewhere first"
                )));
            }
        }
        Ok(())
    }

    async fn store(&self, command: NewSchemaVersionCommand) -> Outcome<SchemaView> {
        let stored = self.ctx.repos.schema_repo().insert(&command).await?;
        events::emit_action!(
            self.ctx.events,
            &stored.owner(),
            crate::EVENT_PREFIX,
            "schema",
            "create",
            &stored
        );
        Ok(self.view(stored))
    }

    /// `{host}{api}/catalog-agent/public`, where hosted versions are served.
    fn public_base(&self) -> String {
        let common = self.ctx.config.common();
        format!(
            "{}{}/{SERVICE_NAME}{}",
            common.get_host(HostType::Http),
            common.get_api_version(),
            routes::public::PREFIX,
        )
    }

    /// The content as served at `url`; a document without title or description takes the
    /// version's, so whoever dereferences it knows what it is.
    fn served(schema: &DataSchema, url: &str) -> Value {
        let mut content = JsonSchemaCheck::published(&schema.content, url);
        if let Value::Object(object) = &mut content {
            object
                .entry("title")
                .or_insert_with(|| Value::String(schema.title.clone()));
            if let Some(description) = &schema.description {
                object
                    .entry("description")
                    .or_insert_with(|| Value::String(description.clone()));
            }
        }
        content
    }

    fn url_issue(message: impl Into<String>) -> SchemaIssue {
        SchemaIssue {
            path: "/".to_string(),
            message: message.into(),
        }
    }
}

#[async_trait::async_trait]
impl SchemaServiceTrait for SchemaService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn list(
        &self,
        user: &UserInfo,
        filter: &SchemaFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<SchemaView>> {
        filter.validate()?;
        let page = page.clamped();
        let scope = OwnerScope::seeing(user);
        let repo = self.ctx.repos.schema_repo();
        let items = repo.get_page(&scope, filter, &page, sort).await?;
        let total = repo.count(&scope, filter).await?;
        let paginated = Paginated::from_page(items, &page, Some(total), |s| {
            Cursor::encode_composite(&s.created_at, &s.id)
        });
        Ok(paginated.map(|s| self.view(s)))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn versions(&self, user: &UserInfo, id: &str) -> Outcome<Vec<SchemaView>> {
        let versions = self
            .ctx
            .repos
            .schema_repo()
            .get_versions(&OwnerScope::seeing(user), id)
            .await?;
        if versions.is_empty() {
            return Err(Errors::missing_resource(id, "schema not found", None));
        }
        Ok(versions.into_iter().map(|s| self.view(s)).collect())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn version(&self, user: &UserInfo, id: &str, version: i32) -> Outcome<SchemaVersionView> {
        let schema = self
            .ctx
            .repos
            .schema_repo()
            .get_version(&OwnerScope::seeing(user), id, version)
            .await?
            .or_not_found(format!("{id}/{version}"), "schema")?;
        let url = self.url(id, version);
        let content = Self::served(&schema, &url);
        Ok(SchemaVersionView {
            used_by: self.usage(user, &url).await?,
            schema: content,
            summary: SchemaView::assemble(schema, url),
        })
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create(&self, user: &UserInfo, input: &NewSchemaDto) -> Outcome<SchemaView> {
        let title = input.title.trim();
        if title.is_empty() {
            return Err(Self::bad("a schema needs a title"));
        }
        let content = Self::checked(&input.schema)?;
        self.store(NewSchemaVersionCommand {
            id: format!("urn:schema:{}", uuid::Uuid::new_v4()),
            version: 1,
            owner: Owner::of(user, input.visibility.clone()),
            title: title.to_string(),
            description: input.description.clone().filter(|d| !d.trim().is_empty()),
            sha256: Self::sha256(&content),
            content,
        })
        .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn add_version(
        &self,
        user: &UserInfo,
        id: &str,
        input: &NewSchemaVersionDto,
    ) -> Outcome<SchemaView> {
        let latest = self
            .ctx
            .repos
            .schema_repo()
            .get_versions(&OwnerScope::acting(user), id)
            .await?
            .into_iter()
            .next()
            .or_not_found(id, "schema")?;
        let content = Self::checked(&input.schema)?;
        self.store(NewSchemaVersionCommand {
            id: latest.id.clone(),
            version: latest.version + 1,
            owner: latest.owner(),
            title: input
                .title
                .clone()
                .filter(|t| !t.trim().is_empty())
                .unwrap_or(latest.title.clone()),
            description: input.description.clone().or(latest.description.clone()),
            sha256: Self::sha256(&content),
            content,
        })
        .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn delete(&self, user: &UserInfo, id: &str, version: Option<i32>) -> Outcome<()> {
        let repo = self.ctx.repos.schema_repo();
        let scope = OwnerScope::acting(user);
        let versions: Vec<DataSchema> = repo
            .get_versions(&scope, id)
            .await?
            .into_iter()
            .filter(|s| version.is_none_or(|v| s.version == v))
            .collect();
        if versions.is_empty() {
            return Err(Errors::missing_resource(id, "schema not found", None));
        }
        self.ensure_unused(&versions).await?;
        repo.delete(&scope, id, version).await?;
        let deleted = match version {
            Some(v) => format!("{id}/{v}"),
            None => id.to_string(),
        };
        events::emit_action!(
            self.ctx.events,
            &versions[0].owner(),
            crate::EVENT_PREFIX,
            "schema",
            "delete",
            &events::EntityDeletedDto::new(deleted)
        );
        Ok(())
    }

    fn infer(&self, input: &InferSchemaDto) -> Outcome<Value> {
        let samples = Self::samples(&input.samples)?;
        if samples.is_empty() {
            return Err(Self::bad("give at least one sample document"));
        }
        Ok(JsonSchemaInference::infer(samples))
    }

    fn validate(&self, input: &ValidateSchemaDto) -> Outcome<SchemaValidationView> {
        let samples = Self::samples(&input.samples)?;
        let issues = JsonSchemaCheck::schema(&input.schema);
        let reports = if issues.is_empty() {
            JsonSchemaCheck::samples(&input.schema, samples)?
        } else {
            vec![]
        };
        Ok(SchemaValidationView {
            valid: issues.is_empty() && reports.iter().all(|r| r.valid),
            issues,
            samples: reports,
        })
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn check_url(&self, input: &CheckSchemaUrlDto) -> Outcome<SchemaUrlCheckView> {
        let url = input.url.trim().to_string();
        let parsed: axum::http::Uri = url
            .parse()
            .map_err(|e| Self::bad(format!("not a URL: {e}")))?;
        if !matches!(parsed.scheme_str(), Some("http" | "https")) || parsed.host().is_none() {
            return Err(Self::bad("only absolute http and https URLs are fetched"));
        }
        let fetched = http_client().get_json::<Value>(&url, None).await;
        let document = match fetched {
            Ok(document) => document,
            Err(e) => {
                return Ok(SchemaUrlCheckView {
                    url,
                    reachable: false,
                    is_json_schema: false,
                    title: None,
                    issues: vec![Self::url_issue(format!("could not fetch it as JSON: {e}"))],
                })
            }
        };
        let mut issues = JsonSchemaCheck::schema(&document);
        let looks_like_schema = document.as_object().is_some_and(|o| {
            [
                "$schema",
                "type",
                "properties",
                "$defs",
                "allOf",
                "oneOf",
                "anyOf",
            ]
            .iter()
            .any(|k| o.contains_key(*k))
        });
        if issues.is_empty() && !looks_like_schema {
            issues.push(Self::url_issue(
                "valid JSON, but it uses no JSON Schema keyword",
            ));
        }
        Ok(SchemaUrlCheckView {
            url,
            reachable: true,
            is_json_schema: issues.is_empty(),
            title: document
                .get("title")
                .and_then(Value::as_str)
                .map(str::to_string),
            issues,
        })
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn published(&self, id: &str, version: i32) -> Outcome<Value> {
        let schema = self
            .ctx
            .repos
            .schema_repo()
            .get_version(&PublicReader::scope(), id, version)
            .await?
            .or_not_found(format!("{id}/{version}"), "schema")?;
        Ok(Self::served(&schema, &self.url(id, version)))
    }

    fn url(&self, id: &str, version: i32) -> String {
        let path = routes::public::SCHEMA
            .replace("{id}", id)
            .replace("{version}", &version.to_string());
        format!("{}{path}", self.public_base())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn check_reference(&self, user: &UserInfo, url: &str) -> Outcome<()> {
        let prefix = format!("{}/schemas/", self.public_base());
        let Some((id, version)) = url
            .strip_prefix(&prefix)
            .and_then(|rest| rest.rsplit_once('/'))
        else {
            return Ok(());
        };
        let found = match version.trim_start_matches('v').parse::<i32>() {
            Ok(version) => self
                .ctx
                .repos
                .schema_repo()
                .get_version(&OwnerScope::seeing(user), id, version)
                .await?
                .is_some(),
            Err(_) => false,
        };
        if found {
            Ok(())
        } else {
            Err(Self::bad(format!(
                "schema: {url} is no hosted schema version you can see"
            )))
        }
    }
}
