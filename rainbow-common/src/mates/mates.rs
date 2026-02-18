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

use crate::utils::get_urn;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mates {
    pub participant_id: String,
    pub participant_slug: String,
    pub participant_type: String,
    pub base_url: Option<String>,
    pub token: Option<String>,
    pub token_actions: Option<String>,
    pub saved_at: chrono::NaiveDateTime,
    pub last_interaction: chrono::NaiveDateTime,
    pub is_me: bool,
}

impl Mates {
    pub fn default4consumer(
        id: Option<String>,
        slug: String,
        url: String,
        token: Option<String>,
        token_actions: Option<String>,
        is_me: bool,
    ) -> Self {
        let participant_id = id.unwrap_or_else(|| get_urn(None).to_string());

        Self {
            participant_id,
            participant_slug: slug,
            participant_type: "Provider".to_string(),
            base_url: Some(url),
            token,
            token_actions,
            saved_at: chrono::Utc::now().naive_utc(),
            last_interaction: chrono::Utc::now().naive_utc(),
            is_me,
        }
    }

    pub fn default4provider(
        id: Option<String>,
        slug: String,
        url: String,
        token: Option<String>,
        token_actions: Option<String>,
        is_me: bool,
    ) -> Self {
        let participant_id = id.unwrap_or_else(|| get_urn(None).to_string());

        Self {
            participant_id,
            participant_slug: slug,
            participant_type: "Consumer".to_string(),
            base_url: Some(url),
            token,
            token_actions,
            saved_at: chrono::Utc::now().naive_utc(),
            last_interaction: chrono::Utc::now().naive_utc(),
            is_me,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
