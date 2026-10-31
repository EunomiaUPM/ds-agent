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

//! Persistence of policy templates, one row per template version, and of which versions each
//! one references.

use ymir::oauth::OwnerScope;
use ymir::paginated_spec::{Page, Sort};
use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::entities::commands::NewPolicyTemplateCommand;
use crate::entities::filters::PolicyTemplateFilter;
use crate::entities::policy_templates::PolicyTemplateDto;

/// Template versions within the owner scope each call gives.
#[mockall::automock]
#[async_trait::async_trait]
pub trait PolicyTemplateRepoTrait: Send + Sync {
    async fn get_page(
        &self,
        scope: &OwnerScope,
        filter: &PolicyTemplateFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<PolicyTemplateDto>>;
    async fn count(&self, scope: &OwnerScope, filter: &PolicyTemplateFilter) -> Outcome<u64>;
    async fn get_batch(&self, scope: &OwnerScope, ids: &[String]) -> Outcome<Vec<PolicyTemplateDto>>;
    /// Every version of the template.
    async fn get_versions(&self, scope: &OwnerScope, id: &str) -> Outcome<Vec<PolicyTemplateDto>>;
    async fn get_version(
        &self,
        scope: &OwnerScope,
        id: &str,
        version: &str,
    ) -> Outcome<Option<PolicyTemplateDto>>;
    /// The versions named by `(id, version)` among those the scope sees, in one query.
    async fn get_by_keys(
        &self,
        scope: &OwnerScope,
        keys: &[(String, String)],
    ) -> Outcome<Vec<PolicyTemplateDto>>;
    /// Stores the version and the versions it references, in one transaction.
    async fn insert(&self, command: &NewPolicyTemplateCommand) -> Outcome<PolicyTemplateDto>;
    /// `id@version` of every template version, whoever owns it, that references this one.
    async fn dependents(&self, id: &str, version: &str) -> Outcome<Vec<String>>;
    async fn delete_version(&self, scope: &OwnerScope, id: &str, version: &str) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum PolicyTemplateRepoErrors {
    #[error("Policy template not found")]
    NotFound,
    #[error("Invalid pagination cursor")]
    InvalidCursor,
    #[error("Error fetching policy templates. {0}")]
    Fetching(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating policy template. {0}")]
    Creating(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting policy template. {0}")]
    Deleting(Box<dyn std::error::Error + Send + Sync>),
    #[error("Stored template is not valid. {0}")]
    Decoding(String),
}

impl RepoIntoErrors for PolicyTemplateRepoErrors {}
