use crate::entities::dataplane_transfers::{DataplaneTransferDto, InteractionMode, TransferRole};
use rainbow_connector::ConnectorInstanceDto;
use std::sync::Arc;

pub struct DataplaneDriverFactory {}

impl DataplaneDriverFactory {
    pub fn new() -> Self {
        Self {}
    }
    pub fn create_driver(
        &self,
        process: &DataplaneTransferDto,
        connector_instance_dto: Option<&ConnectorInstanceDto>,
    ) -> anyhow::Result<DataplaneDriver> {
        let role = process.inner.role.clone();
        let interaction_mode = process.inner.interaction_mode.clone();

        let auth_driver: Arc<dyn AuthActionTrait> = match (&role, &interaction_mode) {
            (_, _) => Arc::new(NoOpAuth::new()),
        };
        let lifecycle_driver: Arc<dyn LifeCycleActionTrait> = match (&role, &interaction_mode) {
            (_, _) => Arc::new(NoOpLifecycle::new()),
        };

        Ok(DataplaneDriver { auth_driver, lifecycle_driver })
    }
}

pub struct DataplaneDriver {
    pub auth_driver: Arc<dyn AuthActionTrait>,
    pub lifecycle_driver: Arc<dyn LifeCycleActionTrait>,
}

#[async_trait::async_trait]
pub trait AuthActionTrait: Send + Sync {
    async fn perform_auth(&self, connector: Option<&ConnectorInstanceDto>) -> anyhow::Result<()>;
}

pub struct NoOpAuth {}

impl NoOpAuth {
    pub fn new() -> Self {
        Self {}
    }
}
#[async_trait::async_trait]
impl AuthActionTrait for NoOpAuth {
    async fn perform_auth(&self, _connector: Option<&ConnectorInstanceDto>) -> anyhow::Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait LifeCycleActionTrait: Send + Sync {
    async fn perform_subscribe(&self, connector: Option<&ConnectorInstanceDto>) -> anyhow::Result<()>;
    async fn perform_unsubscribe(&self, connector: Option<&ConnectorInstanceDto>) -> anyhow::Result<()>;
}

pub struct NoOpLifecycle {}

impl NoOpLifecycle {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl LifeCycleActionTrait for NoOpLifecycle {
    async fn perform_subscribe(&self, _connector: Option<&ConnectorInstanceDto>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn perform_unsubscribe(&self, _connector: Option<&ConnectorInstanceDto>) -> anyhow::Result<()> {
        Ok(())
    }
}
