/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::protocols::dsp::entities::context_dsp::TransferDSPContextRdf;
use common::dsp_common::data_address::DataAddress;
use ymir::errors::{BadFormat, Errors, Outcome};

/// Each DSP message is a RDF graph, so it's better to treat the deserialization
/// and serialization as such.

static CONSUMER_PID_KEY: &str = "consumerPid";
static PROVIDER_PID_KEY: &str = "providerPid";
static DATA_ADDRESS_KEY: &str = "dataAddress";

/// Pulls the strongly-typed transfer fields out of an expanded DSP message.
/// Internal to [`TransferContextTyped::from_rdf`] — one method per field so each
/// extraction rule has a single home.
pub struct DspTransferRdfExtractor<'a> {
    pub rdf: &'a TransferDSPContextRdf,
}

impl<'a> DspTransferRdfExtractor<'a> {
    pub fn new(rdf: &'a TransferDSPContextRdf) -> Self {
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
