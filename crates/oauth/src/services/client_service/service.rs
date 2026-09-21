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
use common::auth::AccessScope;
use common::errors::NotFoundExt;
use common::paginated_spec::Cursor;
use common::query::QueryFilter;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::repositories::client::ClientRepository;
use crate::entities::client::Client;
use crate::entities::commands::CreateClientCommand;
use crate::entities::query::{ClientFilter, Page, Paginated, Sort};
use crate::services::client_service::ClientServiceTrait;
use crate::services::client_service::views::ClientView;
use crate::services::password;

pub struct ClientService {
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
    async fn list_clients(
        &self,
        scope: &AccessScope,
        filter: &ClientFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<ClientView>> {
        scope.require_read()?;
        filter.validate()?;
        let mut filter = filter.clone();
        filter.tenant_id = scope.resolve_query_tenant(filter.tenant_id.as_deref())?;
        let page = page.clamped();
        let (clients, total) = tokio::try_join!(
            self.client_repo.get_all(&filter, &page, sort),
            self.client_repo.count(&filter),
        )?;
        let views: Vec<ClientView> = clients.into_iter().map(ClientView::assemble).collect();
        Ok(Paginated::from_page(views, &page, Some(total), |c| {
            Cursor::encode_timestamp(&c.created_at)
        }))
    }

    async fn get_client(&self, scope: &AccessScope, client_id: &str) -> Outcome<ClientView> {
        scope.require_read()?;
        let client = self
            .client_repo
            .get_by_id(scope.acting_tenant(), client_id)
            .await?
            .or_not_found(client_id, "client")?;
        Ok(ClientView::assemble(client))
    }

    async fn create_client(
        &self,
        scope: &AccessScope,
        cmd: &CreateClientCommand,
    ) -> Outcome<ClientView> {
        let mut cmd = cmd.clone();
        let target_tenant = scope.resolve_create_tenant(cmd.tenant_id.as_deref())?;
        cmd.tenant_id = Some(target_tenant.clone());

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
            tenant_id: target_tenant,
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

    async fn delete_client(&self, scope: &AccessScope, client_id: &str) -> Outcome<()> {
        scope.require_write()?;
        self.client_repo
            .delete(scope.acting_tenant(), client_id)
            .await?;
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
