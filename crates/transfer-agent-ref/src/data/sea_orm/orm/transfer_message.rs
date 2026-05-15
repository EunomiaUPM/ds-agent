use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "transfer_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub transfer_process_id: String,
    pub tenant_id: String,
    pub direction: String,
    pub protocol: String,
    pub message_type: String,
    pub protocol_version: String,
    pub envelope: Json,
    pub occurred_at: DateTimeWithTimeZone,
    pub correlation_id: Option<String>,
    pub request_id: String,
    pub peer_participant_id: String,
    pub processing_result: Json,
    /// Denormalized from processing_result.resulting_state for efficient filtering.
    pub state_transition_to: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
