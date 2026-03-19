use rainbow_connector::ConnectorInstanceDto;
use urn::Urn;
use ymir::errors::Outcome;

pub mod data_service_resolver_facade;

#[async_trait::async_trait]
#[allow(unused)]
pub trait DataServiceFacadeTrait: Send + Sync {
    async fn resolve_connector_by_agreement_id(
        &self,
        agreement_id: &Urn,
        formats: Option<&String>,
    ) -> Outcome<ConnectorInstanceDto>;
}
