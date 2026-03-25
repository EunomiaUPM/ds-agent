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

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MissingAction {
    Token,
    Wallet,
    Did,
    Onboarding,
    Key,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BadFormat {
    Sent,
    Received,
    Unknown,
}
impl fmt::Display for MissingAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MissingAction::Token => "Token",
            MissingAction::Wallet => "Wallet",
            MissingAction::Did => "DID",
            MissingAction::Key => "Key",
            MissingAction::Onboarding => "Onboarding",
            _ => "Unknown",
        };
        write!(f, "{}", s)
    }
}
