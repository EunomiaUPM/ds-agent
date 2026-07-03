use ymir::errors::Outcome;
use common::dsp_common::odrl::OdrlAgreement;
use crate::entities::protocol::TransferRole;
use crate::protocols::dsp::entities::dsp_context::TransferContextTyped;

pub mod dsp_domain_loader;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait DspDomainLoaderTrait: Send + Sync {
    async fn resolve_agreement(&self, typed: &TransferContextTyped) -> Outcome<OdrlAgreement>;
    async fn resolve_role_for_new(&self, typed: &TransferContextTyped) -> Outcome<TransferRole>;
}