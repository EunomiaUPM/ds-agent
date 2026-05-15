use std::str::FromStr;
use std::sync::Arc;

use chrono::Utc;
use compact_str::CompactString;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repo::transfer_message::{TransferMessageRepoErrors, TransferMessageRepoTrait};
use crate::data::sea_orm::mappers::{message_from_orm, message_to_active_model};
use crate::data::sea_orm::orm::transfer_message as orm;
use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::ids::{MessageId, ParticipantId, RequestId};
use crate::entities::protocol::{ProtocolState, ProtocolMessageType};
use crate::entities::query::{Page, Sort, TransferMessageFilter};
use crate::entities::transfer_message::{MessageProcessingResult, TransferMessage};

pub(crate) struct SeaOrmTransferMessageRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmTransferMessageRepo {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn db_err(e: sea_orm::DbErr) -> ymir::errors::Errors {
        TransferMessageRepoErrors::ErrorFetchingTransferMessage(Box::new(e)).into_errors()
    }
}

#[async_trait::async_trait]
impl TransferMessageRepoTrait for SeaOrmTransferMessageRepo {
    async fn get_all_transfer_messages(
        &self,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferMessage>> {
        let mut q = orm::Entity::find();
        q = q.filter(orm::Column::TenantId.eq(filters.tenant_id.as_str()));
        q = apply_message_filters(q, filters, page, sort);
        q.limit(page.limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(Self::db_err)?
            .into_iter()
            .map(message_from_orm)
            .collect()
    }

    async fn get_messages_by_process_id(
        &self,
        process_id: &Urn,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferMessage>> {
        let mut q = orm::Entity::find();
        q = q.filter(orm::Column::TransferProcessId.eq(process_id.to_string()));
        q = apply_message_filters(q, filters, page, sort);
        q.limit(page.limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(Self::db_err)?
            .into_iter()
            .map(message_from_orm)
            .collect()
    }

    async fn get_transfer_message_by_id(&self, id: &Urn) -> Outcome<Option<TransferMessage>> {
        orm::Entity::find_by_id(id.to_string())
            .one(self.db.as_ref())
            .await
            .map_err(Self::db_err)?
            .map(message_from_orm)
            .transpose()
    }

    async fn create_transfer_message(
        &self,
        cmd: &NewTransferMessageCommand,
    ) -> Outcome<TransferMessage> {
        let msg = message_from_cmd(cmd);
        message_to_active_model(&msg)
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TransferMessageRepoErrors::ErrorCreatingTransferMessage(Box::new(e)).into_errors())
            .and_then(message_from_orm)
    }

    async fn delete_transfer_message(&self, id: &Urn) -> Outcome<()> {
        orm::Entity::delete_by_id(id.to_string())
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TransferMessageRepoErrors::ErrorDeletingTransferMessage(Box::new(e)).into_errors())?;
        Ok(())
    }
}

fn apply_message_filters(
    mut q: sea_orm::Select<orm::Entity>,
    filters: &TransferMessageFilter,
    page: &Page,
    sort: &Sort,
) -> sea_orm::Select<orm::Entity> {
    use serde::Serialize;

    if let Some(dir) = &filters.direction {
        let s = serde_json::to_value(dir).unwrap().as_str().unwrap_or("").to_string();
        q = q.filter(orm::Column::Direction.eq(s));
    }
    if let Some(protocol) = &filters.protocol {
        let s = serde_json::to_value(protocol).unwrap().as_str().unwrap_or("").to_string();
        q = q.filter(orm::Column::Protocol.eq(s));
    }
    if let Some(state) = &filters.state_transition_to {
        q = q.filter(orm::Column::StateTransitionTo.eq(state.0.as_str()));
    }
    if let Some(after) = filters.created_after {
        q = q.filter(orm::Column::OccurredAt.gt(after));
    }
    if let Some(before) = filters.created_before {
        q = q.filter(orm::Column::OccurredAt.lt(before));
    }

    if let Some(cursor) = &page.cursor {
        if let Ok(bytes) = base64::Engine::decode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            cursor,
        ) {
            if let Ok(s) = String::from_utf8(bytes) {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
                    q = match sort {
                        Sort::CreatedAtAsc => q.filter(orm::Column::OccurredAt.gt(dt)),
                        _ => q.filter(orm::Column::OccurredAt.lt(dt)),
                    };
                }
            }
        }
    }

    match sort {
        Sort::CreatedAtAsc => q.order_by_asc(orm::Column::OccurredAt),
        _ => q.order_by_desc(orm::Column::OccurredAt),
    }
}

fn message_from_cmd(cmd: &NewTransferMessageCommand) -> TransferMessage {
    let id = cmd.id.clone().unwrap_or_else(MessageId::generate);
    let peer_participant_id = cmd
        .peer_participant_id
        .clone()
        .unwrap_or_else(|| ParticipantId::new(
            Urn::from_str("urn:unknown:participant").expect("static URN"),
        ));
    let request_id = cmd.request_id.clone().unwrap_or_else(RequestId::generate);
    let protocol_version =
        CompactString::from(cmd.protocol_version.as_deref().unwrap_or("1.0"));
    let processing_result = MessageProcessingResult::Accepted {
        resulting_state: cmd.state_transition_to.clone(),
    };

    TransferMessage {
        id,
        transfer_process_id: cmd.transfer_process_id.clone(),
        tenant_id: cmd.tenant_id.clone(),
        direction: cmd.direction,
        protocol: cmd.protocol.clone(),
        message_type: cmd.message_type.clone(),
        protocol_version,
        envelope: cmd.envelope.clone(),
        occurred_at: Utc::now(),
        correlation_id: cmd.correlation_id.clone(),
        request_id,
        peer_participant_id,
        processing_result,
    }
}
