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

//! Process-wide infrastructure built once at the composition root and injected into every module.

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use ymir::errors::Outcome;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;

use crate::oauth::{TokenValidatorTrait, token_validator};
use crate::config::services::CommonConfig;

/// One vault, one DB pool and one token validator for the whole process.
#[derive(Clone)]
pub struct RootContext {
    pub vault: Arc<VaultService>,
    pub db: DatabaseConnection,
    pub validator: Arc<dyn TokenValidatorTrait>,
}

impl RootContext {
    /// Opens the database through the vault and builds the validator of the configured
    /// provider.
    pub async fn connect(common: &CommonConfig, vault: Arc<VaultService>) -> Outcome<Self> {
        let db = vault.get_db_connection(common).await?;
        Ok(Self {
            validator: token_validator(common)?,
            vault,
            db,
        })
    }
}
