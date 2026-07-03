use crate::protocols::dsp::entities::dsp_context::TransferContextRdf;
use common::dsp_common::data_address::DataAddress;
use ymir::errors::{BadFormat, Errors, Outcome};

static CONSUMER_PID_KEY: &str = "consumerPid";
static PROVIDER_PID_KEY: &str = "providerPid";
static DATA_ADDRESS_KEY: &str = "dataAddress";

/// Pulls the strongly-typed transfer fields out of an expanded DSP message.
/// Internal to [`TransferContextTyped::from_rdf`] — one method per field so each
/// extraction rule has a single home.
pub struct DspTransferRdfExtractor<'a> {
    pub rdf: &'a TransferContextRdf,
}

impl<'a> DspTransferRdfExtractor<'a> {
    pub fn new(rdf: &'a TransferContextRdf) -> Self {
        Self { rdf }
    }

    fn body(&self) -> &serde_json::Value {
        &self.rdf.parsed.json_value
    }

    pub fn consumer_pid(&self) -> Option<String> {
        self.string_field(CONSUMER_PID_KEY)
    }

    pub fn provider_pid(&self) -> Option<String> {
        self.string_field(PROVIDER_PID_KEY)
    }

    pub fn data_address(&self) -> Outcome<Option<DataAddress>> {
        match self.body().get(DATA_ADDRESS_KEY) {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(v) => serde_json::from_value(v.clone()).map(Some).map_err(|e| {
                Errors::format(
                    BadFormat::Received,
                    format!("invalid dataAddress: {e}"),
                    None,
                )
            }),
        }
    }

    fn string_field(&self, key: &str) -> Option<String> {
        self.body()
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    }
}
