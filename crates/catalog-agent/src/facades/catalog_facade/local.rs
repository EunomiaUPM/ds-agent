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

use common::oauth::UserInfo;
use connector::CatalogFacadeTrait;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

use crate::services::distributions::DistributionServiceTrait;

/// Distribution checks for the connector, which always runs inside the catalog's process.
pub struct CatalogLocalFacade {
    distributions: Arc<dyn DistributionServiceTrait>,
}

impl CatalogLocalFacade {
    pub fn new(distributions: Arc<dyn DistributionServiceTrait>) -> Self {
        Self { distributions }
    }
}

#[async_trait::async_trait]
impl CatalogFacadeTrait for CatalogLocalFacade {
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "catalog", user = %user.id())
    )]
    async fn resolve_distribution_by_id(
        &self,
        user: &UserInfo,
        distribution_id: &str,
    ) -> Outcome<()> {
        let urn = Urn::from_str(distribution_id)?;
        match self
            .distributions
            .get_distribution_by_id(user, &urn)
            .await
        {
            Ok(_) => Ok(()),
            Err(Errors::MissingResourceError { .. }) => Err(Errors::missing_resource(
                distribution_id,
                "Distribution not found among those the user sees",
                None,
            )),
            Err(e) => Err(e),
        }
    }
}
