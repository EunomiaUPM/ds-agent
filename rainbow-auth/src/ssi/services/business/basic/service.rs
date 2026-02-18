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

use chrono::Utc;
use rainbow_common::auth::business::RainbowBusinessLoginRequest;
use rand::distr::Alphanumeric;
use rand::Rng;
use uuid::Uuid;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::{business_mates, mates, recv_request, recv_verification};
use ymir::utils::{create_opaque_token, get_from_opt};

use super::super::BusinessTrait;
use super::config::{BusinessConfig, BusinessConfigTrait};
use crate::ssi::types::business::BusinessResponse;

pub struct BasicBusinessService {
    config: BusinessConfig
}

impl BasicBusinessService {
    pub fn new(config: BusinessConfig) -> BasicBusinessService { BasicBusinessService { config } }
}

impl BusinessTrait for BasicBusinessService {
    fn start(
        &self,
        payload: &RainbowBusinessLoginRequest
    ) -> (recv_request::NewModel, recv_verification::Model) {
        let id = Uuid::new_v4().to_string();
        let nonce: String =
            rand::rng().sample_iter(&Alphanumeric).take(12).map(char::from).collect();
        let provider_url = format!(
            "{}{}",
            self.config.hosts().get_host(HostType::Http),
            self.config.get_api_path()
        );
        let provider_url = match self.config.is_local() {
            true => provider_url.replace("127.0.0.1", "host.docker.internal"),
            false => provider_url
        };

        let client_id = format!("{}/verify", &provider_url);
        let audience = format!("{}/{}", client_id, &payload.auth_request_id);
        let ver_model = recv_verification::Model {
            id: id.clone(),
            state: payload.auth_request_id.clone(),
            nonce,
            vc_type: "DataspaceParticipant".to_string(),
            audience,
            holder: None,
            vpt: None,
            success: None,
            status: "Pending".to_string(),
            created_at: Utc::now().naive_utc(),
            ended_at: None
        };

        let req_model = recv_request::NewModel { id, consumer_slug: "----".to_string() };

        (req_model, ver_model)
    }
    fn get_token(
        &self,
        mate: &mates::Model,
        bus_model: &business_mates::Model
    ) -> anyhow::Result<BusinessResponse> {
        let token = get_from_opt(&bus_model.token, "token")?;
        Ok(BusinessResponse { token, mate: mate.clone() })
    }
    fn end(
        &self,
        ver_model: &recv_verification::Model
    ) -> anyhow::Result<business_mates::NewModel> {
        let holder = get_from_opt(&ver_model.holder, "holder")?;
        let token = create_opaque_token();

        Ok(business_mates::NewModel {
            id: ver_model.state.clone(),
            participant_id: holder,
            token: Some(token)
        })
    }
}
