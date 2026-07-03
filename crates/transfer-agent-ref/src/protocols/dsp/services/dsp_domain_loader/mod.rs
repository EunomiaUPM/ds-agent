use crate::entities::protocol::TransferRole;
use crate::protocols::dsp::entities::context_dsp::TransferDSPContextTyped;
use common::dsp_common::odrl::OdrlAgreement;
use ymir::errors::Outcome;

pub mod dsp_domain_loader;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait DspDomainLoaderTrait: Send + Sync {
    async fn resolve_agreement(&self, typed: &TransferDSPContextTyped) -> Outcome<OdrlAgreement>;
    async fn resolve_role_for_new(&self, typed: &TransferDSPContextTyped) -> Outcome<TransferRole>;
}
