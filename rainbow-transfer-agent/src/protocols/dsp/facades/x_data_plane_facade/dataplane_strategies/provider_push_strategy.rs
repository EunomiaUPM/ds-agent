use crate::protocols::dsp::facades::data_plane_facade::dataplane_strategies::DataPlaneStrategyTrait;
use crate::protocols::dsp::facades::data_plane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::protocol_types::DataAddressDto;
use rainbow_catalog_agent::DataServiceDto;
use rainbow_common::adv_protocol::interplane::data_plane_provision::DataPlaneProvisionRequest;
use rainbow_common::adv_protocol::interplane::data_plane_start::DataPlaneStart;
use rainbow_common::adv_protocol::interplane::data_plane_stop::DataPlaneStop;
use rainbow_common::adv_protocol::interplane::{
    DataPlaneControllerMessages, DataPlaneControllerVersion, DataPlaneSDPConfigField,
    DataPlaneSDPConfigTypes, DataPlaneSDPFieldTypes, DataPlaneSDPRequestField,
};
use rainbow_common::dcat_formats::{DctFormats, FormatAction};
use rainbow_dataplane::coordinator::dataplane_access_controller::DataPlaneAccessControllerTrait;
use std::sync::Arc;
use url::Url;
use urn::Urn;

pub struct ProviderPushDataplaneStrategy {
    dataplane_controller_access: Arc<dyn DataPlaneAccessControllerTrait>,
}

impl ProviderPushDataplaneStrategy {
    pub fn new(dataplane_controller_access: Arc<dyn DataPlaneAccessControllerTrait>) -> Self {
        Self { dataplane_controller_access }
    }
}

#[async_trait::async_trait]
impl DataPlaneStrategyTrait for ProviderPushDataplaneStrategy {}

#[async_trait::async_trait]
impl DataPlaneFacadeTrait for ProviderPushDataplaneStrategy {
    async fn get_dataplane_address(&self, session_id: &Urn) -> anyhow::Result<DataAddressDto> {
        todo!()
    }

    async fn on_transfer_request_pre(
        &self,
        session_id: &Urn,
        format: &DctFormats,
        data_service: &Option<DataServiceDto>,
        data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_transfer_request_post(
        &self,
        session_id: &Urn,
        format: &DctFormats,
        data_service: &Option<DataServiceDto>,
        data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<()> {
        let DataServiceDto { inner, .. } = data_service.as_ref().unwrap();
        let endpoint_url = Url::parse(inner.dcat_endpoint_url.as_str())?;
        let endpoint_scheme = endpoint_url.scheme().to_string();
        let endpoint_address = endpoint_url.to_string();

        let provision_request = self
            .dataplane_controller_access
            .data_plane_provision_request(&DataPlaneProvisionRequest {
                _type: DataPlaneControllerMessages::DataPlaneProvisionRequest,
                version: DataPlaneControllerVersion::Version10,
                session_id: session_id.clone(),
                sdp_request: vec![
                    DataPlaneSDPRequestField {
                        _type: DataPlaneSDPFieldTypes::DataPlaneAddressScheme,
                        format: "https://www.iana.org/assignments/uri-schemes/uri-schemes.xhtml".to_string(),
                    },
                    DataPlaneSDPRequestField {
                        _type: DataPlaneSDPFieldTypes::DataPlaneAddress,
                        format: "uri".to_string(),
                    },
                    DataPlaneSDPRequestField {
                        _type: DataPlaneSDPFieldTypes::DataPlaneAddressAuthType,
                        format: "https://www.iana.org/assignments/http-authschemes/http-authschemes.xhtml".to_string(),
                    },
                    DataPlaneSDPRequestField {
                        _type: DataPlaneSDPFieldTypes::DataPlaneAddressAuthToken,
                        format: "jwt".to_string(),
                    },
                ],
                sdp_config: Some(vec![
                    DataPlaneSDPConfigField {
                        _type: DataPlaneSDPConfigTypes::NextHopAddressScheme,
                        format: Some("https://www.iana.org/assignments/uri-schemes/uri-schemes.xhtml".to_string()),
                        content: endpoint_scheme,
                    },
                    DataPlaneSDPConfigField {
                        _type: DataPlaneSDPConfigTypes::NextHopAddress,
                        format: Some("uri".to_string()),
                        content: endpoint_address,
                    },
                    DataPlaneSDPConfigField {
                        _type: DataPlaneSDPConfigTypes::Direction,
                        format: Some("dcterms:transferDirection".to_string()),
                        content: FormatAction::Push.to_string(),
                    },
                ]),
            })
            .await?;
        Ok(())
    }

    async fn on_transfer_start_pre(&self, session_id: &Urn) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_transfer_start_post(&self, session_id: &Urn) -> anyhow::Result<()> {
        self.dataplane_controller_access
            .data_plane_start(&DataPlaneStart {
                _type: DataPlaneControllerMessages::DataPlaneStart,
                version: DataPlaneControllerVersion::Version10,
                session_id: session_id.clone(),
            })
            .await?;
        Ok(())
    }

    async fn on_transfer_suspension_pre(&self, session_id: &Urn) -> anyhow::Result<()> {
        self.dataplane_controller_access
            .data_plane_stop(&DataPlaneStop {
                _type: DataPlaneControllerMessages::DataPlaneStop,
                version: DataPlaneControllerVersion::Version10,
                session_id: session_id.clone(),
            })
            .await?;
        Ok(())
    }

    async fn on_transfer_suspension_post(&self, session_id: &Urn) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_transfer_completion_pre(&self, session_id: &Urn) -> anyhow::Result<()> {
        self.dataplane_controller_access
            .data_plane_stop(&DataPlaneStop {
                _type: DataPlaneControllerMessages::DataPlaneStop,
                version: DataPlaneControllerVersion::Version10,
                session_id: session_id.clone(),
            })
            .await?;
        Ok(())
    }

    async fn on_transfer_completion_post(&self, session_id: &Urn) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_transfer_termination_pre(&self, session_id: &Urn) -> anyhow::Result<()> {
        self.dataplane_controller_access
            .data_plane_stop(&DataPlaneStop {
                _type: DataPlaneControllerMessages::DataPlaneStop,
                version: DataPlaneControllerVersion::Version10,
                session_id: session_id.clone(),
            })
            .await?;
        Ok(())
    }

    async fn on_transfer_termination_post(&self, session_id: &Urn) -> anyhow::Result<()> {
        Ok(())
    }
}
