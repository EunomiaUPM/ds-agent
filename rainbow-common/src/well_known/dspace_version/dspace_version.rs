use crate::well_known::dspace_version::WellKnownDSpaceVersionTrait;

#[derive(Clone)]
pub struct WellKnownDSpaceVersionService {}

impl WellKnownDSpaceVersionService {
    pub fn new() -> WellKnownDSpaceVersionService { WellKnownDSpaceVersionService {} }
}

impl WellKnownDSpaceVersionTrait for WellKnownDSpaceVersionService {
    fn dspace_path(&self) -> String { "/dsp/current".to_string() }
}
