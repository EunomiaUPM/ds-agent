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

//! Provisioning over the resource service and the participant registry.

use std::sync::Arc;

use common::config::types::traits::MinKnownConfigTrait;
use common::config::services::traits::CatalogConfigTrait;
use ymir::config::types::HostType;
use ymir::errors::Outcome;
use ymir::types::oauth::{RoleTrait, UserInfo};

use crate::entities::commands::NewResourceCommand;
use crate::entities::graph::ResourceGraph;
use crate::entities::resource_kind::ResourceKind;
use crate::entities::vocab::Vocab;
use crate::services::main_catalog::{MainCatalog, MainCatalogServiceTrait};
use crate::setup::context::AppContext;

pub struct MainCatalogService {
    ctx: Arc<AppContext>,
}

impl MainCatalogService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    /// DSP endpoint of this connector; the main data service points to it.
    fn dsp_url(&self) -> String {
        format!(
            "{}{}",
            self.ctx.config.contracts().get_host(HostType::Http),
            common::dsp_common::paths::DSP_CURRENT
        )
    }

    fn command(kind: ResourceKind, parent: Option<urn::Urn>, description: ResourceGraph) -> NewResourceCommand {
        NewResourceCommand {
            kind,
            id: None,
            owner: None,
            visibility: None,
            parent,
            description,
        }
    }
}

#[async_trait::async_trait]
impl MainCatalogServiceTrait for MainCatalogService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn provision(&self, user: &UserInfo) -> Outcome<MainCatalog> {
        user.require_root()?;
        let resources = self.ctx.resources();
        let catalog = match resources.main(ResourceKind::Catalog).await? {
            Some(catalog) => catalog,
            None => {
                // The catalog publishes this connector's identity.
                let me = self.ctx.ports.auth.mates.get_me_mate().await?;
                let mut description = ResourceGraph::default();
                description.put(
                    Vocab::DSPACE_PARTICIPANT_ID,
                    vec![ResourceGraph::iri(&me.participant_id)],
                );
                resources
                    .create_main(user, &Self::command(ResourceKind::Catalog, None, description))
                    .await?
            }
        };
        let data_service = match resources.main(ResourceKind::DataService).await? {
            Some(service) => service,
            None => {
                let mut description = ResourceGraph::default();
                description.put(Vocab::DCAT_ENDPOINT_URL, vec![ResourceGraph::iri(&self.dsp_url())]);
                description.put(
                    Vocab::DCT_CONFORMS_TO,
                    vec![ResourceGraph::iri(Vocab::DSPACE)],
                );
                resources
                    .create_main(
                        user,
                        &Self::command(ResourceKind::DataService, Some(catalog.id().clone()), description),
                    )
                    .await?
            }
        };
        Ok(MainCatalog {
            catalog,
            data_service,
        })
    }
}
