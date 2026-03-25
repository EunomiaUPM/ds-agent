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

use common::config::types::roles::RoleConfig;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::StringLen::N;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use urn::{Urn, UrnBuilder};

use crate::DataplaneInitCommandType;
use strum::Display;
use ymir::errors::Errors;

#[derive(
    Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Display,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum TransferRole {
    #[sea_orm(string_value = "Provider")]
    Provider,
    #[sea_orm(string_value = "Consumer")]
    Consumer,
}

impl TryFrom<RoleConfig> for TransferRole {
    type Error = Errors;

    fn try_from(value: RoleConfig) -> Result<Self, Self::Error> {
        match value {
            RoleConfig::Consumer => Ok(Self::Consumer),
            RoleConfig::Provider => Ok(Self::Provider),
            RoleConfig::NotDefined => Err(Errors::crazy(
                "Not allowed here this role. Dataplane must have Provider or Consumer Role",
                None,
            )),
        }
    }
}

impl TryFrom<&RoleConfig> for TransferRole {
    type Error = Errors;

    fn try_from(value: &RoleConfig) -> Result<Self, Self::Error> {
        match value {
            RoleConfig::Consumer => Ok(Self::Consumer),
            RoleConfig::Provider => Ok(Self::Provider),
            RoleConfig::NotDefined => Err(Errors::crazy(
                "Not allowed here this role. Dataplane must have Provider or Consumer Role",
                None,
            )),
        }
    }
}

impl TryFrom<DataplaneInitCommandType> for TransferRole {
    type Error = Errors;
    fn try_from(value: DataplaneInitCommandType) -> Result<Self, Self::Error> {
        match value {
            DataplaneInitCommandType::Provider { .. } => Ok(Self::Provider),
            DataplaneInitCommandType::Consumer { .. } => Ok(Self::Consumer),
        }
    }
}

impl TryFrom<&DataplaneInitCommandType> for TransferRole {
    type Error = Errors;
    fn try_from(value: &DataplaneInitCommandType) -> Result<Self, Self::Error> {
        match value {
            DataplaneInitCommandType::Provider { .. } => Ok(Self::Provider),
            DataplaneInitCommandType::Consumer { .. } => Ok(Self::Consumer),
        }
    }
}

#[derive(
    Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Display,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum InteractionMode {
    #[sea_orm(string_value = "Pull")]
    Pull,
    #[sea_orm(string_value = "Push")]
    Push,
}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum TransferState {
    #[sea_orm(string_value = "INIT")]
    Init,
    #[sea_orm(string_value = "CONFIGURING")]
    Configuring,
    #[sea_orm(string_value = "AUTH")]
    Auth,
    #[sea_orm(string_value = "READY")]
    Ready,
    #[sea_orm(string_value = "SUBSCRIBING")]
    Subscribing,
    #[sea_orm(string_value = "STARTED")]
    Started,
    #[sea_orm(string_value = "UNSUBSCRIBING")]
    Unsubscribing,
    #[sea_orm(string_value = "STOPPED")]
    Stopped,
    #[sea_orm(string_value = "TERMINATED")]
    Terminated,
    #[sea_orm(string_value = "ERROR")]
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "dataplane_transfers")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub transfer_process_id: String,
    pub role: TransferRole,
    pub interaction_mode: InteractionMode,
    pub state: TransferState,
    pub connector_instance_id: Option<String>,
    pub ingress_config: Json,
    pub egress_config: Json,
    pub flow_control: Option<Json>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::dataplane_transfer_logs::Entity")]
    TransferLogs,
}

impl Related<super::dataplane_transfer_logs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TransferLogs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone)]
pub struct NewDataplaneTransfer {
    pub id: Option<Urn>,
    pub transfer_process_id: String,
    pub role: TransferRole,
    pub interaction_mode: InteractionMode,
    pub state: TransferState,
    pub connector_instance_id: Option<Urn>,
    pub ingress_config: serde_json::Value,
    pub egress_config: serde_json::Value,
}

impl From<NewDataplaneTransfer> for ActiveModel {
    fn from(value: NewDataplaneTransfer) -> Self {
        let new_urn = UrnBuilder::new(
            "dataplane-process",
            uuid::Uuid::new_v4().to_string().as_str(),
        )
        .build()
        .expect("UrnBuilder failed");
        Self {
            id: ActiveValue::Set(value.id.unwrap_or(new_urn.clone()).to_string()),
            transfer_process_id: ActiveValue::Set(value.transfer_process_id),
            role: ActiveValue::Set(value.role),
            interaction_mode: ActiveValue::Set(value.interaction_mode),
            state: ActiveValue::Set(value.state),
            connector_instance_id: ActiveValue::Set(
                value.connector_instance_id.map(|urn| urn.to_string()),
            ),
            ingress_config: ActiveValue::Set(value.ingress_config),
            egress_config: ActiveValue::Set(value.egress_config),
            flow_control: ActiveValue::Set(Some(serde_json::json!({}))),
            created_at: ActiveValue::Set(chrono::Utc::now().into()),
            updated_at: ActiveValue::Set(None),
        }
    }
}

pub type NewDataplaneTransferModel = NewDataplaneTransfer;

#[derive(Clone, Debug)]
pub struct EditDataplaneTransferModel {
    pub state: Option<TransferState>,
    pub connector_instance_id: Option<Urn>,
    pub ingress_config: Option<serde_json::Value>,
    pub egress_config: Option<serde_json::Value>,
    pub flow_control: Option<serde_json::Value>,
}

impl From<EditDataplaneTransferModel> for ActiveModel {
    fn from(value: EditDataplaneTransferModel) -> Self {
        let mut active_model = Self {
            updated_at: ActiveValue::Set(Some(chrono::Utc::now().into())),
            ..Default::default()
        };

        if let Some(state) = value.state {
            active_model.state = ActiveValue::Set(state);
        }

        if let Some(connector_instance_id) = value.connector_instance_id {
            active_model.connector_instance_id =
                ActiveValue::Set(Some(connector_instance_id.to_string()));
        }

        if let Some(ingress_config) = value.ingress_config {
            active_model.ingress_config = ActiveValue::Set(ingress_config);
        }

        if let Some(egress_config) = value.egress_config {
            active_model.egress_config = ActiveValue::Set(egress_config);
        }

        if let Some(flow_control) = value.flow_control {
            active_model.flow_control = ActiveValue::Set(Some(flow_control));
        }

        active_model
    }
}
