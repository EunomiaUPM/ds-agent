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

use ymir::errors::Outcome;
use ymir::types::gnap::grant_request::{Client4GR, Key4GR, KeyProof};

use crate::config::types::{DisplayInfo, EntityClientConfig};

pub trait EntityClientTrait {
    fn client_config(&self) -> &EntityClientConfig;
    fn get_clas_id(&self) -> &str {
        &self.client_config().class_id
    }
    fn get_display_info(&self) -> Option<&DisplayInfo> {
        self.client_config().display.as_ref()
    }
    fn get_pretty_client_config(&self, cert: &str) -> Outcome<Client4GR> {
        let clean_cert = cert
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect::<String>();

        Ok(Client4GR {
            key: Key4GR::new(KeyProof::HttpSig, None, Some(clean_cert))?,
            class_id: Some(self.get_clas_id().to_string()),
            display: None,
        })
    }
}
