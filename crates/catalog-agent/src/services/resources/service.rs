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

//! Resource service over the repository. Every write checks where the resource hangs from and
//! what it links to, then invalidates the cached projections and emits a `catalog:` event.

use std::collections::HashSet;
use std::sync::Arc;

use common::errors::NotFoundExt;
use common::oauth::{Owner, OwnerScope};
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use serde_json::{json, Value};
use urn::Urn;
use ymir::errors::{BadFormat, ErrorReason, Errors, Outcome};
use ymir::types::oauth::{RoleTrait, UserInfo};
use ymir::types::participants::Visibility;

use crate::data::repo::resource::ResourceRepoTrait;
use crate::entities::admin_document::AdminDocument;
use crate::entities::commands::{NewResourceCommand, ReplaceResourceCommand};
use crate::entities::errors::CatalogErrors;
use crate::entities::filters::ResourceFilter;
use crate::entities::graph::ResourceGraph;
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::{Parent, ResourceKind};
use crate::services::resources::ResourceServiceTrait;
use crate::setup::context::AppContext;

pub struct ResourceService {
    ctx: Arc<AppContext>,
}

impl ResourceService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    fn repo(&self) -> Arc<dyn ResourceRepoTrait> {
        self.ctx.repos.resource_repo()
    }

    /// The resource as `user` sees it, if it is of `kind`.
    async fn seen(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        id: &Urn,
    ) -> Outcome<CatalogResource> {
        self.repo()
            .get_by_id(&OwnerScope::seeing(user), id)
            .await?
            .filter(|r| r.kind() == kind)
            .or_not_found(id, kind.term())
    }

    /// The resource as `user` may change it, if it is of `kind`.
    async fn acted_on(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        id: &Urn,
    ) -> Outcome<CatalogResource> {
        self.repo()
            .get_by_id(&OwnerScope::acting(user), id)
            .await?
            .filter(|r| r.kind() == kind)
            .or_not_found(id, kind.term())
    }

    /// The catalog or dataset the kind requires, seen by `user`, as `(catalog_id, dataset_id)`.
    async fn placement(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        parent: Option<&Urn>,
    ) -> Outcome<(Option<Urn>, Option<Urn>)> {
        let required = match kind.parent() {
            Parent::None => {
                return match parent {
                    None => Ok((None, None)),
                    Some(_) => Err(Self::invalid(
                        "inCatalog",
                        format!("a {} hangs from no other resource", kind.term()),
                    )),
                };
            }
            Parent::Catalog => ResourceKind::Catalog,
            Parent::Dataset => ResourceKind::Dataset,
        };
        let term = if required == ResourceKind::Dataset {
            "ofDataset"
        } else {
            "inCatalog"
        };
        let parent = parent.ok_or_else(|| {
            Self::invalid(
                term,
                format!("a {} needs its {}", kind.term(), required.term()),
            )
        })?;
        let found = self
            .repo()
            .get_by_id(&OwnerScope::seeing(user), parent)
            .await?
            .filter(|p| p.kind() == required)
            .ok_or_else(|| {
                Self::invalid(
                    term,
                    format!("{parent} is not a {} you can see", required.term()),
                )
            })?;
        let id = found.id().clone();
        Ok(match required {
            ResourceKind::Dataset => (None, Some(id)),
            _ => (Some(id), None),
        })
    }

    /// Every `urn:` link must reach a resource `user` sees, of the kind the link expects.
    async fn check_links(&self, user: &UserInfo, description: &ResourceGraph) -> Outcome<()> {
        let links = description.local_links();
        if links.is_empty() {
            return Ok(());
        }
        let ids: Vec<Urn> = links
            .iter()
            .map(|(_, id)| id.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|id| {
                id.parse::<Urn>()
                    .map_err(|_| Self::invalid("@id", format!("{id} is not a URN")))
            })
            .collect::<Outcome<_>>()?;
        let targets = self
            .repo()
            .get_batch(&OwnerScope::seeing(user), &ids)
            .await?;
        let mut reasons = Vec::new();
        for (predicate, id) in &links {
            let found = targets.iter().find(|t| t.id().to_string() == *id);
            match (found, ResourceKind::linked_by(predicate)) {
                (None, _) => reasons.push(Self::reason(
                    predicate,
                    format!("links to {id}, which does not exist or you cannot see"),
                )),
                (Some(target), Some(kinds)) if !kinds.contains(&target.kind()) => {
                    reasons.push(Self::reason(
                        predicate,
                        format!("links to {id}, a {}", target.kind().term()),
                    ))
                }
                _ => {}
            }
        }
        if reasons.is_empty() {
            Ok(())
        } else {
            Err(Errors::validation_with(reasons))
        }
    }

    /// What every resource of a kind needs before it is stored; the rest is the doctor's.
    fn check_shape(kind: ResourceKind, description: &ResourceGraph) -> Outcome<()> {
        match kind.missing_essential(description) {
            Some((term, reason)) => Err(Self::invalid(term, reason)),
            None => Ok(()),
        }
    }

    /// Cached projections and auth's registry are stale after any write; neither failure fails it.
    async fn touched(&self) {
        self.ctx.cache.invalidate().await;
        self.ctx.registry().resync().await;
    }

    async fn emit(&self, resource: &CatalogResource, action: &str) {
        events::emit_action!(
            self.ctx.events,
            &resource.owner(),
            crate::EVENT_PREFIX,
            resource.kind().as_str(),
            action,
            &AdminDocument::render(resource)
        );
    }

    fn reason(path: &str, message: String) -> ErrorReason {
        ErrorReason {
            path: path.to_string(),
            code: 1001,
            message,
        }
    }

    fn invalid(path: &str, message: impl Into<String>) -> Errors {
        Errors::validation_with(vec![Self::reason(path, message.into())])
    }

    fn version_matches(resource: &CatalogResource, expected: Option<i64>) -> Outcome<()> {
        match expected {
            Some(v) if v != resource.version() => Err(CatalogErrors::stale(format!(
                "{} is at version {}, not {v}",
                resource.id(),
                resource.version()
            ))),
            _ => Ok(()),
        }
    }

    fn mint(command: &NewResourceCommand) -> Urn {
        command.id.clone().unwrap_or_else(|| command.kind.mint())
    }

    fn cursor(resource: &CatalogResource) -> String {
        Cursor::encode_composite(&resource.issued(), resource.id().as_ref())
    }

    fn format_error(reason: &str) -> Errors {
        Errors::format(BadFormat::Received, reason.to_string(), None)
    }

    /// The document read as a resource of `kind`; one with no `@type` is taken as one.
    async fn read_document(kind: ResourceKind, mut document: Value) -> Outcome<AdminDocument> {
        if let Value::Object(object) = &mut document {
            object.entry("@type").or_insert_with(|| json!(kind.term()));
        }
        let read = AdminDocument::read(document).await?;
        if read.kind != kind {
            return Err(Self::format_error(&format!(
                "a {} is expected, the document describes a {}",
                kind.term(),
                read.kind.term()
            )));
        }
        Ok(read)
    }
}

#[async_trait::async_trait]
impl ResourceServiceTrait for ResourceService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn list(
        &self,
        user: &UserInfo,
        filter: &ResourceFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<CatalogResource>> {
        filter.validate()?;
        let page = page.clamped();
        let scope = OwnerScope::seeing(user);
        let items = self.repo().get_page(&scope, filter, &page, sort).await?;
        let total = self.repo().count(&scope, filter).await?;
        Ok(Paginated::from_page(
            items,
            &page,
            Some(total),
            Self::cursor,
        ))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn batch(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        ids: &[Urn],
    ) -> Outcome<Vec<CatalogResource>> {
        Ok(self
            .repo()
            .get_batch(&OwnerScope::seeing(user), ids)
            .await?
            .into_iter()
            .filter(|r| r.kind() == kind)
            .collect())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get(&self, user: &UserInfo, kind: ResourceKind, id: &Urn) -> Outcome<CatalogResource> {
        self.seen(user, kind, id).await
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn main(&self, kind: ResourceKind) -> Outcome<Option<CatalogResource>> {
        self.repo().get_main(kind).await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn children(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        parent: &Urn,
    ) -> Outcome<Vec<CatalogResource>> {
        self.repo()
            .get_children(
                &OwnerScope::seeing(user),
                Some(kind),
                std::slice::from_ref(parent),
            )
            .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn distributions_of(
        &self,
        user: &UserInfo,
        datasets: &[Urn],
    ) -> Outcome<Vec<CatalogResource>> {
        self.repo()
            .get_children(
                &OwnerScope::seeing(user),
                Some(ResourceKind::Distribution),
                datasets,
            )
            .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn distribution_by_format(
        &self,
        user: &UserInfo,
        dataset: &Urn,
        format: &str,
    ) -> Outcome<CatalogResource> {
        self.repo()
            .get_distribution_by_format(&OwnerScope::seeing(user), dataset, format)
            .await?
            .ok_or_else(|| {
                Errors::missing_resource(
                    dataset.to_string(),
                    format!("the dataset has no distribution in format {format}"),
                    None,
                )
            })
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create(
        &self,
        user: &UserInfo,
        command: &NewResourceCommand,
    ) -> Outcome<CatalogResource> {
        let kind = command.kind;
        if kind == ResourceKind::Catalog
            && self.repo().get_main(ResourceKind::Catalog).await?.is_none()
        {
            return Err(CatalogErrors::conflict(
                "the connector's main catalog is created first",
            ));
        }
        Self::check_shape(kind, &command.description)?;
        let (catalog_id, dataset_id) = self.placement(user, kind, command.parent.as_ref()).await?;
        self.check_links(user, &command.description).await?;
        let owner = Owner::for_new(user, command.owner.clone(), command.visibility.clone());
        let resource = CatalogResource::create(
            Self::mint(command),
            kind,
            owner,
            catalog_id,
            dataset_id,
            false,
            command.description.clone(),
        );
        let stored = self.repo().insert(&resource).await?;
        self.touched().await;
        self.emit(&stored, "create").await;
        Ok(stored)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create_main(
        &self,
        user: &UserInfo,
        command: &NewResourceCommand,
    ) -> Outcome<CatalogResource> {
        user.require_root()?;
        let kind = command.kind;
        if !kind.has_main() {
            return Err(Self::format_error(
                "only catalogs and data services have a main one",
            ));
        }
        if let Some(existing) = self.repo().get_main(kind).await? {
            return Ok(existing);
        }
        Self::check_shape(kind, &command.description)?;
        let (catalog_id, dataset_id) = self.placement(user, kind, command.parent.as_ref()).await?;
        self.check_links(user, &command.description).await?;
        // The connector's: the root's and public, served over DSP to every peer.
        let resource = CatalogResource::create(
            Self::mint(command),
            kind,
            Owner::connector(),
            catalog_id,
            dataset_id,
            true,
            command.description.clone(),
        );
        let stored = self.repo().insert(&resource).await?;
        self.touched().await;
        self.emit(&stored, "create").await;
        Ok(stored)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn replace(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        id: &Urn,
        command: &ReplaceResourceCommand,
    ) -> Outcome<CatalogResource> {
        let current = self.acted_on(user, kind, id).await?;
        Self::version_matches(&current, command.expected_version)?;
        if let Some(parent) = &command.parent {
            if Some(parent) != current.parent_id() {
                return Err(Self::invalid(
                    "inCatalog",
                    "a resource keeps the parent it was created in",
                ));
            }
        }
        Self::check_shape(kind, &command.description)?;
        self.check_links(user, &command.description).await?;
        let mut revised = current.revise(command.description.clone());
        if let Some(visibility) = &command.visibility {
            let mut owner = current.owner().clone();
            owner.visibility = visibility.clone();
            revised = revised.with_owner(owner);
        }
        let stored = self
            .repo()
            .replace(&OwnerScope::acting(user), &revised, current.version())
            .await?;
        self.touched().await;
        self.emit(&stored, "edit").await;
        Ok(stored)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create_document(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        document: Value,
        visibility: Option<Visibility>,
        main: bool,
    ) -> Outcome<CatalogResource> {
        let document = Self::read_document(kind, document).await?;
        let mut parent = document.parent;
        if main && parent.is_none() && kind.parent() == Parent::Catalog {
            parent = self
                .repo()
                .get_main(ResourceKind::Catalog)
                .await?
                .map(|c| c.id().clone());
        }
        let command = NewResourceCommand {
            kind,
            id: document.id,
            owner: None,
            visibility,
            parent,
            description: document.description,
        };
        if main {
            self.create_main(user, &command).await
        } else {
            self.create(user, &command).await
        }
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn replace_document(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        id: &Urn,
        document: Value,
        visibility: Option<Visibility>,
        expected: Option<i64>,
    ) -> Outcome<CatalogResource> {
        let document = Self::read_document(kind, document).await?;
        if let Some(named) = document.id.as_ref().filter(|named| *named != id) {
            return Err(Self::format_error(&format!(
                "the document describes {named}, not {id}"
            )));
        }
        let command = ReplaceResourceCommand {
            description: document.description,
            parent: document.parent,
            visibility,
            expected_version: expected,
        };
        self.replace(user, kind, id, &command).await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn delete(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        id: &Urn,
        expected: Option<i64>,
    ) -> Outcome<()> {
        let current = self.acted_on(user, kind, id).await?;
        Self::version_matches(&current, expected)?;
        let deleted = match self.repo().delete(&OwnerScope::acting(user), id).await {
            Ok(deleted) => deleted,
            Err(e) => {
                let referrers = self.repo().referrers(id).await.unwrap_or_default();
                if referrers.is_empty() {
                    return Err(e);
                }
                let list: Vec<String> = referrers
                    .iter()
                    .map(|(subject, predicate)| format!("{subject} ({predicate})"))
                    .collect();
                return Err(CatalogErrors::conflict(format!(
                    "{id} is still linked from {}",
                    list.join(", ")
                )));
            }
        };
        self.touched().await;
        self.emit(&deleted, "delete").await;
        Ok(())
    }
}
