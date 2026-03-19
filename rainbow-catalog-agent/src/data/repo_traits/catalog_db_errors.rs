use anyhow::Error;
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
    ErrorFetchingCatalog(Error),
    #[error("Error creating catalog. {0}")]
    ErrorCreatingCatalog(Error),
    #[error("Error deleting catalog. {0}")]
    ErrorDeletingCatalog(Error),
    #[error("Error updating catalog. {0}")]
    ErrorUpdatingCatalog(Error),
}

#[derive(Error, Debug)]
pub enum DataServiceRepoErrors {
    #[error("DataService not found")]
    DataServiceNotFound,
    #[error("Error fetching data service. {0}")]
    ErrorFetchingDataService(Error),
    #[error("Error creating data service. {0}")]
    ErrorCreatingDataService(Error),
    #[error("Error deleting data service. {0}")]
    ErrorDeletingDataService(Error),
    #[error("Error updating data service. {0}")]
    ErrorUpdatingDataService(Error),
}

#[derive(Error, Debug)]
pub enum DatasetRepoErrors {
    #[error("Dataset not found")]
    DatasetNotFound,
    #[error("Error fetching dataset. {0}")]
    ErrorFetchingDataset(Error),
    #[error("Error creating dataset. {0}")]
    ErrorCreatingDataset(Error),
    #[error("Error deleting dataset. {0}")]
    ErrorDeletingDataset(Error),
    #[error("Error updating dataset. {0}")]
    ErrorUpdatingDataset(Error),
}

#[derive(Error, Debug)]
pub enum DistributionRepoErrors {
    #[error("Distribution not found")]
    DistributionNotFound,
    #[error("Error fetching distribution. {0}")]
    ErrorFetchingDistribution(Error),
    #[error("Error creating distribution. {0}")]
    ErrorCreatingDistribution(Error),
    #[error("Error deleting distribution. {0}")]
    ErrorDeletingDistribution(Error),
    #[error("Error updating distribution. {0}")]
    ErrorUpdatingDistribution(Error),
}

#[derive(Error, Debug)]
pub enum OdrlOfferRepoErrors {
    #[error("OdrlOffer not found")]
    OdrlOfferNotFound,
    #[error("Error fetching odrl offer. {0}")]
    ErrorFetchingOdrlOffer(Error),
    #[error("Error creating odrl offer. {0}")]
    ErrorCreatingOdrlOffer(Error),
    #[error("Error deleting odrl offer. {0}")]
    ErrorDeletingOdrlOffer(Error),
    #[error("Error updating odrl offer. {0}")]
    ErrorUpdatingOdrlOffer(Error),
    #[error("Error fetching offer ids. {missing_ids:?}")]
    SomeOdrlOffersNotFound { missing_ids: String },
}

#[derive(Error, Debug)]
pub enum PolicyTemplatesRepoErrors {
    #[error("PolicyTemplate not found")]
    PolicyTemplateNotFound,
    #[error("Error fetching policy template. {0}")]
    ErrorFetchingPolicyTemplate(Error),
    #[error("Error creating policy template. {0}")]
    ErrorCreatingPolicyTemplate(Error),
    #[error("Error deleting policy template. {0}")]
    ErrorDeletingPolicyTemplate(Error),
}

impl RepoIntoErrors for CatalogAgentRepoErrors {}
impl RepoIntoErrors for CatalogRepoErrors {}
impl RepoIntoErrors for DataServiceRepoErrors {}
impl RepoIntoErrors for DatasetRepoErrors {}
impl RepoIntoErrors for DistributionRepoErrors {}
impl RepoIntoErrors for OdrlOfferRepoErrors {}
impl RepoIntoErrors for PolicyTemplatesRepoErrors {}
