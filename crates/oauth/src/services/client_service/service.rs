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

use std::sync::Arc;

use chrono::Utc;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::repositories::client::ClientRepository;
use crate::entities::client::Client;
use crate::entities::commands::CreateClientCommand;
use crate::http::forms::ClientView;
use crate::services::client_service::ClientServiceTrait;
use crate::services::password;

pub(crate) struct ClientService {
    client_repo: Arc<dyn ClientRepository>,
    event_bus: Option<events::EventBus>,
}

impl ClientService {
    pub fn new(client_repo: Arc<dyn ClientRepository>) -> Self {
        Self {
            client_repo,
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl ClientServiceTrait for ClientService {
    async fn list_clients(&self) -> Outcome<Vec<ClientView>> {
        let clients = self.client_repo.get_all().await?;
        Ok(clients.into_iter().map(ClientView::assemble).collect())
    }

    async fn get_client(&self, client_id: &str) -> Outcome<ClientView> {
        self.client_repo
            .get_by_client_id(client_id)
            .await?
            .map(ClientView::assemble)
            .ok_or_else(|| Errors::format(BadFormat::Received, "client not found", None))
    }

    async fn create_client(&self, cmd: &CreateClientCommand) -> Outcome<ClientView> {
        if self
            .client_repo
            .get_by_client_id(&cmd.client_id)
            .await?
            .is_some()
        {
            return Err(Errors::format(
                BadFormat::Received,
                "client_id already in use",
                None,
            ));
        }

        let (client_secret_hash, _) = password::hash_password(&cmd.client_secret)?;
        let client = Client {
            client_id: cmd.client_id.clone(),
            client_secret_hash,
            client_name: cmd.client_name.clone(),
            role: cmd.role,
            scopes: cmd.scopes.clone(),
            created_at: Utc::now(),
        };

        let view = ClientView::assemble(self.client_repo.create(&client).await?);
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "client",
            "create",
            &view
        );
        Ok(view)
    }

    async fn delete_client(&self, client_id: &str) -> Outcome<()> {
        self.client_repo.delete(client_id).await?;
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "client",
            "delete",
            &events::EntityDeletedDto::new(client_id)
        );
        Ok(())
    }
}
