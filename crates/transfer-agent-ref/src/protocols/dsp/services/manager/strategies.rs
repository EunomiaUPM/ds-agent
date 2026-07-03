use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::services::manager::manager::DspManager;
use crate::services::transfer_message::TransferMessageServiceTrait;
use crate::services::transfer_process::TransferProcessServiceTrait;
use std::sync::Arc;

/// Every strategy needs, cloned in from the manager at selection time.

#[derive(Clone)]
pub(super) struct StrategyDeps {
    pub facades: Arc<dyn FacadeTrait>,
    pub transfers: Arc<dyn TransferProcessServiceTrait>,
    pub messages: Arc<dyn TransferMessageServiceTrait>,
    pub manager: Arc<DspManager>,
}

macro_rules! strategy {
    ($name:ident) => {
        pub(super) struct $name {
            #[allow(dead_code)] // used once the phase bodies land
            deps: StrategyDeps,
        }
        impl $name {
            pub(super) fn new(deps: StrategyDeps) -> Self {
                Self { deps }
            }
        }
    };
}

strategy!(RequestStrategy);
strategy!(StartStrategy);
strategy!(SuspendStrategy);
strategy!(CompleteStrategy);
strategy!(TerminateStrategy);
