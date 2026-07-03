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
        for pid in [typed.provider_pid.as_deref(), typed.consumer_pid.as_deref()]
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
        let consumer_pid = typed.consumer_pid.clone().ok_or_else(|| {
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
        if typed.data_address.is_some() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::repo::transfer_process::MockTransferProcessRepoTrait;
    use crate::protocols::dsp::entities::auth::TransferDSPAuthn;
    use crate::protocols::dsp::entities::context_common::TransferContextRaw;
    use crate::protocols::dsp::entities::context_dsp::{
        TransferDSPContextParsed, TransferDSPContextRdf,
    };
    use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
    use crate::protocols::dsp::facades::data_service_resolver_facade::DataServiceFacadeTrait;
    use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
    use crate::protocols::dsp::services::dsp_domain_loader::MockDspDomainLoaderTrait;
    use axum::extract::Request;
    use chrono::Utc;
    use common::dsp_common::odrl::OdrlAgreement;
    use common::dsp_common::well_known_types::DSPProtocolVersions;
    use common::facades::Mates;

    /// Facades that panic if used — for paths that never touch a facade
    /// (a message without an agreementId resolves to the consumer directly).
    struct StubFacades;
    #[async_trait::async_trait]
    impl FacadeTrait for StubFacades {
        async fn get_data_service_facade(&self) -> Arc<dyn DataServiceFacadeTrait> {
            unimplemented!("facade not expected in this test")
        }
        async fn get_data_plane_facade(&self) -> Arc<dyn DataPlaneFacadeTrait> {
            unimplemented!("facade not expected in this test")
        }
    }

    fn mate() -> Mates {
        let t = Utc::now().naive_utc();
        Mates {
            participant_id: "urn:example:provider".into(),
            participant_slug: "provider".into(),
            participant_type: "provider".into(),
            base_url: None,
            token: None,
            token_actions: None,
            saved_at: t,
            last_interaction: t,
            extra_fields: None,
            is_me: false,
        }
    }

    async fn typed(body: &'static str) -> TransferDSPContextTyped {
        let mut req = Request::builder()
            .method("POST")
            .uri("/transfers")
            .header("authorization", "Bearer x")
            .body(axum::body::Body::from(body))
            .unwrap();
        req.extensions_mut().insert(mate());
        let raw = TransferContextRaw::<TransferDSPAuthn>::from_request(req)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&raw.body_bytes).unwrap();
        let parsed = TransferDSPContextParsed::from_raw(
            raw,
            DSPProtocolVersions::V2025_1,
            TransferDSPMessageType::TransferStartMessage,
            json,
        )
        .unwrap();
        let rdf = TransferDSPContextRdf::from_parsed(parsed).await.unwrap();
        TransferDSPContextTyped::from_rdf(rdf).unwrap()
    }

    #[tokio::test]
    async fn process_slot_rejects_malformed_pid() {
        let loader = DspDomainLoader::new(
            Arc::new(MockTransferProcessRepoTrait::new()),
            Arc::new(StubFacades),
            Arc::new(MockDspDomainLoaderTrait::new()),
        );
        let t = typed(
            r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferStartMessage","providerPid":"not-a-urn"}"#,
        )
        .await;
        assert!(loader.process_slot(&t).await.is_err());
    }

    #[tokio::test]
    async fn load_new_process_without_agreement_is_consumer() {
        let mut repo = MockTransferProcessRepoTrait::new();
        repo.expect_get_transfer_process_by_key_value()
            .returning(|_| Ok(None)); // → New slot

        let mut resolver = MockDspDomainLoaderTrait::new();
        resolver
            .expect_resolve_agreement()
            .returning(|_| Ok(OdrlAgreement::default()));
        resolver
            .expect_resolve_role_for_new()
            .returning(|_| Ok(TransferRole::Consumer));

        let loader =
            DspDomainLoader::new(Arc::new(repo), Arc::new(StubFacades), Arc::new(resolver));
        // No agreementId → connector resolves to consumer without touching the facade.
        let t = typed(
            r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferStartMessage","consumerPid":"urn:uuid:cc"}"#,
        )
        .await;

        let domain = loader.load(t).await.unwrap();
        assert!(matches!(
            domain.process,
            TransferContextProcessSlot::New { .. }
        ));
        assert!(matches!(domain.role, TransferRole::Consumer));
        assert!(matches!(
            domain.connector_instance,
            TransferContextConnectorRole::ConsumerNotHavingConnector
        ));
        assert!(!domain.is_restart && !domain.is_idempotent_replay);
    }
}
