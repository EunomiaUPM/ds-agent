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

use thiserror::Error;
use ymir::errors::RepoIntoErrors;

#[derive(Error, Debug)]
pub enum CatalogAgentRepoErrors {
    #[error("Catalog Repo error: {0}")]
    CatalogRepoErrors(CatalogRepoErrors),
    #[error("Data Service Repo error: {0}")]
    DataServiceRepoErrors(DataServiceRepoErrors),
    #[error("Dataset Repo error: {0}")]
    DatasetRepoErrors(DatasetRepoErrors),
    #[error("Distribution Repo error: {0}")]
    DistributionRepoErrors(DistributionRepoErrors),
    #[error("Odrl Offer Repo error: {0}")]
    OdrlOfferRepoErrors(OdrlOfferRepoErrors),
    #[error("Policy Templates Repo error: {0}")]
    PolicyTemplatesRepoErrors(PolicyTemplatesRepoErrors),
}

#[derive(Error, Debug)]
pub enum CatalogRepoErrors {
    #[error("Catalog not found")]
    CatalogNotFound,
    #[error("Error fetching catalog. {0}")]
    ErrorFetchingCatalog(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating catalog. {0}")]
    ErrorCreatingCatalog(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting catalog. {0}")]
    ErrorDeletingCatalog(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating catalog. {0}")]
    ErrorUpdatingCatalog(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Error, Debug)]
pub enum DataServiceRepoErrors {
    #[error("DataService not found")]
    DataServiceNotFound,
    #[error("Error fetching data service. {0}")]
    ErrorFetchingDataService(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating data service. {0}")]
    ErrorCreatingDataService(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting data service. {0}")]
    ErrorDeletingDataService(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating data service. {0}")]
    ErrorUpdatingDataService(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Error, Debug)]
pub enum DatasetRepoErrors {
    #[error("Dataset not found")]
    DatasetNotFound,
    #[error("Error fetching dataset. {0}")]
    ErrorFetchingDataset(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating dataset. {0}")]
    ErrorCreatingDataset(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting dataset. {0}")]
    ErrorDeletingDataset(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating dataset. {0}")]
    ErrorUpdatingDataset(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Error, Debug)]
pub enum DistributionRepoErrors {
    #[error("Distribution not found")]
    DistributionNotFound,
    #[error("Error fetching distribution. {0}")]
    ErrorFetchingDistribution(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating distribution. {0}")]
    ErrorCreatingDistribution(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting distribution. {0}")]
    ErrorDeletingDistribution(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating distribution. {0}")]
    ErrorUpdatingDistribution(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Error, Debug)]
pub enum OdrlOfferRepoErrors {
    #[error("OdrlOffer not found")]
    OdrlOfferNotFound,
    #[error("Error fetching odrl offer. {0}")]
    ErrorFetchingOdrlOffer(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating odrl offer. {0}")]
    ErrorCreatingOdrlOffer(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting odrl offer. {0}")]
    ErrorDeletingOdrlOffer(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating odrl offer. {0}")]
    ErrorUpdatingOdrlOffer(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error fetching offer ids. {missing_ids:?}")]
    SomeOdrlOffersNotFound { missing_ids: String },
}

#[derive(Error, Debug)]
pub enum PolicyTemplatesRepoErrors {
    #[error("PolicyTemplate not found")]
    PolicyTemplateNotFound,
    #[error("Error fetching policy template. {0}")]
    ErrorFetchingPolicyTemplate(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating policy template. {0}")]
    ErrorCreatingPolicyTemplate(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting policy template. {0}")]
    ErrorDeletingPolicyTemplate(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for CatalogAgentRepoErrors {}
impl RepoIntoErrors for CatalogRepoErrors {}
impl RepoIntoErrors for DataServiceRepoErrors {}
impl RepoIntoErrors for DatasetRepoErrors {}
impl RepoIntoErrors for DistributionRepoErrors {}
impl RepoIntoErrors for OdrlOfferRepoErrors {}
impl RepoIntoErrors for PolicyTemplatesRepoErrors {}
