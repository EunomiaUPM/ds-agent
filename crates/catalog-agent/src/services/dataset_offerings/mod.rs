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

//! Publishing a dataset with its distribution and policy as one use case.

pub mod service;

use crate::entities::dataset_offerings::{DatasetOfferingDto, NewDatasetOfferingDto};
use common::auth::AccessScope;
use ymir::errors::Outcome;

#[mockall::automock]
#[async_trait::async_trait]
pub trait DatasetOfferingServiceTrait: Send + Sync {
    /// Creates dataset, distribution and policy; a failure after the dataset removes it again.
    async fn create_offering(
        &self,
        scope: &AccessScope,
        offering: &NewDatasetOfferingDto,
    ) -> Outcome<DatasetOfferingDto>;
}
