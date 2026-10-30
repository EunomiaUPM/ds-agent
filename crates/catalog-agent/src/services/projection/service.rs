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

//! Projections over the closures and the profile registry, cached per reader and generation.

use std::sync::Arc;

use common::oauth::OwnerScope;
use common::rdf::{RdfGraph, RdfSyntax};
use serde_json::{json, Value};
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::types::oauth::UserInfo;

use crate::cache::ProjectionKey;
use crate::entities::resource_kind::ResourceKind;
use crate::entities::resource_set::Truncation;
use crate::entities::vocab::Vocab;
use crate::profiles::ProfileAdapterTrait;
use crate::services::projection::views::{Projection, ProjectionBody};
use crate::services::projection::{ProjectionRequest, ProjectionServiceTrait};
use crate::setup::context::AppContext;

pub struct ProjectionService {
    ctx: Arc<AppContext>,
}

impl ProjectionService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    fn adapter(&self, key: &str) -> Outcome<Arc<dyn ProfileAdapterTrait>> {
        self.ctx.profiles.get(key).ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                format!(
                    "unknown profile {key}; available: {}",
                    self.ctx.profiles.keys().join(", ")
                ),
                None,
            )
        })
    }

    /// The projection written in the requested syntax; JSON-LD is the profile's own output.
    async fn body(
        adapter: &dyn ProfileAdapterTrait,
        projected: Value,
        syntax: RdfSyntax,
    ) -> Outcome<ProjectionBody> {
        if syntax == RdfSyntax::JsonLd {
            return Ok(ProjectionBody::JsonLd(projected));
        }
        let expansion = adapter.engine().expand(&projected).await?;
        let graph = RdfGraph::from_expanded(&expansion.expanded)?;
        graph
            .write(syntax, &Vocab::PREFIXES)
            .map(ProjectionBody::Text)
    }

    fn cache_key(
        user: &UserInfo,
        root: &str,
        request: &ProjectionRequest,
        generation: u64,
    ) -> String {
        let limit = request
            .limit
            .map(|l| l.to_string())
            .unwrap_or_else(|| "all".into());
        let reader = format!("user:{}:{limit}:{}", user.id(), request.syntax.extension());
        ProjectionKey::of(&request.profile, root, &reader, generation)
    }

    fn to_cache(projection: &Projection) -> Value {
        let body = match &projection.body {
            ProjectionBody::JsonLd(value) => value.clone(),
            ProjectionBody::Text(text) => Value::String(text.clone()),
        };
        json!({ "body": body, "total": projection.truncation.total, "shown": projection.truncation.shown })
    }

    fn from_cache(
        cached: Value,
        root: &str,
        adapter: &dyn ProfileAdapterTrait,
        syntax: RdfSyntax,
    ) -> Option<Projection> {
        let count = |key: &str| cached.get(key).and_then(Value::as_u64).map(|n| n as usize);
        let truncation = Truncation {
            total: count("total")?,
            shown: count("shown")?,
        };
        let body = match (syntax, cached.get("body")?) {
            (RdfSyntax::JsonLd, value) => ProjectionBody::JsonLd(value.clone()),
            (_, Value::String(text)) => ProjectionBody::Text(text.clone()),
            _ => return None,
        };
        Some(Projection {
            root: root.to_string(),
            profile: adapter.key().to_string(),
            profile_iri: adapter.profile_iri().to_string(),
            syntax,
            truncation,
            body,
        })
    }
}

#[async_trait::async_trait]
impl ProjectionServiceTrait for ProjectionService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id(), profile = %request.profile))]
    async fn project(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        id: &Urn,
        request: &ProjectionRequest,
    ) -> Outcome<Projection> {
        let adapter = self.adapter(&request.profile)?;
        let root_id = id.to_string();
        let key = Self::cache_key(user, &root_id, request, self.ctx.cache.generation().await);
        if let Some(cached) = self.ctx.cache.get_projection(&key).await {
            if let Some(projection) =
                Self::from_cache(cached, &root_id, adapter.as_ref(), request.syntax)
            {
                return Ok(projection);
            }
        }
        let (root, set) = self
            .ctx
            .closures()
            .resource(&OwnerScope::seeing(user), id)
            .await?
            .filter(|(root, _)| root.kind() == kind)
            .ok_or_else(|| {
                Errors::missing_resource(
                    root_id.clone(),
                    format!("{} not found", kind.term()),
                    None,
                )
            })?;
        // A profile is what the connector shares, so drafts stay out of it.
        let shared = set.shared_from(&root).ok_or_else(|| {
            Errors::missing_resource(root_id.clone(), format!("{} is a draft", kind.term()), None)
        })?;
        let (set, truncation) = shared.truncated(&root_id, request.limit);
        let projected = adapter.project(&set, &root)?;
        let projection = Projection {
            root: root_id,
            profile: adapter.key().to_string(),
            profile_iri: adapter.profile_iri().to_string(),
            syntax: request.syntax,
            truncation,
            body: Self::body(adapter.as_ref(), projected, request.syntax).await?,
        };
        self.ctx
            .cache
            .put_projection(&key, &Self::to_cache(&projection))
            .await;
        Ok(projection)
    }
}
