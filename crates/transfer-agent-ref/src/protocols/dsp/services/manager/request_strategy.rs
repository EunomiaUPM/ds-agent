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
use crate::protocols::dsp::services::manager::strategies::RequestStrategy;
use crate::protocols::dsp::services::manager::{TransferLifecycleStrategy, TransferResponse};
use ymir::errors::Outcome;

#[async_trait::async_trait]
impl TransferLifecycleStrategy for RequestStrategy {
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

// Draft test, disabled: every method of `RequestStrategy` above is still
// `todo!()`, so this cannot pass. It also mocks `DataPlaneFacadeTrait` where
// `DspManager::new` wants a `FacadeTrait`, and leaves `process`/`trigger` as
// `()` placeholders. Re-enable once the strategy is implemented.
#[cfg(any())]
mod test {
    use crate::entities::ids::{ParticipantId, TransferProcessId};
    use crate::entities::message_envelope::MessageEnvelope;
    use crate::entities::protocol::{
        ProtocolId, ProtocolState, StateMetadata, TransferCorrelation, TransferDirection,
        TransferRole,
    };
    use crate::protocols::dsp::entities::command::{
        TransferCommandDirection, TransferManagerCommand,
    };
    use crate::protocols::dsp::entities::context_common::TransferContextConnectorRole;
    use crate::protocols::dsp::facades::dataplane_facade::{
        DataPlaneFacadeTrait, MockDataPlaneFacadeTrait,
    };
    use crate::protocols::dsp::services::manager::manager::DspManager;
    use crate::protocols::dsp::services::manager::strategies::{RequestStrategy, StrategyDeps};
    use crate::services::transfer_message::{
        MockTransferMessageServiceTrait, TransferMessageServiceTrait,
    };
    use crate::services::transfer_process::views::TransferProcessView;
    use crate::services::transfer_process::{
        MockTransferProcessServiceTrait, TransferProcessServiceTrait,
    };
    use compact_str::CompactString;
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::sync::Arc;
    use url::Url;
    use urn::Urn;

    fn mock_request_strategy(
        mock_facades: Arc<dyn DataPlaneFacadeTrait>,
        mock_transfer_entities: Arc<dyn TransferProcessServiceTrait>,
        mock_messages_entities: Arc<dyn TransferMessageServiceTrait>,
    ) -> (Arc<DspManager>, RequestStrategy) {
        let dsp_manager = Arc::new(DspManager::new(
            mock_facades.clone(),
            mock_transfer_entities.clone(),
            mock_messages_entities.clone(),
        ));
        let request_strategy = RequestStrategy::new(StrategyDeps {
            facades: mock_facades.clone(),
            transfers: mock_transfer_entities.clone(),
            messages: mock_messages_entities.clone(),
            manager: dsp_manager.clone(),
        });
        (dsp_manager, request_strategy)
    }

    #[tokio::test]
    async fn test_request_strategy_being_consumer() {
        let mut mock_facades = Arc::new(MockDataPlaneFacadeTrait::new());
        mock_facades
            .expect_on_transfer_request_pre()
            .times(1)
            .returning(|| Ok(None));
        mock_facades
            .expect_on_transfer_request_post()
            .times(1)
            .returning(|| Ok(None));
        let mut ids = HashMap::new();
        ids.insert("consumerPid".to_string(), "urn:uuid:1".to_string());
        ids.insert("providerPid".to_string(), "urn:uuid:2".to_string());
        let mut mock_transfer_entities = Arc::new(MockTransferProcessServiceTrait::new());
        mock_transfer_entities
            .expect_create()
            .times(1)
            .returning(move || {
                Ok(TransferProcessView {
                    id: TransferProcessId(Urn::from_str("urn:uuid:3").unwrap()),
                    tenant_id: Default::default(),
                    role: TransferRole::Provider,
                    protocol: ProtocolId::Dsp2025_1,
                    state: ProtocolState(CompactString::new("REQUESTED")),
                    state_metadata: StateMetadata {
                        attribute: Some("ON_REQUEST".to_string()),
                        reason: None,
                        code: None,
                    },
                    correlation: TransferCorrelation {
                        identifiers: ids,
                        consumer_pid: Some("urn:uuid:1".to_string()),
                        provider_pid: Some("urn:uuid:2".to_string()),
                        agreement_id: Some(Urn::from_str("urn:uuid:4").unwrap()),
                        callback_address: Some(Url::from_str("http://test.es").unwrap()),
                        peer_participant_id: Some(ParticipantId(
                            Urn::from_str("urn:uuid:4").unwrap(),
                        )),
                    },
                    properties: Default::default(),
                    error_details: None,
                    created_at: Default::default(),
                    updated_at: Default::default(),
                    version: 0,
                })
            });
        let mut mock_messages_entities = Arc::new(MockTransferMessageServiceTrait::new());
        mock_messages_entities.expect_create().times(1);
        let (_, request_strategy) =
            mock_request_strategy(mock_facades, mock_transfer_entities, mock_messages_entities);

        TransferManagerCommand {
            process: (),
            role: TransferRole::Provider,
            direction: TransferCommandDirection::Inbound,
            trigger: (),
            transfer_direction: TransferDirection::Pull,
            connector_instance: TransferContextConnectorRole::ConsumerNotHavingConnector,
            data_address: None,
            is_restart: false,
            envelope: MessageEnvelope {
                canonical_form: None,
                canonical_hash: None,
                payload: serde_json::Value::Null,
            },
            agreement_id: None,
        }
    }
}
