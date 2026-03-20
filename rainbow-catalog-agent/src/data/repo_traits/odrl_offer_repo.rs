use crate::data::entities::odrl_offer;
use crate::data::entities::odrl_offer::NewOdrlOfferModel;
use crate::data::repo_traits::catalog_db_errors::CatalogAgentRepoErrors;
use urn::Urn;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait OdrlOfferRepositoryTrait: Send + Sync {
    async fn get_all_odrl_offers(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<odrl_offer::Model>>;
    async fn get_batch_odrl_offers(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<odrl_offer::Model>>;
    async fn get_all_odrl_offers_by_entity(
        &self,
        entity: &Urn,
    ) -> Outcome<Vec<odrl_offer::Model>>;
    async fn get_odrl_offer_by_id(
        &self,
        odrl_offer_id: &Urn,
    ) -> Outcome<Option<odrl_offer::Model>>;
    async fn create_odrl_offer(
        &self,
        new_odrl_offer_model: &NewOdrlOfferModel,
    ) -> Outcome<odrl_offer::Model>;
    async fn delete_odrl_offer_by_id(
        &self,
        odrl_offer_id: &Urn,
    ) -> Outcome<()>;
    async fn delete_odrl_offers_by_entity(
        &self,
        entity_id: &Urn,
    ) -> Outcome<()>;
}
