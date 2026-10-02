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

//! A provider and a consumer that know each other, as after onboarding.

use tokio::net::TcpListener;

use super::auth_stub::StubAuth;
use super::participant::Participant;
use uuid::Uuid;

pub const PROVIDER_DID: &str = "did:web:provider.test";
pub const CONSUMER_DID: &str = "did:web:consumer.test";
const TO_PROVIDER: &str = "token-consumer-to-provider";
const TO_CONSUMER: &str = "token-provider-to-consumer";

pub struct Dataspace {
    pub provider: Participant,
    pub consumer: Participant,
}

impl Dataspace {
    /// Binds both ports first, so each participant can be told where the other lives.
    pub async fn start() -> Self {
        let provider_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let consumer_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let provider_url = Self::url(&provider_listener);
        let consumer_url = Self::url(&consumer_listener);
        let run = Uuid::new_v4().simple().to_string();
        let provider_tenant = format!("provider-{}", &run[..8]);
        let consumer_tenant = format!("consumer-{}", &run[..8]);

        let provider = Participant::start(
            PROVIDER_DID,
            &provider_tenant,
            provider_listener,
            StubAuth::ports(
                &provider_tenant,
                (PROVIDER_DID, &provider_url),
                (CONSUMER_DID, &consumer_url),
                TO_CONSUMER,
                TO_PROVIDER,
            ),
        )
        .await;
        let consumer = Participant::start(
            CONSUMER_DID,
            &consumer_tenant,
            consumer_listener,
            StubAuth::ports(
                &consumer_tenant,
                (CONSUMER_DID, &consumer_url),
                (PROVIDER_DID, &provider_url),
                TO_PROVIDER,
                TO_CONSUMER,
            ),
        )
        .await;
        Self { provider, consumer }
    }

    fn url(listener: &TcpListener) -> String {
        format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port())
    }
}
