use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "transfer_identifiers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub transfer_process_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub key: String,
    pub value: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
