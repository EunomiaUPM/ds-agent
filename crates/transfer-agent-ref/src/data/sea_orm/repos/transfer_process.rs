use std::sync::Arc;

use base64::Engine;
use chrono::DateTime;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repo::transfer_process::{TransferProcessRepoErrors, TransferProcessRepoTrait};
use crate::data::sea_orm::mappers::{process_from_cmd, process_from_orm, process_to_active_model};
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

    fn db_err(e: sea_orm::DbErr) -> ymir::errors::Errors {
        TransferProcessRepoErrors::ErrorFetchingTransferProcess(Box::new(e)).into_errors()
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
        let mut q = orm::Entity::find();

        q = q.filter(orm::Column::TenantId.eq(filters.tenant_id.as_str()));

        if let Some(protocol) = &filters.protocol {
            use serde::Serialize;
            let s = serde_json::to_value(protocol)
                .unwrap()
                .as_str()
                .unwrap_or("")
                .to_string();
            q = q.filter(orm::Column::Protocol.eq(s));
        }
        if let Some(state) = &filters.state {
            q = q.filter(orm::Column::ProtocolState.eq(state.0.as_str()));
        }
        if let Some(role) = &filters.role {
            use serde::Serialize;
            let s = serde_json::to_value(role)
                .unwrap()
                .as_str()
                .unwrap_or("")
                .to_string();
            q = q.filter(orm::Column::Role.eq(s));
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

        if let Some(cursor) = &page.cursor {
            if let Ok(cursor_dt) = decode_cursor(cursor) {
                q = match sort {
                    Sort::CreatedAtAsc => q.filter(orm::Column::CreatedAt.gt(cursor_dt)),
                    Sort::CreatedAtDesc | Sort::UpdatedAtDesc => {
                        q.filter(orm::Column::CreatedAt.lt(cursor_dt))
                    }
                };
            }
        }

        q = match sort {
            Sort::CreatedAtAsc => q.order_by_asc(orm::Column::CreatedAt),
            Sort::CreatedAtDesc => q.order_by_desc(orm::Column::CreatedAt),
            Sort::UpdatedAtDesc => q.order_by_desc(orm::Column::UpdatedAt),
        };

        q.limit(page.limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(Self::db_err)?
            .into_iter()
            .map(process_from_orm)
            .collect()
    }

    async fn get_batch_transfer_processes(&self, ids: &Vec<Urn>) -> Outcome<Vec<TransferProcess>> {
        let id_strings: Vec<String> = ids.iter().map(|u| u.to_string()).collect();
        orm::Entity::find()
            .filter(orm::Column::Id.is_in(id_strings))
            .all(self.db.as_ref())
            .await
            .map_err(Self::db_err)?
            .into_iter()
            .map(process_from_orm)
            .collect()
    }

    async fn get_transfer_process_by_id(&self, id: &Urn) -> Outcome<Option<TransferProcess>> {
        orm::Entity::find_by_id(id.to_string())
            .one(self.db.as_ref())
            .await
            .map_err(Self::db_err)?
            .map(process_from_orm)
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
            .map_err(Self::db_err)?;

        match ident {
            None => Ok(None),
            Some(i) => self.get_transfer_process_by_id(
                &Urn::from_str_safe(&i.transfer_process_id)?,
            ).await,
        }
    }

    async fn get_transfer_process_by_key_value(&self, id: &Urn) -> Outcome<Option<TransferProcess>> {
        use crate::data::sea_orm::orm::transfer_identifier as ident_orm;

        let ident = ident_orm::Entity::find()
            .filter(ident_orm::Column::Value.eq(id.to_string()))
            .one(self.db.as_ref())
            .await
            .map_err(Self::db_err)?;

        match ident {
            None => Ok(None),
            Some(i) => self.get_transfer_process_by_id(
                &Urn::from_str_safe(&i.transfer_process_id)?,
            ).await,
        }
    }

    async fn create_transfer_process(
        &self,
        cmd: &NewTransferProcessCommand,
    ) -> Outcome<TransferProcess> {
        let process = process_from_cmd(cmd);
        process_to_active_model(&process)
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TransferProcessRepoErrors::ErrorCreatingTransferProcess(Box::new(e)).into_errors())
            .and_then(process_from_orm)
    }

    async fn put_transfer_process(
        &self,
        id: &Urn,
        edit_model: &EditTransferProcessCommand,
    ) -> Outcome<TransferProcess> {
        let existing = orm::Entity::find_by_id(id.to_string())
            .one(self.db.as_ref())
            .await
            .map_err(Self::db_err)?
            .ok_or_else(|| TransferProcessRepoErrors::TransferProcessNotFound.into_errors())?;

        let mut process = process_from_orm(existing)?;
        process.apply_edit(edit_model.clone());

        process_to_active_model(&process)
            .update(self.db.as_ref())
            .await
            .map_err(|e| TransferProcessRepoErrors::ErrorUpdatingTransferProcess(Box::new(e)).into_errors())
            .and_then(process_from_orm)
    }

    async fn delete_transfer_process(&self, id: &Urn) -> Outcome<()> {
        orm::Entity::delete_by_id(id.to_string())
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TransferProcessRepoErrors::ErrorDeletingTransferProcess(Box::new(e)).into_errors())?;
        Ok(())
    }
}

fn decode_cursor(cursor: &str) -> Result<chrono::DateTime<chrono::FixedOffset>, ()> {
    let bytes =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(cursor).map_err(|_| ())?;
    let s = String::from_utf8(bytes).map_err(|_| ())?;
    DateTime::parse_from_rfc3339(&s).map_err(|_| ())
}

trait UrnFromStr {
    fn from_str_safe(s: &str) -> Outcome<Urn>;
}

impl UrnFromStr for Urn {
    fn from_str_safe(s: &str) -> Outcome<Urn> {
        use std::str::FromStr;
        Urn::from_str(s).map_err(|e| ymir::errors::Errors::crazy("invalid URN in database", Some(Box::new(e))))
    }
}
