use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "dataplane_fields")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub key: String,
    pub value: Option<String>,
    pub data_plane_process_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::dataplane_transfers::Entity",
        from = "Column::DataPlaneProcessId",
        to = "super::dataplane_transfers::Column::TransferProcessId",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    DataplaneTransfers,
}

impl Related<super::dataplane_transfers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DataplaneTransfers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDataPlaneFieldModel {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditDataPlaneFieldModel {
    pub value: Option<String>,
}
