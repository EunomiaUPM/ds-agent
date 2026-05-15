use ymir::errors::Outcome;

#[async_trait::async_trait]
pub(crate) trait UnitOfWork: Send + Sync {
    async fn commit(&self) -> Outcome<()>;
    async fn rollback(&self) -> Outcome<()>;
}
