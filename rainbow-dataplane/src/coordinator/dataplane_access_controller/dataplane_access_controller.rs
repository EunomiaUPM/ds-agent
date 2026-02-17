use crate::coordinator::data_source_connector::DataSourceConnectorTrait;
use crate::coordinator::dataplane_access_controller::DataPlaneAccessControllerTrait;
use crate::entities::dataplane_transfers::dataplane_transfers_entity::DataplaneTransfersEntityService;
use crate::entities::dataplane_transfers::{
    DataplaneTransfersEntitiesTrait, EditDataplaneTransferDto, NewDataplaneTransferDto,
};
use rainbow_common::adv_protocol::interplane::data_plane_provision::{
    DataPlaneProvisionRequest, DataPlaneProvisionResponse,
};
use rainbow_common::adv_protocol::interplane::data_plane_start::{
    DataPlaneStart, DataPlaneStartAck,
};
use rainbow_common::adv_protocol::interplane::data_plane_status::{
    DataPlaneStatusRequest, DataPlaneStatusResponse,
};
use rainbow_common::adv_protocol::interplane::data_plane_stop::{DataPlaneStop, DataPlaneStopAck};
use rainbow_common::adv_protocol::interplane::{
    DataPlaneControllerMessages, DataPlaneControllerVersion, DataPlaneProcessDirection,
    DataPlaneProcessState, DataPlaneSDPConfigTypes, DataPlaneSDPFieldTypes,
    DataPlaneSDPResponseField,
};
use rainbow_common::config::services::TransferConfig;
use rainbow_common::config::traits::CommonConfigTrait;
use rainbow_common::dcat_formats::FormatAction;
use std::collections::HashMap;
use std::sync::Arc;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;

pub struct DataPlaneAccessControllerService {
    data_source_connector_service: Arc<dyn DataSourceConnectorTrait>,
    dataplane_process_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    config: Arc<TransferConfig>,
}

impl DataPlaneAccessControllerService {
    pub fn new(
        data_source_connector_service: Arc<dyn DataSourceConnectorTrait>,
        dataplane_process_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
        config: Arc<TransferConfig>,
    ) -> Self {
        Self { data_source_connector_service, dataplane_process_entity, config }
    }
}

#[async_trait::async_trait]
impl DataPlaneAccessControllerTrait for DataPlaneAccessControllerService {
    async fn data_plane_provision_request(
        &self,
        input: &DataPlaneProvisionRequest,
    ) -> anyhow::Result<DataPlaneProvisionResponse> {
        let process_address = self.config.common().get_host(HostType::Http);
        let sdp_config = input.sdp_config.as_ref().unwrap();
        let next_hop_protocol = sdp_config
            .iter()
            .find(|s| s._type == DataPlaneSDPConfigTypes::NextHopAddressScheme)
            .expect("DataPlaneSDPConfigTypes::NextHopAddressScheme must be defined");
        let next_hop_address = sdp_config
            .iter()
            .find(|s| s._type == DataPlaneSDPConfigTypes::NextHopAddress)
            .expect("DataPlaneSDPConfigTypes::NextHopAddress must be defined");
        let next_hop_direction = sdp_config
            .iter()
            .find(|s| s._type == DataPlaneSDPConfigTypes::Direction)
            .expect("DataPlaneSDPConfigTypes::Direction must be defined");
        let next_hop_direction_as = next_hop_direction.content.parse::<FormatAction>()?;

        let data_plane_url = format!("{}/data/{}", process_address, input.session_id.to_string());

        let ingress_config = serde_json::json!({
            "ProcessAddressProtocol": "",
            "ProcessAddressUrl": data_plane_url,
            "ProcessAddressAuth": "",
            "ProcessAddressAuthContent": ""
        });

        let egress_config = serde_json::json!({
            "DownstreamHopAddressProtocol": next_hop_protocol.content,
            "DownstreamHopAddressUrl": next_hop_address.content,
            "DownstreamHopAddressAuth": "",
            "DownstreamHopAddressAuthContent": "",
             // Upstream fields were initialized to empty string in original code, maybe meaningful for bi-directional?
            "UpstreamHopAddressProtocol": "",
            "UpstreamHopAddressUrl": "",
            "UpstreamHopAddressAuth": "",
            "UpstreamHopAddressAuthContent": ""
        });

        use crate::data::entities::dataplane_transfers::{
            InteractionMode, TransferRole, TransferState,
        };
        use std::str::FromStr;

        // Infer InteractionMode from direction (simple mapping assumption)
        // If direction is not PULL/PUSH, we might default or error.
        // Assuming "distribute" -> PUSH? "collect" -> PULL? FormatAction has specific values.
        // For now, I'll default to InteractionMode::Pull if unknown, or map from string if possible.
        // But `InteractionMode` is an enum.

        /*
        let interaction_mode = match next_hop_direction_as {
            FormatAction::Distribute => InteractionMode::Push, // Assuming Distribute ~ Push
            FormatAction::Collect => InteractionMode::Pull, // Assuming Collect ~ Pull
            _ => InteractionMode::Pull, // Default
        };
        */
        let interaction_mode = InteractionMode::Pull;

        // TransferRole: This provisioning request usually comes to the Provider DP or Consumer DP?
        // If it's "Provision", it's creating a transfer process.
        // I'll assume Provider for now as it's common to provision source side first.
        let role = TransferRole::Provider;

        let state = TransferState::Init;

        // ID must be Uuid. input.session_id is Urn.
        // "urn:uuid:..." -> extract UUID.
        let uuid_str = input.session_id.nss();
        let transfer_uuid = uuid::Uuid::parse_str(uuid_str)?;

        // connector_instance_id: None for now as it wasn't present before.

        let dataplane_response = self
            .dataplane_process_entity
            .create_dataplane_transfer(&NewDataplaneTransferDto {
                id: transfer_uuid,
                transfer_process_id: input.session_id.to_string(),
                role,
                interaction_mode,
                state,
                connector_instance_id: None,
                ingress_config: ingress_config.clone(),
                egress_config: egress_config.clone(),
            })
            .await?;

        // Helper to extract from JSON safely
        let get_field = |val: &serde_json::Value, key: &str| -> String {
            val.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
        };

        Ok(DataPlaneProvisionResponse {
            _type: DataPlaneControllerMessages::DataPlaneProvisionResponse,
            version: DataPlaneControllerVersion::Version10,
            session_id: input.session_id.clone(),
            sdp_response: vec![
                DataPlaneSDPResponseField {
                    _type: DataPlaneSDPFieldTypes::DataPlaneAddressScheme,
                    format: "https://www.iana.org/assignments/uri-schemes/uri-schemes.xhtml"
                        .to_string(),
                    content: get_field(
                        &dataplane_response.inner.ingress_config,
                        "ProcessAddressProtocol",
                    ),
                },
                DataPlaneSDPResponseField {
                    _type: DataPlaneSDPFieldTypes::DataPlaneAddress,
                    format: "uri".to_string(),
                    content: get_field(
                        &dataplane_response.inner.ingress_config,
                        "ProcessAddressUrl",
                    ),
                },
                DataPlaneSDPResponseField {
                    _type: DataPlaneSDPFieldTypes::DataPlaneAddressAuthType,
                    format:
                        "https://www.iana.org/assignments/http-authschemes/http-authschemes.xhtml"
                            .to_string(),
                    content: get_field(
                        &dataplane_response.inner.ingress_config,
                        "ProcessAddressAuth",
                    ),
                },
                DataPlaneSDPResponseField {
                    _type: DataPlaneSDPFieldTypes::DataPlaneAddressAuthToken,
                    format: "jwt".to_string(),
                    content: get_field(
                        &dataplane_response.inner.ingress_config,
                        "ProcessAddressAuthContent",
                    ),
                },
            ],
            sdp_request: None,
            sdp_config: None,
        })
    }

    async fn data_plane_start(&self, input: &DataPlaneStart) -> anyhow::Result<DataPlaneStartAck> {
        // Parse UUID
        let uuid_str = input.session_id.nss();
        let transfer_uuid = uuid::Uuid::parse_str(uuid_str)?;

        use crate::data::entities::dataplane_transfers::TransferState;

        let _dp_process = self
            .dataplane_process_entity
            .update_state(transfer_uuid, TransferState::Started)
            .await?;
        Ok(DataPlaneStartAck {
            _type: DataPlaneControllerMessages::DataPlaneStartAck,
            version: DataPlaneControllerVersion::Version10,
            session_id: input.session_id.clone(),
        })
    }

    async fn data_plane_stop(&self, input: &DataPlaneStop) -> anyhow::Result<DataPlaneStopAck> {
        // Parse UUID
        let uuid_str = input.session_id.nss();
        let transfer_uuid = uuid::Uuid::parse_str(uuid_str)?;

        use crate::data::entities::dataplane_transfers::TransferState;

        let _dp_process = self
            .dataplane_process_entity
            .update_state(transfer_uuid, TransferState::Stopped)
            .await?;
        Ok(DataPlaneStopAck {
            _type: DataPlaneControllerMessages::DataPlaneStopAck,
            version: DataPlaneControllerVersion::Version10,
            session_id: input.session_id.clone(),
        })
    }

    async fn data_plane_get_status(
        &self,
        input: &DataPlaneStatusRequest,
    ) -> anyhow::Result<DataPlaneStatusResponse> {
        Ok(DataPlaneStatusResponse {
            _type: DataPlaneControllerMessages::DataPlaneStatusResponse,
            version: DataPlaneControllerVersion::Version10,
            session_id: input.session_id.clone(),
            sdp_response: vec![],
        })
    }
}
