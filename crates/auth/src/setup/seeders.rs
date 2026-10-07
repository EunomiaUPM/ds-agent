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

//! Boot seeders of the auth module, acting on its services in-process.

use std::sync::Arc;

use crate::core::AuthCore;
use crate::modules::ParticipantModule;
use common::boot::seeders::{BootPhase, BootSeeder};
use ymir::errors::{Errors, Outcome};
use ymir::modules::WalletModuleTrait;
use ymir::types::oauth::UserInfo;

/// Links the agent's own wallet as a participant when the auth plane does not know it yet.
#[derive(Clone)]
pub struct SelfParticipantOnboarder {
    core: Arc<AuthCore>,
}

impl SelfParticipantOnboarder {
    /// Acts on `core` in-process, as `UserInfo::system()`.
    pub fn new(core: Arc<AuthCore>) -> Self {
        Self { core }
    }
}

#[async_trait::async_trait]
impl BootSeeder for SelfParticipantOnboarder {
    fn name(&self) -> &'static str {
        "self-participant"
    }

    /// In-process, so the participant exists before the first request is served.
    fn phase(&self) -> BootPhase {
        BootPhase::BeforeServe
    }

    async fn seed(&self) -> Outcome<()> {
        let user = UserInfo::system();
        let participant = match self.core.get_myself(&user).await {
            Ok(participant) => participant,
            Err(Errors::MissingResourceError { .. }) => {
                self.core.link().await?;
                self.core.get_myself(&user).await?
            }
            Err(e) => return Err(e),
        };
        tracing::info!(
            participant = participant.participant_id,
            "Self participant ready"
        );
        Ok(())
    }
}
