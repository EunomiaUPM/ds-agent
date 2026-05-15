use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "transfer_processes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub tenant_id: String,
    pub role: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub version: i32,
    pub protocol: String,
    pub protocol_state: String,
    pub state_metadata: Json,
    /// Denormalized from correlation.agreement_id for efficient filtering.
    pub agreement_id: Option<String>,
    /// Denormalized from correlation.peer_participant_id for efficient filtering.
    pub peer_participant_id: Option<String>,
    pub correlation: Json,
    pub properties: Json,
    pub error_details: Option<Json>,
    pub last_inbound_envelope: Option<Json>,
    pub last_outbound_envelope: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
