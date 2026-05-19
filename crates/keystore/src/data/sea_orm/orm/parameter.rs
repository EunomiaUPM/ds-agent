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

use chrono::Utc;
use sea_orm::prelude::{DateTimeWithTimeZone, Json};
use sea_orm::{
    ActiveModelBehavior, ActiveValue, DeriveEntityModel, DerivePrimaryKey, DeriveRelation,
    EnumIter, PrimaryKeyTrait,
};

use crate::entities::commands::{EditParameterCommand, NewParameterCommand};
use crate::entities::entry::Entry;
use crate::entities::key::Key;
use crate::entities::metadata::Metadata;
use crate::entities::version::Version;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "keystore_parameters")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub key: String,
    pub value: Json,
    pub version: i64,
    pub description: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub created_by: String,
    pub updated_by: String,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn from_new_cmd(cmd: &NewParameterCommand<serde_json::Value>) -> Self {
        let now: DateTimeWithTimeZone = Utc::now().into();
        Self {
            key: ActiveValue::Set(cmd.key.as_str().to_owned()),
            value: ActiveValue::Set(cmd.value.clone()),
            version: ActiveValue::Set(Version::INITIAL.value() as i64),
            description: ActiveValue::Set(cmd.description.clone()),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
            created_by: ActiveValue::Set(String::new()),
            updated_by: ActiveValue::Set(String::new()),
            deleted_at: ActiveValue::Set(None),
        }
    }

    pub fn apply_edit_cmd(
        mut self,
        cmd: &EditParameterCommand<serde_json::Value>,
        next_version: i64,
    ) -> Self {
        self.value = ActiveValue::Set(cmd.value.clone());
        self.version = ActiveValue::Set(next_version);
        self.updated_at = ActiveValue::Set(Utc::now().into());
        self.updated_by = ActiveValue::Set(String::new());
        if let Some(desc) = &cmd.description {
            self.description = ActiveValue::Set(Some(desc.clone()));
        }
        self
    }
}

// Model -> domain ───────────────────────────────────────────────────────────

impl Model {
    pub fn into_entry(self) -> Result<Entry<serde_json::Value>, String> {
        let key = Key::new(self.key).map_err(|e| e.to_string())?;
        Ok(Entry {
            metadata: Metadata {
                key,
                version: Version::new(self.version as u64),
                created_at: self.created_at.with_timezone(&Utc),
                updated_at: self.updated_at.with_timezone(&Utc),
                created_by: self.created_by,
                updated_by: self.updated_by,
                deleted_at: self.deleted_at.map(|dt| dt.with_timezone(&Utc)),
                description: self.description,
            },
            value: self.value,
        })
    }
}
