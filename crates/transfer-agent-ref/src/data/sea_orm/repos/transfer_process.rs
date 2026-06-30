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

use base64::Engine;
use chrono::DateTime;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repo::transfer_process::{TransferProcessRepoErrors, TransferProcessRepoTrait};
use crate::data::sea_orm::orm::ser_enum;
use crate::data::sea_orm::orm::transfer_process as orm;
use crate::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use crate::entities::query::{Page, Sort, TransferProcessFilter};
use crate::entities::transfer_process::TransferProcess;

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
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(cursor)
            .map_err(|_| TransferProcessRepoErrors::InvalidCursor.into_errors())?;
        let s = String::from_utf8(bytes)
            .map_err(|_| TransferProcessRepoErrors::InvalidCursor.into_errors())?;
        DateTime::parse_from_rfc3339(&s)
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
            q = q.filter(orm::Column::ProtocolState.eq(state.0.as_str()));
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
                Sort::CreatedAtDesc => q.filter(orm::Column::CreatedAt.lt(cursor_dt)),
                Sort::UpdatedAtDesc => q.filter(orm::Column::UpdatedAt.lt(cursor_dt)),
            };
        }

        q = match sort {
            Sort::CreatedAtAsc => q
                .order_by_asc(orm::Column::CreatedAt)
                .order_by_asc(orm::Column::Id),
            Sort::CreatedAtDesc => q
                .order_by_desc(orm::Column::CreatedAt)
                .order_by_desc(orm::Column::Id),
            Sort::UpdatedAtDesc => q
                .order_by_desc(orm::Column::UpdatedAt)
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

    async fn get_batch_transfer_processes(&self, ids: &[Urn]) -> Outcome<Vec<TransferProcess>> {
        let id_strings: Vec<String> = ids.iter().map(|u| u.to_string()).collect();
        orm::Entity::find()
            .filter(orm::Column::Id.is_in(id_strings))
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .into_iter()
            .map(orm::Model::into_domain)
            .collect()
    }

    async fn get_transfer_process_by_id(&self, id: &Urn) -> Outcome<Option<TransferProcess>> {
        orm::Entity::find_by_id(id.to_string())
            .one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .map(orm::Model::into_domain)
            .transpose()
    }

    async fn get_transfer_process_by_key_id(
        &self,
        key_id: &str,
        id: &Urn,
    ) -> Outcome<Option<TransferProcess>> {
        use crate::data::sea_orm::orm::transfer_identifier as ident_orm;

        let ident = ident_orm::Entity::find()
            .filter(ident_orm::Column::Key.eq(key_id))
            .filter(ident_orm::Column::Value.eq(id.to_string()))
            .one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;

        match ident {
            None => Ok(None),
            Some(i) => {
                self.get_transfer_process_by_id(&parse_urn(&i.transfer_process_id)?)
                    .await
            }
        }
    }

    async fn get_transfer_process_by_key_value(
        &self,
        id: &Urn,
    ) -> Outcome<Option<TransferProcess>> {
        use crate::data::sea_orm::orm::transfer_identifier as ident_orm;

        let ident = ident_orm::Entity::find()
            .filter(ident_orm::Column::Value.eq(id.to_string()))
            .one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;

        match ident {
            None => Ok(None),
            Some(i) => {
                self.get_transfer_process_by_id(&parse_urn(&i.transfer_process_id)?)
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
        id: &Urn,
        edit_model: &EditTransferProcessCommand,
    ) -> Outcome<TransferProcess> {
        let existing = orm::Entity::find_by_id(id.to_string())
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

    async fn delete_transfer_process(&self, id: &Urn) -> Outcome<()> {
        orm::Entity::delete_by_id(id.to_string())
            .exec(self.db.as_ref())
            .await
            .map_err(|e| {
                TransferProcessRepoErrors::ErrorDeletingTransferProcess(Box::new(e)).into_errors()
            })?;
        Ok(())
    }
}

#[allow(clippy::result_large_err)]
fn parse_urn(s: &str) -> Outcome<Urn> {
    use std::str::FromStr;
    Urn::from_str(s)
        .map_err(|e| ymir::errors::Errors::crazy("invalid URN in database", Some(Box::new(e))))
}
