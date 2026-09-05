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

use std::sync::Arc;

use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::repo::transfer_process::TransferProcessRepoTrait;
use crate::entities::protocol::{TransferDirection, TransferRole};
use crate::protocols::dsp::entities::context_common::{
    TransferContextConnectorRole, TransferContextProcessSlot,
};
use crate::protocols::dsp::entities::context_dsp::{
    TransferDSPContextDomain, TransferDSPContextTyped,
};
use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::services::dsp_domain_loader::DspDomainLoaderTrait;

pub struct DspDomainLoader {
    process_repo: Arc<dyn TransferProcessRepoTrait>,
    facades: Arc<dyn FacadeTrait>,
    resolver: Arc<dyn DspDomainLoaderTrait>,
}

impl DspDomainLoader {
    pub fn new(
        process_repo: Arc<dyn TransferProcessRepoTrait>,
        facades: Arc<dyn FacadeTrait>,
        resolver: Arc<dyn DspDomainLoaderTrait>,
    ) -> Self {
        Self {
            process_repo,
            facades,
            resolver,
        }
    }

    pub async fn load(&self, typed: TransferDSPContextTyped) -> Outcome<TransferDSPContextDomain> {
        let process = self.process_slot(&typed).await?;
        let role = self.role(&process, &typed).await?;
        let is_restart = Self::is_restart(&process);
        let transfer_direction = Self::transfer_direction(&typed, is_restart);
        let agreement = self.resolver.resolve_agreement(&typed).await?;
        let connector_instance = self.connector(&typed).await?;
        let is_idempotent_replay = self.is_idempotent_replay(&typed, &process).await?;
        TransferDSPContextDomain::from_typed(
            typed,
            process,
            agreement,
            role,
            transfer_direction,
            connector_instance,
            is_restart,
            is_idempotent_replay,
        )
    }

    /// Load the existing process by its pid, or signal a brand-new one.
    async fn process_slot(
        &self,
        typed: &TransferDSPContextTyped,
    ) -> Outcome<TransferContextProcessSlot> {
        for pid in [
            typed.fields.provider_pid.as_deref(),
            typed.fields.consumer_pid.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            let urn: Urn = pid.parse().map_err(|_| {
                Errors::format(
                    BadFormat::Received,
                    format!("pid is not a URN: {pid}"),
                    None,
                )
            })?;
            if let Some(found) = self
                .process_repo
                .get_transfer_process_by_key_value(&urn)
                .await?
            {
                return Ok(TransferContextProcessSlot::Existing(found));
            }
        }
        let consumer_pid = typed.fields.consumer_pid.clone().ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "new transfer requires a consumerPid",
                None,
            )
        })?;
        Ok(TransferContextProcessSlot::New { consumer_pid })
    }

    /// An existing process carries its role; a new one's role is an external rule.
    async fn role(
        &self,
        process: &TransferContextProcessSlot,
        typed: &TransferDSPContextTyped,
    ) -> Outcome<TransferRole> {
        match process {
            TransferContextProcessSlot::Existing(p) => Ok(p.role()),
            TransferContextProcessSlot::New { .. } => {
                self.resolver.resolve_role_for_new(typed).await
            }
        }
    }

    async fn connector(
        &self,
        typed: &TransferDSPContextTyped,
    ) -> Outcome<TransferContextConnectorRole> {
        let agreement_id = typed
            .rdf
            .parsed
            .json_value
            .get("agreementId")
            .and_then(|v| v.as_str());
        let Some(agreement_id) = agreement_id else {
            return Ok(TransferContextConnectorRole::ConsumerNotHavingConnector);
        };
        let agreement_id: Urn = agreement_id
            .parse()
            .map_err(|_| Errors::format(BadFormat::Received, "agreementId is not a URN", None))?;
        let connector = self
            .facades
            .get_data_service_facade()
            .await
            .resolve_connector_by_agreement_id(&agreement_id, None)
            .await?;
        Ok(TransferContextConnectorRole::ProviderHavingConnector(
            connector,
        ))
    }

    fn transfer_direction(typed: &TransferDSPContextTyped, is_restart: bool) -> TransferDirection {
        if is_restart {
            return TransferDirection::Pull;
        }
        if typed.fields.data_address.is_some() {
            TransferDirection::Push
        } else {
            TransferDirection::Pull
        }
    }

    fn is_restart(_process: &TransferContextProcessSlot) -> bool {
        false
    }

    async fn is_idempotent_replay(
        &self,
        _typed: &TransferDSPContextTyped,
        _process: &TransferContextProcessSlot,
    ) -> Outcome<bool> {
        Ok(false)
    }
}
