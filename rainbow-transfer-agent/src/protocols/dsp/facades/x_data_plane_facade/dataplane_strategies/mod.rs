pub(crate) mod consumer_pull_strategy;
pub(crate) mod consumer_push_strategy;
pub(crate) mod provider_pull_strategy;
pub(crate) mod provider_push_strategy;

#[async_trait::async_trait]
pub trait DataPlaneStrategyTrait: Send + Sync + Sized {}
