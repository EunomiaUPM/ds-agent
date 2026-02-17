use crate::data::entities::dataplane_transfer_logs;
use crate::data::repo_traits::dataplane_transfer_logs_repo::DataplaneTransferLogsRepo;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub struct DataplaneTransferLogsRepoForSql {
    db: DatabaseConnection,
}

impl DataplaneTransferLogsRepoForSql {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl DataplaneTransferLogsRepo for DataplaneTransferLogsRepoForSql {
    async fn create_log(
        &self,
        new_log: dataplane_transfer_logs::NewTransferLog,
    ) -> anyhow::Result<dataplane_transfer_logs::Model> {
        let active_model: dataplane_transfer_logs::ActiveModel = new_log.into();
        let result = active_model.insert(&self.db).await?;
        Ok(result)
    }

    async fn get_all_transfer_logs(&self) -> anyhow::Result<Vec<dataplane_transfer_logs::Model>> {
        let logs = dataplane_transfer_logs::Entity::find().all(&self.db).await?;
        Ok(logs)
    }

    async fn get_transfer_log_by_id(
        &self,
        log_id: &uuid::Uuid,
    ) -> anyhow::Result<Option<dataplane_transfer_logs::Model>> {
        let log = dataplane_transfer_logs::Entity::find_by_id(*log_id).one(&self.db).await?;
        Ok(log)
    }

    async fn get_transfer_logs_by_transfer_id(
        &self,
        transfer_id: &uuid::Uuid,
    ) -> anyhow::Result<Vec<dataplane_transfer_logs::Model>> {
        let logs = dataplane_transfer_logs::Entity::find()
            .filter(dataplane_transfer_logs::Column::TransferId.eq(*transfer_id))
            .all(&self.db)
            .await?;
        Ok(logs)
    }
}
