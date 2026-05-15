use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use sea_orm::ActiveValue::Set;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repo::transfer_process_identifier::{
    TransferIdentifierRepoErrors, TransferIdentifierRepoTrait,
};
use crate::data::sea_orm::mappers::{identifier_from_orm, identifier_to_active_model};
use crate::data::sea_orm::orm::transfer_identifier as orm;
use crate::entities::transfer_process_identifier::TransferProcessIdentifier;

pub(crate) struct SeaOrmTransferIdentifierRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmTransferIdentifierRepo {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn db_err(e: sea_orm::DbErr) -> ymir::errors::Errors {
        TransferIdentifierRepoErrors::ErrorFetchingTransferIdentifier(Box::new(e)).into_errors()
    }
}

#[async_trait::async_trait]
impl TransferIdentifierRepoTrait for SeaOrmTransferIdentifierRepo {
    async fn get_identifiers_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<TransferProcessIdentifier>> {
        orm::Entity::find()
            .filter(orm::Column::TransferProcessId.eq(process_id.to_string()))
            .all(self.db.as_ref())
            .await
            .map_err(Self::db_err)?
            .into_iter()
            .map(identifier_from_orm)
            .collect()
    }

    async fn get_identifiers_by_batch_process_id(
        &self,
        process_id_batch: &Vec<Urn>,
    ) -> Outcome<Vec<TransferProcessIdentifier>> {
        let ids: Vec<String> = process_id_batch.iter().map(|u| u.to_string()).collect();
        orm::Entity::find()
            .filter(orm::Column::TransferProcessId.is_in(ids))
            .all(self.db.as_ref())
            .await
            .map_err(Self::db_err)?
            .into_iter()
            .map(identifier_from_orm)
            .collect()
    }

    async fn get_identifier_by_key(
        &self,
        process_id: &Urn,
        key: &str,
    ) -> Outcome<Option<TransferProcessIdentifier>> {
        orm::Entity::find()
            .filter(orm::Column::TransferProcessId.eq(process_id.to_string()))
            .filter(orm::Column::Key.eq(key))
            .one(self.db.as_ref())
            .await
            .map_err(Self::db_err)?
            .map(identifier_from_orm)
            .transpose()
    }

    async fn upsert_identifier(
        &self,
        process_id: &Urn,
        identifier: &TransferProcessIdentifier,
    ) -> Outcome<TransferProcessIdentifier> {
        let active = identifier_to_active_model(process_id, identifier);
        // ON CONFLICT (transfer_process_id, key) DO UPDATE SET value = excluded.value
        use sea_orm::sea_query::OnConflict;
        orm::Entity::insert(active)
            .on_conflict(
                OnConflict::columns([orm::Column::TransferProcessId, orm::Column::Key])
                    .update_column(orm::Column::Value)
                    .to_owned(),
            )
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TransferIdentifierRepoErrors::ErrorUpsertingTransferIdentifier(Box::new(e)).into_errors())?;

        self.get_identifier_by_key(process_id, &identifier.key)
            .await?
            .ok_or_else(|| TransferIdentifierRepoErrors::TransferIdentifierNotFound.into_errors())
    }

    async fn delete_identifier(&self, process_id: &Urn, key: &str) -> Outcome<()> {
        orm::Entity::delete_many()
            .filter(orm::Column::TransferProcessId.eq(process_id.to_string()))
            .filter(orm::Column::Key.eq(key))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TransferIdentifierRepoErrors::ErrorDeletingTransferIdentifier(Box::new(e)).into_errors())?;
        Ok(())
    }
}
