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

use sea_orm::QueryTrait;
use std::sync::Arc;

use crate::data::repo::transfer_process::{TransferProcessRepoErrors, TransferProcessRepoTrait};
use crate::data::sea_orm::orm::ser_enum;
use crate::data::sea_orm::orm::transfer_process as orm;
use crate::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use crate::entities::filters::TransferProcessFilter;
use crate::entities::transfer_process::TransferProcess;
use common::paginated_spec::Cursor;
use common::query::{Page, Sort};
use common::utils::parse_urn;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

pub(crate) struct SeaOrmTransferProcessRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmTransferProcessRepo {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn fetch_err(e: sea_orm::DbErr) -> ymir::errors::Errors {
        TransferProcessRepoErrors::ErrorFetchingTransferProcess(Box::new(e)).into_errors()
    }

    #[allow(clippy::result_large_err)]
    fn decode_cursor(&self, cursor: &str) -> Outcome<chrono::DateTime<chrono::FixedOffset>> {
        Cursor::decode_timestamp(cursor)
            .map_err(|_| TransferProcessRepoErrors::InvalidCursor.into_errors())
    }

    fn apply_base_filters(
        mut q: sea_orm::Select<orm::Entity>,
        filters: &TransferProcessFilter,
    ) -> sea_orm::Select<orm::Entity> {
        if let Some(tid) = &filters.tenant_id {
            q = q.filter(orm::Column::TenantId.eq(tid.as_str()));
        }
        if let Some(protocol) = &filters.protocol {
            q = q.filter(orm::Column::Protocol.eq(ser_enum(protocol)));
        }
        if let Some(state) = &filters.state {
            let state_str = state.0.as_str();
            let upper = state_str.to_uppercase();
            q = q.filter(
                orm::Column::ProtocolState
                    .eq(state_str)
                    .or(orm::Column::ProtocolState.eq(upper.as_str())),
            );
        }
        if let Some(role) = &filters.role {
            q = q.filter(orm::Column::Role.eq(ser_enum(role)));
        }
        if let Some(agreement_id) = &filters.agreement_id {
            q = q.filter(orm::Column::AgreementId.eq(agreement_id.to_string()));
        }
        if let Some(peer) = &filters.peer_participant_id {
            q = q.filter(orm::Column::PeerParticipantId.eq(peer.to_string()));
        }
        if let Some(after) = filters.created_after {
            q = q.filter(orm::Column::CreatedAt.gt(after));
        }
        if let Some(before) = filters.created_before {
            q = q.filter(orm::Column::CreatedAt.lt(before));
        }
        q
    }
}

#[async_trait::async_trait]
impl TransferProcessRepoTrait for SeaOrmTransferProcessRepo {
    async fn get_all_transfer_processes(
        &self,
        filters: &TransferProcessFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferProcess>> {
        let mut q = Self::apply_base_filters(orm::Entity::find(), filters);

        if let Some(cursor) = &page.cursor {
            let cursor_dt = self.decode_cursor(cursor)?;
            q = match sort {
                Sort::CreatedAtAsc => q.filter(orm::Column::CreatedAt.gt(cursor_dt)),
                Sort::UpdatedAtAsc => q.filter(orm::Column::UpdatedAt.gt(cursor_dt)),
                Sort::UpdatedAtDesc => q.filter(orm::Column::UpdatedAt.lt(cursor_dt)),
                _ => q.filter(orm::Column::CreatedAt.lt(cursor_dt)),
            };
        }

        q = match sort {
            Sort::CreatedAtAsc => q
                .order_by_asc(orm::Column::CreatedAt)
                .order_by_asc(orm::Column::Id),
            Sort::UpdatedAtAsc => q
                .order_by_asc(orm::Column::UpdatedAt)
                .order_by_asc(orm::Column::Id),
            Sort::UpdatedAtDesc => q
                .order_by_desc(orm::Column::UpdatedAt)
                .order_by_desc(orm::Column::Id),
            _ => q
                .order_by_desc(orm::Column::CreatedAt)
                .order_by_desc(orm::Column::Id),
        };

        q.limit(page.limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .into_iter()
            .map(orm::Model::into_domain)
            .collect()
    }

    async fn count_transfer_processes(&self, filters: &TransferProcessFilter) -> Outcome<u64> {
        Self::apply_base_filters(orm::Entity::find(), filters)
            .count(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)
    }

    async fn get_batch_transfer_processes(
        &self,
        tenant_id: Option<String>,
        ids: &[Urn],
    ) -> Outcome<Vec<TransferProcess>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let id_strings: Vec<String> = ids.iter().map(|u| u.to_string()).collect();
        let q = orm::Entity::find()
            .filter(orm::Column::Id.is_in(id_strings))
            .apply_if(tenant_id, |q, t| q.filter(orm::Column::TenantId.eq(t)));
        q.all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .into_iter()
            .map(orm::Model::into_domain)
            .collect()
    }

    async fn get_transfer_process_by_id(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<TransferProcess>> {
        let q = orm::Entity::find_by_id(id.to_string())
            .apply_if(tenant_id, |q, t| q.filter(orm::Column::TenantId.eq(t)));
        q.one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .map(orm::Model::into_domain)
            .transpose()
    }

    async fn get_transfer_process_by_key_value(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<TransferProcess>> {
        use crate::data::sea_orm::orm::transfer_identifier as ident_orm;

        let mut q = ident_orm::Entity::find().filter(ident_orm::Column::Value.eq(id.to_string()));
        if let Some(tid) = &tenant_id {
            q = q.filter(ident_orm::Column::TenantId.eq(tid.as_str()));
        }
        let ident = q.one(self.db.as_ref()).await.map_err(Self::fetch_err)?;

        match ident {
            None => Ok(None),
            Some(i) => {
                let tid = tenant_id.as_deref().unwrap_or(&i.tenant_id);
                self.get_transfer_process_by_id(
                    Some(tid.to_string()),
                    &parse_urn(&i.transfer_process_id)?,
                )
                .await
            }
        }
    }

    async fn create_transfer_process(
        &self,
        cmd: &NewTransferProcessCommand,
    ) -> Outcome<TransferProcess> {
        orm::ActiveModel::from_cmd(cmd)?
            .insert(self.db.as_ref())
            .await
            .map_err(|e| {
                TransferProcessRepoErrors::ErrorCreatingTransferProcess(Box::new(e)).into_errors()
            })
            .and_then(orm::Model::into_domain)
    }

    async fn put_transfer_process(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
        edit_model: &EditTransferProcessCommand,
    ) -> Outcome<TransferProcess> {
        let q = orm::Entity::find_by_id(id.to_string())
            .apply_if(tenant_id, |q, t| q.filter(orm::Column::TenantId.eq(t)));
        let existing = q
            .one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .ok_or_else(|| TransferProcessRepoErrors::TransferProcessNotFound.into_errors())?;

        let mut process = existing.into_domain()?;
        process.apply_edit(edit_model.clone());

        orm::ActiveModel::from_domain(&process)
            .update(self.db.as_ref())
            .await
            .map_err(|e| {
                TransferProcessRepoErrors::ErrorUpdatingTransferProcess(Box::new(e)).into_errors()
            })
            .and_then(orm::Model::into_domain)
    }

    async fn delete_transfer_process(&self, tenant_id: Option<String>, id: &Urn) -> Outcome<()> {
        let q = orm::Entity::delete_many()
            .filter(orm::Column::Id.eq(id.to_string()))
            .apply_if(tenant_id.clone(), |q, t| {
                q.filter(orm::Column::TenantId.eq(t))
            });
        let res = q.exec(self.db.as_ref()).await.map_err(|e| {
            TransferProcessRepoErrors::ErrorDeletingTransferProcess(Box::new(e)).into_errors()
        })?;
        if res.rows_affected == 0 {
            return Err(TransferProcessRepoErrors::TransferProcessNotFound.into_errors());
        }

        use crate::data::sea_orm::orm::transfer_identifier as ident_orm;
        ident_orm::Entity::delete_many()
            .filter(ident_orm::Column::TransferProcessId.eq(id.to_string()))
            .apply_if(tenant_id, |q, t| {
                q.filter(ident_orm::Column::TenantId.eq(t))
            })
            .exec(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;

        Ok(())
    }
}
