/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use rainbow_common::config::types::ClientConfig;
use ymir::types::gnap::grant_request::{Client4GR, Key4GR};

pub fn get_pretty_client_config_helper(
    client_config: &ClientConfig,
    cert: &str,
) -> anyhow::Result<Client4GR> {
    let clean_cert = cert.lines().filter(|line| !line.starts_with("-----")).collect::<String>();

    let client = Client4GR {
        key: Key4GR { proof: "httpsig".to_string(), jwk: None, cert: Some(clean_cert) },
        class_id: Some(client_config.class_id.clone()),
        display: None,
    };

    Ok(client)
}
