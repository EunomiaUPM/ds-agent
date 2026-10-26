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

//! Profiles are adapters over the neutral catalog: each one projects a loaded slice of the graph
//! in its own vocabulary and diagnoses it against its own rules. A profile is a module of its
//! own; registering it mounts its routes and makes the doctor see it.

pub mod dcat3;
pub mod dcat_ap;
pub mod dsp;
pub mod shared;

use std::sync::{Arc, RwLock};

use axum::Router;
use common::boot::seeders::BootSeeder;
use common::boot::workers::BackgroundWorker;
use common::module_loader::module_group::ModuleGroup;
use common::module_loader::service_module::ServiceModuleTrait;
use serde::Serialize;
use serde_json::Value;
use tonic::service::RoutesBuilder;
use ymir::errors::Outcome;

use crate::entities::resource::CatalogResource;
use crate::entities::resource_set::ResourceSet;
use crate::setup::context::AppContext;

/// One profile of the catalog: what it is, how it writes the graph and what it requires.
pub trait ProfileAdapterTrait: Send + Sync {
    /// The key routes and the doctor know it by (`dsp`, `dcat3`, `dcat-ap`).
    fn key(&self) -> &'static str;
    /// The IRI identifying the profile, sent as `Link: <iri>; rel="profile"`.
    fn profile_iri(&self) -> &'static str;
    /// The JSON-LD context document its output uses.
    fn context(&self) -> Value;
    /// `root` and what hangs from it, in the profile's vocabulary.
    fn project(&self, set: &ResourceSet, root: &CatalogResource) -> Outcome<Value>;
    /// Whether `root` (with `deep`, its whole closure) conforms, and the findings.
    fn diagnose(&self, set: &ResourceSet, root: &CatalogResource, deep: bool) -> Outcome<ProfileDiagnosis>;
}

/// A profile as a composable module: its own routes, plus the adapter the registry keeps.
pub trait ProfileModuleTrait: ServiceModuleTrait {
    fn adapter(&self) -> Arc<dyn ProfileAdapterTrait>;
}

/// The adapters registered in this process, by key.
#[derive(Default)]
pub struct ProfileRegistry {
    adapters: RwLock<Vec<Arc<dyn ProfileAdapterTrait>>>,
}

impl ProfileRegistry {
    /// Adds an adapter; one with the same key is replaced.
    pub fn register(&self, adapter: Arc<dyn ProfileAdapterTrait>) {
        let mut adapters = self.adapters.write().unwrap_or_else(|e| e.into_inner());
        adapters.retain(|a| a.key() != adapter.key());
        adapters.push(adapter);
    }

    pub fn get(&self, key: &str) -> Option<Arc<dyn ProfileAdapterTrait>> {
        self.read().iter().find(|a| a.key() == key).cloned()
    }

    /// Every adapter, in registration order.
    pub fn all(&self) -> Vec<Arc<dyn ProfileAdapterTrait>> {
        self.read().clone()
    }

    pub fn keys(&self) -> Vec<&'static str> {
        self.read().iter().map(|a| a.key()).collect()
    }

    fn read(&self) -> std::sync::RwLockReadGuard<'_, Vec<Arc<dyn ProfileAdapterTrait>>> {
        self.adapters.read().unwrap_or_else(|e| e.into_inner())
    }
}

/// The profile modules of the agent: each one registered here is mounted and known to the
/// registry, so adding a profile is one more `register`.
pub struct ProfileModules {
    ctx: Arc<AppContext>,
    group: ModuleGroup,
}

impl ProfileModules {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self {
            ctx,
            group: ModuleGroup::new("catalog-profiles"),
        }
    }

    pub fn register(mut self, module: impl ProfileModuleTrait + 'static) -> Self {
        self.ctx.profiles.register(module.adapter());
        self.group = self.group.register(module);
        self
    }
}

impl ServiceModuleTrait for ProfileModules {
    fn name(&self) -> &'static str {
        "catalog-profiles"
    }

    fn http(&self) -> Option<(String, Router)> {
        self.group.http()
    }

    fn grpc(&self, routes: &mut RoutesBuilder) {
        self.group.grpc(routes);
    }

    fn workers(&self) -> Vec<Box<dyn BackgroundWorker>> {
        self.group.workers()
    }

    fn seeders(&self) -> Vec<Box<dyn BootSeeder>> {
        self.group.seeders()
    }
}

/// How bad a finding is: a violation breaks conformance, the rest do not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Violation,
    Warning,
    Info,
}

/// One thing a profile found about one resource.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub severity: Severity,
    pub resource: String,
    /// The property concerned, as an IRI, when there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,
    pub message: String,
}

/// A profile's verdict on a resource.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDiagnosis {
    pub profile: String,
    pub profile_iri: String,
    pub compliant: bool,
    pub violations: usize,
    pub warnings: usize,
    pub findings: Vec<Finding>,
}

impl ProfileDiagnosis {
    /// The verdict the findings give: compliant when none is a violation.
    pub fn from_findings(adapter: &dyn ProfileAdapterTrait, findings: Vec<Finding>) -> Self {
        let count = |s: Severity| findings.iter().filter(|f| f.severity == s).count();
        let violations = count(Severity::Violation);
        Self {
            profile: adapter.key().to_string(),
            profile_iri: adapter.profile_iri().to_string(),
            compliant: violations == 0,
            violations,
            warnings: count(Severity::Warning),
            findings,
        }
    }
}
