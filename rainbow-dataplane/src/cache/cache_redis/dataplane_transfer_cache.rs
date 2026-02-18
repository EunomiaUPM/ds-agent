use crate::cache::cache_traits::redis_cache_connector_trait::RedisCacheConnectorTrait;
use crate::cache::cache_traits::utils_trait::UtilsCacheTrait;
use crate::entities::dataplane_transfers::DataplaneTransferDto;

pub struct DataplaneTransferCacheForRedis {
    pub redis_connection: redis::aio::MultiplexedConnection,
}

impl DataplaneTransferCacheForRedis {
    pub fn new(redis_connection: redis::aio::MultiplexedConnection) -> Self {
        Self { redis_connection }
    }
}

impl UtilsCacheTrait for DataplaneTransferCacheForRedis {
    type Dto = DataplaneTransferDto;
}

impl RedisCacheConnectorTrait for DataplaneTransferCacheForRedis {
    type Dto = DataplaneTransferDto;
    fn get_conn(&self) -> redis::aio::MultiplexedConnection {
        self.redis_connection.clone()
    }
    fn get_entity_name(&self) -> &str {
        "transfers"
    }
}
