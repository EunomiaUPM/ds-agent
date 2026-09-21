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

use crate::data::entities::dataplane_field::{
    self, Column, EditDataPlaneFieldModel, Entity as DataplaneFieldEntity, NewDataPlaneFieldModel,
};
use crate::data::repo::dataplane_field::{DataplaneFieldRepoErrors, DataplaneFieldRepoTrait};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

pub struct DataplaneFieldRepoForSql {
    db: Arc<DatabaseConnection>,
}

impl DataplaneFieldRepoForSql {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub fn new_with_raw_db(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }
}

#[async_trait::async_trait]
impl DataplaneFieldRepoTrait for DataplaneFieldRepoForSql {
    async fn get_all_dataplane_fields_by_process_id(
        &self,
        tenant_id: &str,
        process_id: &Urn,
    ) -> Outcome<Vec<dataplane_field::Model>> {
        let result = DataplaneFieldEntity::find()
            .filter(Column::DataplaneProcessId.eq(process_id.to_string()))
            .filter(Column::TenantId.eq(tenant_id))
            .all(self.db.as_ref())
            .await
            .map_err(|e| {
                DataplaneFieldRepoErrors::ErrorFetchingDataplaneField(Box::new(e)).into_errors()
            })?;
        Ok(result)
    }

    async fn get_dataplane_field_by_id(
        &self,
        tenant_id: &str,
        field_id: &Urn,
    ) -> Outcome<Option<dataplane_field::Model>> {
        let result = DataplaneFieldEntity::find_by_id(field_id.to_string())
            .filter(Column::TenantId.eq(tenant_id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| {
                DataplaneFieldRepoErrors::ErrorFetchingDataplaneField(Box::new(e)).into_errors()
            })?;
        Ok(result)
    }

    async fn create_dataplane_field(
        &self,
        tenant_id: &str,
        process_id: &Urn,
        new_dataplane_field: &NewDataPlaneFieldModel,
    ) -> Outcome<dataplane_field::Model> {
        let id = format!("urn:dataplane-field:{}", uuid::Uuid::new_v4());
        let new_model = dataplane_field::ActiveModel {
            id: ActiveValue::Set(id),
            tenant_id: ActiveValue::Set(tenant_id.to_string()),
            key: ActiveValue::Set(new_dataplane_field.key.clone()),
            value: ActiveValue::Set(new_dataplane_field.value.clone()),
            dataplane_process_id: ActiveValue::Set(process_id.to_string()),
        };

        let result = new_model.insert(self.db.as_ref()).await.map_err(|e| {
            DataplaneFieldRepoErrors::ErrorCreatingDataplaneField(Box::new(e)).into_errors()
        })?;
        Ok(result)
    }

    async fn put_dataplane_field(
        &self,
        tenant_id: &str,
        field_id: &Urn,
        edit_field: &EditDataPlaneFieldModel,
    ) -> Outcome<dataplane_field::Model> {
        let mut model: dataplane_field::ActiveModel =
            DataplaneFieldEntity::find_by_id(field_id.to_string())
                .filter(Column::TenantId.eq(tenant_id))
                .one(self.db.as_ref())
                .await
                .map_err(|e| {
                    DataplaneFieldRepoErrors::ErrorFetchingDataplaneField(Box::new(e)).into_errors()
                })?
                .ok_or_else(|| DataplaneFieldRepoErrors::DataplaneFieldNotFound.into_errors())?
                .into();

        if let Some(value) = &edit_field.value {
            model.value = Set(Some(value.clone()));
        }

        let result = model.update(self.db.as_ref()).await.map_err(|e| {
            DataplaneFieldRepoErrors::ErrorUpdatingDataplaneField(Box::new(e)).into_errors()
        })?;
        Ok(result)
    }

    async fn delete_dataplane_field(&self, tenant_id: &str, field_id: &Urn) -> Outcome<()> {
        let result = DataplaneFieldEntity::delete_many()
            .filter(Column::Id.eq(field_id.to_string()))
            .filter(Column::TenantId.eq(tenant_id))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| {
                DataplaneFieldRepoErrors::ErrorDeletingDataplaneField(Box::new(e)).into_errors()
            })?;

        if result.rows_affected == 0 {
            return Err(DataplaneFieldRepoErrors::DataplaneFieldNotFound.into_errors());
        }
        Ok(())
    }

    async fn delete_all_dataplane_fields_by_process_id(
        &self,
        tenant_id: &str,
        process_id: &Urn,
    ) -> Outcome<()> {
        DataplaneFieldEntity::delete_many()
            .filter(Column::DataplaneProcessId.eq(process_id.to_string()))
            .filter(Column::TenantId.eq(tenant_id))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| {
                DataplaneFieldRepoErrors::ErrorDeletingDataplaneField(Box::new(e)).into_errors()
            })?;
        Ok(())
    }
}
