/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

pub(crate) mod distributions;

use crate::data::entities::distribution;
use crate::data::entities::distribution::{EditDistributionModel, Model, NewDistributionModel};
use serde::{Deserialize, Serialize};
use urn::Urn;
use ymir::errors::Outcome;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DistributionDto {
    #[serde(flatten)]
    pub inner: distribution::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewDistributionDto {
    pub id: Option<Urn>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub dct_formats: Option<String>,
    pub dcat_access_service: String,
    pub dataset_id: Urn,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditDistributionDto {
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub dcat_access_service: Option<String>,
}

impl From<NewDistributionDto> for NewDistributionModel {
    fn from(dto: NewDistributionDto) -> Self {
        Self {
            id: dto.id,
            dct_title: dto.dct_title,
            dct_description: dto.dct_description,
            dct_formats: dto.dct_formats,
            dcat_access_service: dto.dcat_access_service,
            dataset_id: dto.dataset_id,
        }
    }
}

impl From<EditDistributionDto> for EditDistributionModel {
    fn from(dto: EditDistributionDto) -> Self {
        Self {
            dct_title: dto.dct_title,
            dct_description: dto.dct_description,
            dcat_access_service: dto.dcat_access_service,
        }
    }
}

impl From<distribution::Model> for DistributionDto {
    fn from(value: Model) -> Self {
        Self { inner: value }
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait DistributionEntityTrait: Send + Sync {
    async fn get_all_distributions(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<DistributionDto>>;
    async fn get_batch_distributions(&self, ids: &Vec<Urn>)
        -> Outcome<Vec<DistributionDto>>;

    async fn get_distributions_by_dataset_id(
        &self,
        dataset_id: &Urn,
    ) -> Outcome<Vec<DistributionDto>>;
    async fn get_distribution_by_dataset_id_and_dct_format(
        &self,
        dataset_id: &Urn,
        dct_formats: &String,
    ) -> Outcome<DistributionDto>;
    async fn get_distribution_by_id(
        &self,
        distribution_id: &Urn,
    ) -> Outcome<Option<DistributionDto>>;
    async fn put_distribution_by_id(
        &self,
        distribution_id: &Urn,
        edit_distribution_model: &EditDistributionDto,
    ) -> Outcome<DistributionDto>;
    async fn create_distribution(
        &self,
        new_distribution_model: &NewDistributionDto,
    ) -> Outcome<DistributionDto>;
    async fn delete_distribution_by_id(&self, distribution_id: &Urn) -> Outcome<()>;
}
