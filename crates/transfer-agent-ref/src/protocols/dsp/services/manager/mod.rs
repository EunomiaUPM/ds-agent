/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

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
