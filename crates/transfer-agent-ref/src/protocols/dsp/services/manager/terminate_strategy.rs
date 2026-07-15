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

use crate::protocols::dsp::entities::command::TransferManagerCommand;
use crate::protocols::dsp::services::manager::strategies::TerminateStrategy;
use crate::protocols::dsp::services::manager::{TransferLifecycleStrategy, TransferResponse};
use ymir::errors::Outcome;

#[async_trait::async_trait]
impl TransferLifecycleStrategy for TerminateStrategy {
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
