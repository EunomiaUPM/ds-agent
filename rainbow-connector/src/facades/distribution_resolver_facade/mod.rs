/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use sea_orm::prelude::DateTimeWithTimeZone;
use serde::Deserialize;

pub mod data_service_resolver_facade;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Distribution {
    pub id: String,
    pub dct_issued: DateTimeWithTimeZone,
    pub dct_modified: Option<DateTimeWithTimeZone>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub dcat_access_service: String,
    pub dataset_id: String,
    pub dct_format: Option<String>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait DistributionFacadeTrait: Send + Sync {
    async fn resolve_distribution_by_id(
        &self,
        distribution_id: &String,
    ) -> anyhow::Result<Distribution>;
}
