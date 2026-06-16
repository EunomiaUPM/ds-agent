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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use ymir::errors::Errors;

pub enum WhoEntity {
    Authority,
    Provider,
}

impl FromStr for WhoEntity {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "authority" => Ok(WhoEntity::Authority),
            "provider" => Ok(WhoEntity::Provider),
            format => Err(Errors::parse(format!("Unknown entity: {}", format), None)),
        }
    }
}

impl Display for WhoEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            WhoEntity::Authority => "Authority",
            WhoEntity::Provider => "Provider",
        };

        write!(f, "{s}")
    }
}
