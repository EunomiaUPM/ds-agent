use crate::entities::ids::{TenantId, TransferProcessId};
use crate::entities::protocol::{
    ProtocolId, ProtocolState, StateMetadata, TransferCorrelation, TransferRole,
};
use crate::entities::transfer_process::TransferProcess;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value as Json;
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransferProcessView {
    pub id: TransferProcessId,
    pub tenant_id: TenantId,
    pub role: TransferRole,
    pub protocol: ProtocolId,
    pub state: ProtocolState,
    pub state_metadata: StateMetadata,
    pub correlation: TransferCorrelation,
    pub extra_identifiers: HashMap<String, String>,
    pub properties: Json,
    pub error_details: Option<Json>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u64,
}

impl TransferProcessView {
    pub(crate) fn assemble(
        process: TransferProcess,
        extra_identifiers: HashMap<String, String>,
    ) -> Self {
        Self {
            id: process.id().clone(),
            tenant_id: process.tenant_id().clone(),
            role: process.role(),
            protocol: process.protocol().clone(),
            state: process.state().clone(),
            state_metadata: process.state_metadata().clone(),
            correlation: process.correlation().clone(),
            extra_identifiers,
            properties: process.properties().clone(),
            error_details: process.error_details().cloned(),
            created_at: process.created_at(),
            updated_at: process.updated_at(),
            version: process.version(),
        }
    }
}
