use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::protocol_types::DataAddressDto;
use rainbow_common::config::types::roles::RoleConfig;
use rainbow_connector::ConnectorInstanceDto;
use rainbow_dataplane::{
    DataplaneAddress, DataplaneCommand, DataplaneManager, DataplaneManagerInput,
};
use std::sync::Arc;
use urn::Urn;

pub struct DataPlaneFacade {
    dataplane_manager: Arc<DataplaneManager>,
    proxy_base_url: String,
}

impl DataPlaneFacade {
    pub fn new(
        dataplane_manager: Arc<DataplaneManager>,
        proxy_base_url: String,
    ) -> DataPlaneFacade {
        DataPlaneFacade { dataplane_manager, proxy_base_url }
    }

    /// Reads the `IngressConfig::HttpListener` path from the DB and prepends `proxy_base_url`
    /// to produce a full DataAddressDto. Returns `None` for Connector-based ingress.
    async fn ingress_as_data_address(
        &self,
        transfer_id: &Urn,
    ) -> anyhow::Result<Option<DataAddressDto>> {
        if let Some(addr) = self.dataplane_manager.get_ingress_address(transfer_id).await? {
            return Ok(Some(DataAddressDto {
                endpoint_type: addr.endpoint_type,
                endpoint: Some(format!("{}{}", self.proxy_base_url, addr.endpoint)),
                endpoint_properties: None,
            }));
        }
        Ok(None)
    }

    /// Helper: fire a simple command (no payload) on the DataplaneManager.
    async fn fire_command(
        &self,
        transfer_id: &Urn,
        command: DataplaneCommand,
    ) -> anyhow::Result<()> {
        self.dataplane_manager
            .execute_command(&DataplaneManagerInput {
                transfer_process_id: transfer_id.clone(),
                command,
            })
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl DataPlaneFacadeTrait for DataPlaneFacade {
    // ─── TransferRequest ───

    async fn on_transfer_request_pre(
        &self,
        transfer_id: &Urn,
        data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<Option<DataAddressDto>> {
        // Convert the optional DataAddressDto into a DataplaneAddress.
        // Presence of data_address signals PUSH mode to the DataplaneManager so it can set
        // interaction_mode = Push when creating the consumer's dataplane process.
        let init_data_address = data_address.as_ref().map(|da| DataplaneAddress {
            endpoint_type: da.endpoint_type.clone(),
            endpoint: da.endpoint.clone().unwrap_or_default(),
            authorization_type: None,
            authorization: None,
        });
        // Consumer side: register dataplane process
        self.fire_command(
            transfer_id,
            DataplaneCommand::SetInit {
                role: RoleConfig::Consumer,
                connector_instance: None,
                data_address: init_data_address,
            },
        )
        .await?;
        // PUSH mode: set consumer egress to the data client's original destination endpoint,
        // then return the auto-generated ingest URL to replace the outgoing DataAddress field.
        if let Some(da) = data_address {
            if let Some(endpoint) = &da.endpoint {
                self.fire_command(
                    transfer_id,
                    DataplaneCommand::SetEgress {
                        data_address: DataplaneAddress {
                            endpoint_type: da.endpoint_type.clone(),
                            endpoint: endpoint.clone(),
                            authorization_type: None,
                            authorization: None,
                        },
                    },
                )
                .await?;
            }
            return self.ingress_as_data_address(transfer_id).await;
        }
        Ok(None)
    }

    async fn on_transfer_request_post(
        &self,
        transfer_id: &Urn,
        connector_instance: &ConnectorInstanceDto,
        data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<()> {
        // Provider side: register dataplane process with connector
        self.fire_command(
            transfer_id,
            DataplaneCommand::SetInit {
                role: RoleConfig::Provider,
                connector_instance: Some(connector_instance.id.clone()),
                data_address: None,
            },
        )
        .await?;
        // PUSH Provider: set egress to consumer's ingest URL from TransferRequest.data_address.
        // The autonomous chain (Init→Ready) runs inside fire_command(SetInit), so the process
        // exists and is Ready before we fire SetEgress.
        if let Some(da) = data_address {
            if let Some(endpoint) = &da.endpoint {
                self.fire_command(
                    transfer_id,
                    DataplaneCommand::SetEgress {
                        data_address: DataplaneAddress {
                            endpoint_type: da.endpoint_type.clone(),
                            endpoint: endpoint.clone(),
                            authorization_type: None,
                            authorization: None,
                        },
                    },
                )
                .await?;
            }
        }
        Ok(())
    }

    // ─── TransferStart ───

    async fn on_transfer_start_pre(
        &self,
        transfer_id: &Urn,
    ) -> anyhow::Result<Option<DataAddressDto>> {
        self.fire_command(transfer_id, DataplaneCommand::SetStarted).await?;
        // PULL Provider: return the proxy listener URL to include in the TransferStart message.
        // PULL Consumer / PUSH: ingress is an HttpListener too, but consumer never sends a
        // TransferStart, so the value is unused — returning it is harmless.
        self.ingress_as_data_address(transfer_id).await
    }

    async fn on_transfer_start_post(
        &self,
        transfer_id: &Urn,
        data_address: Option<DataAddressDto>,
    ) -> anyhow::Result<Option<DataAddressDto>> {
        // PULL Consumer: apply the provider's proxy URL as egress before starting.
        // PULL Consumer: apply the provider's proxy URL as egress before starting.
        // CHECK if we are in PULL mode before applying. In PUSH mode, Egress is already set correctly
        // to the client destination in on_transfer_request_pre and must not be overwritten.
        if self.dataplane_manager.is_pull(transfer_id).await? {
            if let Some(ref da) = data_address {
                if let Some(endpoint) = &da.endpoint {
                    self.fire_command(
                        transfer_id,
                        DataplaneCommand::SetEgress {
                            data_address: DataplaneAddress {
                                endpoint_type: da.endpoint_type.clone(),
                                endpoint: endpoint.clone(),
                                authorization_type: None,
                                authorization: None,
                            },
                        },
                    )
                    .await?;
                }
            }
        }

        self.fire_command(transfer_id, DataplaneCommand::SetStarted).await?;
        // Return consumer's own ingress URL so the data client knows where to fetch data.
        self.ingress_as_data_address(transfer_id).await
    }

    // ─── TransferSuspension → SetStopped ───

    async fn on_transfer_suspension_pre(&self, transfer_id: &Urn) -> anyhow::Result<()> {
        self.fire_command(transfer_id, DataplaneCommand::SetStopped).await
    }

    async fn on_transfer_suspension_post(&self, transfer_id: &Urn) -> anyhow::Result<()> {
        self.fire_command(transfer_id, DataplaneCommand::SetStopped).await
    }

    // ─── TransferCompletion → SetTerminated ───

    async fn on_transfer_completion_pre(&self, transfer_id: &Urn) -> anyhow::Result<()> {
        self.fire_command(transfer_id, DataplaneCommand::SetTerminated).await
    }

    async fn on_transfer_completion_post(&self, transfer_id: &Urn) -> anyhow::Result<()> {
        self.fire_command(transfer_id, DataplaneCommand::SetTerminated).await
    }

    // ─── TransferTermination → SetTerminated ───

    async fn on_transfer_termination_pre(&self, transfer_id: &Urn) -> anyhow::Result<()> {
        self.fire_command(transfer_id, DataplaneCommand::SetTerminated).await
    }

    async fn on_transfer_termination_post(&self, transfer_id: &Urn) -> anyhow::Result<()> {
        self.fire_command(transfer_id, DataplaneCommand::SetTerminated).await
    }

    // ─── Config updates ───

    async fn set_egress(
        &self,
        transfer_id: &Urn,
        data_address: DataplaneAddress,
    ) -> anyhow::Result<()> {
        self.fire_command(transfer_id, DataplaneCommand::SetEgress { data_address }).await
    }
}
