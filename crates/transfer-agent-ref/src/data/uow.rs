use ymir::errors::Outcome;

pub trait UnitOfWorkTrait {
    async fn commit(&self) -> Outcome<()>;
    async fn rollback(&self) -> Outcome<()>;
}
