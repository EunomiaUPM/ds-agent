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

use std::str::FromStr;
use std::sync::Arc;

use common::utils::get_urn_from_string;
use connector::ConnectorInstanceDto;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

use crate::protocols::dsp::facades::catalog_facade::CatalogFacadeTrait;
use crate::protocols::dsp::facades::negotiation_facade::NegotiationFacadeTrait;
use crate::protocols::dsp::services::connector_resolver::ConnectorResolverTrait;

pub struct ConnectorResolver {
    negotiation: Arc<dyn NegotiationFacadeTrait>,
    catalog: Arc<dyn CatalogFacadeTrait>,
}

impl ConnectorResolver {
    pub fn new(
        negotiation: Arc<dyn NegotiationFacadeTrait>,
        catalog: Arc<dyn CatalogFacadeTrait>,
    ) -> Self {
        Self {
            negotiation,
            catalog,
        }
    }
}

#[async_trait::async_trait]
impl ConnectorResolverTrait for ConnectorResolver {
    #[tracing::instrument(level = "info", skip_all, err, fields(agreement_id = %agreement_id))]
    async fn resolve_connector_by_agreement_id(
        &self,
        agreement_id: &Urn,
        formats: Option<&String>,
    ) -> Outcome<ConnectorInstanceDto> {
        let format = formats.ok_or_else(|| Errors::crazy("dct_formats is required", None))?;

        // The agreement fixes the tenant every later lookup is scoped to.
        let agreement = self.negotiation.get_agreement(agreement_id).await?;
        let tenant_id = agreement.inner.tenant_id.as_str();
        let target = get_urn_from_string(&agreement.inner.target)?;

        let dataset = self.catalog.get_dataset(tenant_id, &target).await?;
        let dataset_id = get_urn_from_string(&dataset.inner.id)?;

        let distribution = self
            .catalog
            .get_distribution_by_format(tenant_id, &dataset_id, format)
            .await?;
        let distribution_id = Urn::from_str(distribution.inner.id.as_str())?;

        self.catalog
            .get_instance_by_distribution(tenant_id, &distribution_id)
            .await?
            .ok_or_else(|| {
                Errors::crazy(
                    format!("No connector instance found for distribution {distribution_id}"),
                    None,
                )
            })
    }
}
