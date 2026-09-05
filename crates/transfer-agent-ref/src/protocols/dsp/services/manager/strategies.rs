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

use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::services::manager::manager::DspManager;
use crate::services::transfer_message::TransferMessageServiceTrait;
use crate::services::transfer_process::TransferProcessServiceTrait;
use std::sync::Arc;

/// What every strategy needs, cloned in from the manager at selection time.
#[derive(Clone)]
pub(super) struct StrategyDeps {
    pub facades: Arc<dyn FacadeTrait>,
    pub transfers: Arc<dyn TransferProcessServiceTrait>,
    pub messages: Arc<dyn TransferMessageServiceTrait>,
    pub manager: Arc<DspManager>,
}

macro_rules! strategy {
    ($name:ident) => {
        pub(super) struct $name {
            #[allow(dead_code)] // used once the phase bodies land
            deps: StrategyDeps,
        }
        impl $name {
            pub(super) fn new(deps: StrategyDeps) -> Self {
                Self { deps }
            }
        }
    };
}

strategy!(RequestStrategy);
strategy!(StartStrategy);
strategy!(SuspendStrategy);
strategy!(CompleteStrategy);
strategy!(TerminateStrategy);
