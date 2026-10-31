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

//! Import over the profile registry and the resource repository: parse, validate, plan, check
//! the plan against the stored catalog, then one transactional write.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use common::eunomia::EunomiaProfile;
use common::rdf::shacl::ShapeSet;
use common::rdf::{RdfGraph, RdfSyntax};
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::oauth::{Owner, OwnerScope};
use ymir::types::oauth::UserInfo;

use crate::data::repo::resource::{ImportWrite, ResourceRepoTrait};
use crate::entities::admin_document::AdminDocument;
use crate::entities::commands::{ImportCatalogCommand, ImportMode};
use crate::entities::errors::CatalogErrors;
use crate::entities::findings::Severity;
use crate::entities::import_plan::{ImportPlan, PlannedResource};
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::{Parent, ResourceKind};
use crate::services::catalog_import::views::{ImportAction, ImportReport, ImportedResource};
use crate::services::catalog_import::CatalogImportServiceTrait;
use crate::setup::context::AppContext;

pub struct CatalogImportService {
    ctx: Arc<AppContext>,
}

impl CatalogImportService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    fn repo(&self) -> Arc<dyn ResourceRepoTrait> {
        self.ctx.repos.resource_repo()
    }

    /// The shapes of `profile`; a profile without them is no import format.
    fn shapes(&self, profile: &str) -> Outcome<ShapeSet> {
        let importable: Vec<&str> = self
            .ctx
            .profiles
            .all()
            .iter()
            .filter(|a| a.shapes().is_some())
            .map(|a| a.key())
            .collect();
        self.ctx
            .profiles
            .get(profile)
            .and_then(|adapter| adapter.shapes().cloned())
            .ok_or_else(|| {
                Self::bad(format!(
                    "{profile} is not an import profile; use one of {}",
                    importable.join(", ")
                ))
            })
    }

    /// The catalog named as target, if the caller sees it.
    async fn target(&self, user: &UserInfo, target: Option<&Urn>) -> Outcome<Option<Urn>> {
        let Some(target) = target else {
            return Ok(None);
        };
        self.repo()
            .get_by_id(&OwnerScope::seeing(user), target)
            .await?
            .filter(|c| c.kind() == ResourceKind::Catalog)
            .map(|c| Some(c.id().clone()))
            .ok_or_else(|| Self::bad(format!("target {target} is not a catalog you can see")))
    }

    /// What is stored under the plan's ids, and which of those the caller may change.
    async fn stored(
        &self,
        user: &UserInfo,
        plan: &ImportPlan,
    ) -> Outcome<(HashMap<String, CatalogResource>, HashSet<String>)> {
        let ids: Vec<Urn> = plan.resources.iter().map(|r| r.id.clone()).collect();
        let seen: HashMap<String, CatalogResource> = self
            .repo()
            .get_batch(&OwnerScope::seeing(user), &ids)
            .await?
            .into_iter()
            .map(|r| (r.id().to_string(), r))
            .collect();
        if seen.is_empty() {
            return Ok((seen, HashSet::new()));
        }
        let stored: Vec<Urn> = seen.values().map(|r| r.id().clone()).collect();
        let actable = self
            .repo()
            .get_batch(&OwnerScope::acting(user), &stored)
            .await?
            .into_iter()
            .map(|r| r.id().to_string())
            .collect();
        Ok((seen, actable))
    }

    /// A planned resource over a stored one must be the same kind, under the same parent, and
    /// the caller's to change.
    fn check_stored(
        plan: &mut ImportPlan,
        mode: ImportMode,
        seen: &HashMap<String, CatalogResource>,
        actable: &HashSet<String>,
    ) {
        let mut issues = Vec::new();
        for planned in &plan.resources {
            let Some(stored) = seen.get(&planned.id.to_string()) else {
                continue;
            };
            let problem = if stored.kind() != planned.kind {
                Some(format!(
                    "{} is stored as a {}",
                    planned.id,
                    stored.kind().term()
                ))
            } else if mode == ImportMode::Create {
                Some(format!("{} is already in the catalog", planned.id))
            } else if stored.is_main() {
                Some("the connector's main resources are not imported over".to_string())
            } else if !actable.contains(&planned.id.to_string()) {
                Some(format!("{} is stored and you cannot change it", planned.id))
            } else if stored.parent_id() != planned.parent.as_ref() {
                Some(format!("{} keeps the parent it was created in", planned.id))
            } else {
                None
            };
            if let Some(message) = problem {
                issues.push((planned.source.clone(), message));
            }
        }
        for (source, message) in issues {
            plan.issue(Severity::Violation, &source, "@id", message);
        }
    }

    /// Every `urn:` link must reach a resource of the plan or one the caller sees, of the kind
    /// the link expects.
    async fn check_links(&self, user: &UserInfo, plan: &mut ImportPlan) -> Outcome<()> {
        let planned: HashMap<String, ResourceKind> = plan
            .resources
            .iter()
            .map(|r| (r.id.to_string(), r.kind))
            .collect();
        let links: Vec<(String, &'static str, String)> = plan
            .resources
            .iter()
            .flat_map(|r| {
                r.description
                    .local_links()
                    .into_iter()
                    .map(move |(predicate, id)| (r.source.clone(), predicate, id))
            })
            .collect();
        let outside: Vec<Urn> = links
            .iter()
            .filter(|(_, _, id)| !planned.contains_key(id))
            .filter_map(|(_, _, id)| id.parse().ok())
            .collect::<HashSet<Urn>>()
            .into_iter()
            .collect();
        let stored: HashMap<String, ResourceKind> = if outside.is_empty() {
            HashMap::new()
        } else {
            self.repo()
                .get_batch(&OwnerScope::seeing(user), &outside)
                .await?
                .into_iter()
                .map(|r| (r.id().to_string(), r.kind()))
                .collect()
        };
        for (source, predicate, id) in links {
            match planned.get(&id).or_else(|| stored.get(&id)) {
                None => plan.issue(
                    Severity::Violation,
                    &source,
                    predicate,
                    format!("links to {id}, which is neither in the document nor in the catalog"),
                ),
                Some(kind)
                    if ResourceKind::linked_by(predicate)
                        .is_some_and(|kinds| !kinds.contains(kind)) =>
                {
                    plan.issue(
                        Severity::Violation,
                        &source,
                        predicate,
                        format!("links to {id}, a {}", kind.term()),
                    )
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// The writes of the plan: new resources owned by the caller, stored ones one version on.
    fn writes(
        plan: &ImportPlan,
        seen: &HashMap<String, CatalogResource>,
        owner: &Owner,
        command: &ImportCatalogCommand,
    ) -> Vec<ImportWrite> {
        plan.resources
            .iter()
            .map(|planned| match seen.get(&planned.id.to_string()) {
                Some(stored) => {
                    let mut revised = stored.reimported(planned.description.clone());
                    if let Some(visibility) = &command.visibility {
                        let mut owner = stored.owner().clone();
                        owner.visibility = visibility.clone();
                        revised = revised.with_owner(owner);
                    }
                    ImportWrite {
                        resource: revised,
                        expected: Some(stored.version()),
                    }
                }
                None => {
                    let (catalog, dataset) = Self::placement(planned);
                    ImportWrite {
                        resource: CatalogResource::imported(
                            planned.id.clone(),
                            planned.kind,
                            owner.clone(),
                            catalog,
                            dataset,
                            planned.description.clone(),
                        ),
                        expected: None,
                    }
                }
            })
            .collect()
    }

    fn placement(planned: &PlannedResource) -> (Option<Urn>, Option<Urn>) {
        match planned.kind.parent() {
            Parent::Catalog => (planned.parent.clone(), None),
            Parent::Dataset => (None, planned.parent.clone()),
            Parent::None => (None, None),
        }
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

    fn bad(reason: impl Into<String>) -> Errors {
        Errors::format(BadFormat::Received, reason.into(), None)
    }
}

#[async_trait::async_trait]
impl CatalogImportServiceTrait for CatalogImportService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id(), profile = %command.profile))]
    async fn import(
        &self,
        user: &UserInfo,
        syntax: RdfSyntax,
        body: &[u8],
        command: &ImportCatalogCommand,
    ) -> Outcome<ImportReport> {
        let shapes = self.shapes(&command.profile)?;
        let graph = RdfGraph::parse(syntax, body, EunomiaProfile::shared()).await?;
        if graph.is_empty() {
            return Err(Self::bad("the document has no triples"));
        }
        let validation = shapes.validate(&graph, &command.checks)?.attributed(&graph);
        let main = self
            .repo()
            .get_main(ResourceKind::Catalog)
            .await?
            .ok_or_else(|| {
                CatalogErrors::conflict("the connector's main catalog is created first")
            })?;
        let target = self.target(user, command.target.as_ref()).await?;
        let mut plan = ImportPlan::build(&graph, target.as_ref(), main.id());
        let (seen, actable) = self.stored(user, &plan).await?;
        Self::check_stored(&mut plan, command.mode, &seen, &actable);
        self.check_links(user, &mut plan).await?;

        let resources: Vec<ImportedResource> = plan
            .resources
            .iter()
            .map(|r| {
                let action = if seen.contains_key(&r.id.to_string()) {
                    ImportAction::Update
                } else {
                    ImportAction::Create
                };
                ImportedResource::assemble(r, action)
            })
            .collect();
        let updated = resources
            .iter()
            .filter(|r| r.action == ImportAction::Update)
            .count();
        let conforms = validation.conforms && plan.is_clean();
        let mut report = ImportReport {
            profile: command.profile.clone(),
            dry_run: command.dry_run,
            conforms,
            imported: false,
            validation,
            issues: plan.issues.clone(),
            created: resources.len() - updated,
            updated,
            resources,
        };
        if command.dry_run || !conforms {
            return Ok(report);
        }

        let owner = Owner::for_new(user, None, command.visibility.clone());
        let writes = Self::writes(&plan, &seen, &owner, command);
        self.repo()
            .import(&OwnerScope::acting(user), &writes)
            .await?;
        self.ctx.cache.invalidate().await;
        for write in &writes {
            let action = if write.expected.is_some() {
                "edit"
            } else {
                "create"
            };
            self.emit(&write.resource, action).await;
        }
        report.imported = true;
        Ok(report)
    }
}
