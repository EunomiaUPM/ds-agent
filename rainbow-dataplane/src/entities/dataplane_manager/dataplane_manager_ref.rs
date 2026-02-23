use urn::Urn;
use rainbow_connector::ConnectorInstanceDto;
use crate::entities::dataplane_manager::driver_factory::DataplaneDriver;
use crate::entities::dataplane_transfers::{DataplaneTransferDto, InteractionMode, TransferState};

struct CommandContext<'a> {
    process_id: Urn,
    state: &'a TransferState,
    mode: &'a InteractionMode,
    driver: &'a DataplaneDriver,
    connector: Option<&'a ConnectorInstanceDto>
}

impl<'a> CommandContext<'a> {
    fn from(process: &'a DataplaneTransferDto, driver: &'a DataplaneDriver, connector: Option<&'a ConnectorInstanceDto>) -> anyhow::Result<Self> {
        Ok(Self {
            process_id: process.inner.id.parse()?,
            driver,
            connector
        })
    }
}