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

//! Closures in five queries at most: the roots, their members, the distributions, the linked
//! resources and the offers.

use std::str::FromStr;
use std::sync::Arc;

use urn::Urn;
use ymir::errors::{Errors, Outcome};
use ymir::oauth::OwnerScope;

use crate::entities::filters::ResourceFilter;
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::ResourceKind;
use crate::entities::resource_set::ResourceSet;
use crate::services::closure::ClosureServiceTrait;
use crate::setup::context::AppContext;

pub struct ClosureService {
    ctx: Arc<AppContext>,
}

impl ClosureService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    /// Members of the catalogs, then the distributions of the datasets among them.
    async fn members(
        &self,
        scope: &OwnerScope,
        set: &mut ResourceSet,
        catalogs: &[Urn],
    ) -> Outcome<()> {
        let repo = self.ctx.repos.resource_repo();
        let members = repo.get_children(scope, None, catalogs).await?;
        let datasets: Vec<Urn> = members
            .iter()
            .filter(|m| m.kind() == ResourceKind::Dataset)
            .map(|m| m.id().clone())
            .collect();
        set.extend(members);
        self.distributions(scope, set, &datasets).await
    }

    async fn distributions(
        &self,
        scope: &OwnerScope,
        set: &mut ResourceSet,
        datasets: &[Urn],
    ) -> Outcome<()> {
        let distributions = self
            .ctx
            .repos
            .resource_repo()
            .get_children(scope, Some(ResourceKind::Distribution), datasets)
            .await?;
        set.extend(distributions);
        Ok(())
    }

    /// The resources the set links to that it does not hold yet, then every offer on the set.
    async fn complete(&self, scope: &OwnerScope, set: &mut ResourceSet) -> Outcome<()> {
        let missing: Vec<Urn> = set
            .iter()
            .flat_map(|r| r.description().local_links())
            .map(|(_, id)| id)
            .filter(|id| !set.contains(id))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .filter_map(|id| Urn::from_str(&id).ok())
            .collect();
        if !missing.is_empty() {
            let linked = self
                .ctx
                .repos
                .resource_repo()
                .get_batch(scope, &missing)
                .await?;
            set.extend(linked);
        }
        let ids: Vec<Urn> = set.iter().map(|r| r.id().clone()).collect();
        let offers = self
            .ctx
            .repos
            .odrl_offer_repo()
            .get_by_entities(scope, &ids)
            .await?;
        set.insert_offers(offers);
        Ok(())
    }
}

#[async_trait::async_trait]
impl ClosureServiceTrait for ClosureService {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn main_catalog(&self, scope: &OwnerScope) -> Outcome<(CatalogResource, ResourceSet)> {
        let repo = self.ctx.repos.resource_repo();
        let main = repo.get_main(ResourceKind::Catalog).await?.ok_or_else(|| {
            Errors::missing_resource("main", "the connector has no main catalog", None)
        })?;
        let mut set = ResourceSet::new();
        set.insert(main.clone());
        let mut subs = ResourceFilter::of(ResourceKind::Catalog);
        subs.main = Some(false);
        set.extend(repo.get_all(scope, &subs).await?);
        let catalogs: Vec<Urn> = set.iter().map(|c| c.id().clone()).collect();
        self.members(scope, &mut set, &catalogs).await?;
        self.complete(scope, &mut set).await?;
        Ok((main, set))
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn resource(
        &self,
        scope: &OwnerScope,
        id: &Urn,
    ) -> Outcome<Option<(CatalogResource, ResourceSet)>> {
        let Some(root) = self.ctx.repos.resource_repo().get_by_id(scope, id).await? else {
            return Ok(None);
        };
        if root.kind() == ResourceKind::Catalog && root.is_main() {
            return self.main_catalog(scope).await.map(Some);
        }
        let mut set = ResourceSet::new();
        set.insert(root.clone());
        match root.kind() {
            ResourceKind::Catalog => {
                self.members(scope, &mut set, std::slice::from_ref(id))
                    .await?
            }
            ResourceKind::Dataset => {
                self.distributions(scope, &mut set, std::slice::from_ref(id))
                    .await?
            }
            ResourceKind::Distribution => {
                // Its dataset, for the schema it inherits; not part of the projection.
                if let Some(dataset) = root.parent_id() {
                    let repo = self.ctx.repos.resource_repo();
                    if let Some(parent) = repo.get_by_id(scope, dataset).await? {
                        set.insert_parent(parent);
                    }
                }
            }
            _ => {}
        }
        self.complete(scope, &mut set).await?;
        Ok(Some((root, set)))
    }
}
