/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::data::repo::EventRepoErrors;
use axum::extract::rejection::JsonRejection;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use urn::Urn;

#[derive(Error, Debug)]
pub enum NotificationErrors {
    #[error("Error from database: {0}")]
    DbErr(EventRepoErrors),
    #[error("{entity} with id {} not found", id.as_str())]
    NotFound { id: Urn, entity: String },
    #[error("Error from deserializing JSON: {0}")]
    JsonRejection(JsonRejection),
    #[error("Error from deserializing path. {0}")]
    UrnUuidSchema(String),
}

#[derive(Serialize, Deserialize)]
pub struct NotificationErrorMessage {
    pub code: String,
    pub title: String,
    pub message: String,
}
