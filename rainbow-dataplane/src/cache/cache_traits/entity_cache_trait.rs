use crate::cache::cache_traits::redis_cache_connector_trait::RedisCacheConnectorTrait;
use crate::cache::cache_traits::utils_trait::UtilsCacheTrait;
use crate::cache::cache_traits::DESIRED_CACHE_TTL;
use serde::de::DeserializeOwned;
use serde::Serialize;
use urn::Urn;

#[async_trait::async_trait]
pub trait EntityCacheTrait<D>: Send + Sync {
    async fn get_single(&self, id: &Urn) -> anyhow::Result<Option<D>>;
    async fn set_single(&self, id: &Urn, model: &D) -> anyhow::Result<()>;
    async fn delete_single(&self, id: &Urn) -> anyhow::Result<()>;
    async fn get_main(&self) -> anyhow::Result<Option<D>>;
    async fn set_main(&self, id: &Urn, model: &D) -> anyhow::Result<()>;
    async fn get_collection(&self, limit: Option<u64>, page: Option<u64>)
        -> anyhow::Result<Vec<D>>;
    async fn add_to_collection(&self, id: &Urn, score: f64) -> anyhow::Result<()>;
    async fn remove_from_collection(&self, id: &Urn) -> anyhow::Result<()>;
    async fn get_batch(&self, ids: &Vec<Urn>) -> anyhow::Result<Vec<D>>;
}

#[async_trait::async_trait]
impl<T, D> EntityCacheTrait<D> for T
where
    T: RedisCacheConnectorTrait<Dto = D> + UtilsCacheTrait<Dto = D> + Send + Sync,
    D: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
{
    async fn get_single(&self, id: &Urn) -> anyhow::Result<Option<D>> {
        tracing::debug!("cache: get single");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        Self::hydrate_from_single_key(self.get_conn(), key).await
    }

    async fn set_single(&self, id: &Urn, model: &D) -> anyhow::Result<()> {
        tracing::debug!("cache: set single");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        let json = serde_json::to_string(model)?;
        redis::pipe()
            .atomic()
            .cmd("JSON.SET")
            .arg(&key)
            .arg("$")
            .arg(json)
            .cmd("EXPIRE")
            .arg(&key)
            .arg(DESIRED_CACHE_TTL)
            .query_async::<()>(&mut self.get_conn())
            .await?;
        Ok(())
    }

    async fn delete_single(&self, id: &Urn) -> anyhow::Result<()> {
        tracing::debug!("cache: delete single");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        let _: () = redis::cmd("DEL").arg(&key).query_async(&mut self.get_conn()).await?;
        Ok(())
    }

    async fn get_main(&self) -> anyhow::Result<Option<D>> {
        tracing::debug!("cache: get main");
        let main_key = self.format_key_name_main(self.get_entity_name());
        let target_key: Option<String> =
            redis::cmd("GET").arg(main_key).query_async(&mut self.get_conn()).await?;
        if let Some(key) = target_key {
            return Self::hydrate_from_single_key(self.get_conn(), key).await;
        }
        Ok(None)
    }

    async fn set_main(&self, id: &Urn, model: &D) -> anyhow::Result<()> {
        tracing::debug!("cache: set main");
        let main_key = self.format_key_name_main(self.get_entity_name());
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        self.set_single(id, model).await?;
        let _: () =
            redis::cmd("SET").arg(main_key).arg(&key).query_async(&mut self.get_conn()).await?;
        Ok(())
    }

    async fn get_collection(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<D>> {
        tracing::debug!("cache: get collection all");
        let collection_key = self.format_key_name_all(self.get_entity_name());
        let (start, stop) = self.compute_pagination_range(limit, page);
        let keys: Vec<String> = redis::cmd("ZREVRANGE")
            .arg(collection_key)
            .arg(start)
            .arg(stop)
            .query_async(&mut self.get_conn())
            .await?;

        Self::hydrate_from_multiple_keys(self.get_conn(), keys).await
    }

    async fn add_to_collection(&self, id: &Urn, score: f64) -> anyhow::Result<()> {
        tracing::debug!("cache: add to collection all");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        let collection_key = self.format_key_name_all(self.get_entity_name());
        let _: () = redis::cmd("ZADD")
            .arg(collection_key)
            .arg(score)
            .arg(key)
            .query_async(&mut self.get_conn())
            .await?;
        Ok(())
    }

    async fn remove_from_collection(&self, id: &Urn) -> anyhow::Result<()> {
        tracing::debug!("cache: remove from collection all");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        let collection_key = self.format_key_name_all(self.get_entity_name());
        let _: () = redis::cmd("ZREM")
            .arg(collection_key)
            .arg(key)
            .query_async(&mut self.get_conn())
            .await?;
        Ok(())
    }

    async fn get_batch(&self, ids: &Vec<Urn>) -> anyhow::Result<Vec<D>> {
        tracing::debug!("cache: get batch");
        let keys: Vec<String> =
            ids.iter().map(|id| self.format_key_name_with_id(self.get_entity_name(), id)).collect();
        Self::hydrate_from_multiple_keys(self.get_conn(), keys).await
    }
}
