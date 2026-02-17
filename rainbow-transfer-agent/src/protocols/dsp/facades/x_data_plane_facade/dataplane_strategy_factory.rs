use crate::protocols::dsp::facades::data_plane_facade::dataplane_strategies::consumer_pull_strategy::ConsumerPullDataplaneStrategy;
use crate::protocols::dsp::facades::data_plane_facade::dataplane_strategies::consumer_push_strategy::ConsumerPushDataplaneStrategy;
use crate::protocols::dsp::facades::data_plane_facade::dataplane_strategies::provider_pull_strategy::ProviderPullDataplaneStrategy;
use crate::protocols::dsp::facades::data_plane_facade::dataplane_strategies::provider_push_strategy::ProviderPushDataplaneStrategy;
use crate::protocols::dsp::facades::data_plane_facade::DataPlaneFacadeTrait;
use anyhow::{anyhow, bail};
use rainbow_common::config::types::roles::RoleConfig;
use rainbow_common::dcat_formats::{DctFormats, FormatAction};
use rainbow_dataplane::coordinator::dataplane_access_controller::DataPlaneAccessControllerTrait;
use std::sync::Arc;

pub struct DataPlaneStrategyFactory {
    dataplane_access_controller: Arc<dyn DataPlaneAccessControllerTrait>,
}

impl DataPlaneStrategyFactory {
    pub fn new(dataplane_access_controller: Arc<dyn DataPlaneAccessControllerTrait>) -> Self {
        Self { dataplane_access_controller }
    }
    pub fn get_strategy(
        &self,
        role: &RoleConfig,
        format: &DctFormats,
    ) -> Box<dyn DataPlaneFacadeTrait> {
        match (role, format.action) {
            (RoleConfig::Provider, FormatAction::Pull) => Box::new(
                ProviderPullDataplaneStrategy::new(self.dataplane_access_controller.clone()),
            ),
            (RoleConfig::Provider, FormatAction::Push) => Box::new(
                ProviderPushDataplaneStrategy::new(self.dataplane_access_controller.clone()),
            ),
            (RoleConfig::Consumer, FormatAction::Pull) => Box::new(
                ConsumerPullDataplaneStrategy::new(self.dataplane_access_controller.clone()),
            ),
            (RoleConfig::Consumer, FormatAction::Push) => Box::new(
                ConsumerPushDataplaneStrategy::new(self.dataplane_access_controller.clone()),
            ),
            _ => Box::new(ConsumerPushDataplaneStrategy::new(
                self.dataplane_access_controller.clone(),
            )),
        }
    }
}
