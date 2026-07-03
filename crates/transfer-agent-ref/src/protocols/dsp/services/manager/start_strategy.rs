use crate::protocols::dsp::entities::command::TransferManagerCommand;
use crate::protocols::dsp::services::manager::strategies::StartStrategy;
use crate::protocols::dsp::services::manager::{TransferLifecycleStrategy, TransferResponse};
use ymir::errors::Outcome;

#[async_trait::async_trait]
impl TransferLifecycleStrategy for StartStrategy {
    async fn validations(&self, _cmd: &mut TransferManagerCommand) -> Outcome<()> {
        todo!()
    }

    async fn pre_hook(&self, _cmd: &mut TransferManagerCommand) -> Outcome<()> {
        todo!()
    }

    async fn persist(&self, cmd: &mut TransferManagerCommand) -> Outcome<()> {
        todo!()
    }

    async fn send_to_peer(&self, _cmd: &TransferManagerCommand) -> Outcome<()> {
        todo!()
    }

    async fn fire_events(&self, _cmd: &TransferManagerCommand) -> Outcome<()> {
        todo!()
    }

    async fn post_hook(&self, _cmd: &mut TransferManagerCommand) -> Outcome<()> {
        todo!()
    }

    async fn build_response(&self, cmd: &TransferManagerCommand) -> Outcome<TransferResponse> {
        todo!()
    }
}
