use ymir::errors::Outcome;

use crate::data::unit_of_work::UnitOfWork;

pub(crate) struct InMemoryUow;

#[async_trait::async_trait]
impl UnitOfWork for InMemoryUow {
    async fn commit(&self) -> Outcome<()> {
        Ok(())
    }

    async fn rollback(&self) -> Outcome<()> {
        Ok(())
    }
}
