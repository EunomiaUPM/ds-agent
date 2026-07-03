mod complete_strategy;
mod manager;
mod request_strategy;
mod start_strategy;
mod strategies;
mod suspend_strategy;
mod terminate_strategy;

use ymir::errors::Outcome;

use crate::protocols::dsp::entities::command::TransferManagerCommand;
use crate::protocols::dsp::facades::FacadeTrait;
use crate::services::transfer_message::TransferMessageServiceTrait;
use crate::services::transfer_process::TransferProcessServiceTrait;

/// What the manager hands back: the DSP `ack` or `error` for the caller to
/// return on the wire (inbound) or forward to the peer (outbound).
pub enum TransferResponse {
    Ack(TransferManagerCommand),
    Error(TransferManagerCommand),
}

/// One implementation per DSP message type. The manager owns the ordered
/// template; a strategy only overrides the phases it cares about (defaults noop).
#[async_trait::async_trait]
pub trait TransferLifecycleStrategy: Send + Sync {
    async fn validations(&self, _cmd: &mut TransferManagerCommand) -> Outcome<()> {
        Ok(())
    }
    async fn pre_hook(&self, _cmd: &mut TransferManagerCommand) -> Outcome<()> {
        Ok(())
    }
    async fn persist(&self, cmd: &mut TransferManagerCommand) -> Outcome<()>;
    async fn send_to_peer(&self, _cmd: &TransferManagerCommand) -> Outcome<()> {
        Ok(())
    }
    async fn fire_events(&self, _cmd: &TransferManagerCommand) -> Outcome<()> {
        Ok(())
    }
    async fn post_hook(&self, _cmd: &mut TransferManagerCommand) -> Outcome<()> {
        Ok(())
    }
    async fn build_response(&self, cmd: &TransferManagerCommand) -> Outcome<TransferResponse>;
}
