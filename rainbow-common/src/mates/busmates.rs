/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use serde::{Deserialize, Serialize};

use crate::utils::get_urn;

#[derive(Serialize, Deserialize, Debug)]
pub struct BusMates {
    pub id: String,
    pub participant_id: String,
    pub token: Option<String>,
    pub token_actions: Option<String>,
    pub saved_at: chrono::NaiveDateTime,
    pub last_interaction: chrono::NaiveDateTime
}

impl BusMates {
    pub fn default4consumer(
        id: String,
        participant_id: Option<String>,
        token: Option<String>
    ) -> Self {
        let participant_id = participant_id.unwrap_or_else(|| get_urn(None).to_string());

        Self {
            id,
            participant_id,
            token,
            token_actions: Some("talk".to_string()),
            saved_at: chrono::Utc::now().naive_utc(),
            last_interaction: chrono::Utc::now().naive_utc()
        }
    }

    pub fn default4provider(
        id: String,
        participant_id: Option<String>,
        token: Option<String>
    ) -> Self {
        let participant_id = participant_id.unwrap_or_else(|| get_urn(None).to_string());

        Self {
            id,
            participant_id,
            token,
            token_actions: Some("talk".to_string()),
            saved_at: chrono::Utc::now().naive_utc(),
            last_interaction: chrono::Utc::now().naive_utc()
        }
    }
}
