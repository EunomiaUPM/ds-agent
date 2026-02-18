use crate::data::entities::dataplane_field::{
    self, EditDataPlaneFieldModel, NewDataPlaneFieldModel,
};
use crate::data::repo_traits::dataplane_fields_repo::{
    DataplaneFieldRepoErrors, DataplaneFieldRepoTrait,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QuerySelect, Set,
};
use urn::Urn;

pub struct DataplaneFieldRepoForSql {
    db: DatabaseConnection,
}

impl DataplaneFieldRepoForSql {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl DataplaneFieldRepoTrait for DataplaneFieldRepoForSql {
    async fn get_all_dataplane_fields(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<dataplane_field::Model>, DataplaneFieldRepoErrors> {
      let fields = dataplane_field::Entity::find()
            .limit(limit.unwrap_or(100))
            .offset(page.map(|p| p * limit.unwrap_or(100)).unwrap_or(0))
            .all(&self.db)
            .await;
        match fields {
            Ok(fields) => Ok(fields),
            Err(e) => Err(DataplaneFieldRepoErrors::ErrorFetchingDataplaneField(e.into())),
        }
    }

    async fn get_batch_dataplane_fields(
        &self,
        ids: &Vec<Urn>,
    ) -> anyhow::Result<Vec<dataplane_field::Model>, DataplaneFieldRepoErrors> {
        let id_strings: Vec<String> = ids.iter().map(|urn| urn.to_string()).collect();
        let result = dataplane_field::Entity::find()
            .filter(dataplane_field::Column::Id.is_in(id_strings))
            .all(&self.db)
            .await
            .map_err(|e| DataplaneFieldRepoErrors::ErrorFetchingDataplaneField(e.into()))?;
        Ok(result)
    }

    async fn get_all_dataplane_fields_by_process_id(
        &self,
        process_id: &Urn,
    ) -> anyhow::Result<Vec<dataplane_field::Model>, DataplaneFieldRepoErrors> {
        let result = dataplane_field::Entity::find()
            .filter(dataplane_field::Column::DataplaneProcessId.eq(process_id.to_string()))
            .all(&self.db)
            .await
            .map_err(|e| DataplaneFieldRepoErrors::ErrorFetchingDataplaneField(e.into()))?;
        Ok(result)
    }

    async fn get_dataplane_field_by_id(
        &self,
        field_id: &Urn,
    ) -> anyhow::Result<Option<dataplane_field::Model>, DataplaneFieldRepoErrors> {
        let result = dataplane_field::Entity::find_by_id(field_id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| DataplaneFieldRepoErrors::ErrorFetchingDataplaneField(e.into()))?;
        Ok(result)
    }

    async fn create_dataplane_field(
        &self,
        process_id: &Urn,
        new_dataplane_field: &NewDataPlaneFieldModel,
    ) -> anyhow::Result<dataplane_field::Model, DataplaneFieldRepoErrors> {
        let id = format!("urn:dataplane-field:{}", uuid::Uuid::new_v4());
        let new_model = dataplane_field::ActiveModel {
            id: ActiveValue::Set(id),
            key: ActiveValue::Set(new_dataplane_field.key.clone()),
            value: ActiveValue::Set(new_dataplane_field.value.clone()),
            dataplane_process_id: ActiveValue::Set(process_id.to_string()),
            // ..Default::default()
        };

        let result = new_model
            .insert(&self.db)
            .await
            .map_err(|e| DataplaneFieldRepoErrors::ErrorCreatingDataplaneField(e.into()))?;
        Ok(result)
    }

    async fn put_dataplane_field(
        &self,
        field_id: &Urn,
        edit_field: &EditDataPlaneFieldModel,
    ) -> anyhow::Result<dataplane_field::Model, DataplaneFieldRepoErrors> {
        let mut model: dataplane_field::ActiveModel =
            dataplane_field::Entity::find_by_id(field_id.to_string())
                .one(&self.db)
                .await
                .map_err(|e| DataplaneFieldRepoErrors::ErrorFetchingDataplaneField(e.into()))?
                .ok_or(DataplaneFieldRepoErrors::DataplaneFieldNotFound)?
                .into();

        if let Some(value) = &edit_field.value {
            model.value = Set(Some(value.clone()));
        }

        let result = model
            .update(&self.db)
            .await
            .map_err(|e| DataplaneFieldRepoErrors::ErrorUpdatingDataplaneField(e.into()))?;
        Ok(result)
    }

    async fn delete_dataplane_field(
        &self,
        field_id: &Urn,
    ) -> anyhow::Result<(), DataplaneFieldRepoErrors> {
        let result = dataplane_field::Entity::delete_by_id(field_id.to_string())
            .exec(&self.db)
            .await
            .map_err(|e| DataplaneFieldRepoErrors::ErrorDeletingDataplaneField(e.into()))?;

        if result.rows_affected == 0 {
            return Err(DataplaneFieldRepoErrors::DataplaneFieldNotFound);
        }
        Ok(())
    }

    async fn delete_all_dataplane_fields_by_process_id(
        &self,
        process_id: &Urn,
    ) -> anyhow::Result<(), DataplaneFieldRepoErrors> {
        let _result = dataplane_field::Entity::delete_many()
            .filter(dataplane_field::Column::DataplaneProcessId.eq(process_id.to_string()))
            .exec(&self.db)
            .await
            .map_err(|e| DataplaneFieldRepoErrors::ErrorDeletingDataplaneField(e.into()))?;
        Ok(())
    }
}
