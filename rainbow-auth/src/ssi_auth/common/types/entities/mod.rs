/*
 *
 *  * Copyright (C) 2024 - Universidad Politécnica de Madrid - UPM
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

use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Url2RequestVC {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ReachAuthority {
    pub id: String,
    pub slug: String,
    pub url: String,
    pub vc_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReachProvider {
    pub id: String,
    pub slug: String,
    pub url: String,
    pub actions: String,
}

pub enum WhatEntity {
    Provider,
    Authority,
}

impl PartialEq for WhatEntity {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other), (WhatEntity::Provider, WhatEntity::Provider) | (WhatEntity::Authority, WhatEntity::Authority))
    }
}

impl fmt::Debug for WhatEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        match self {
            WhatEntity::Provider => write!(f, "Agent"),
            WhatEntity::Authority => write!(f, "Authority"),
        }

    }
}

#[derive(Clone)]
pub enum InteractStart {
    Oidc,
    CrossUser,
}

impl fmt::Debug for InteractStart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InteractStart::Oidc => write!(f, "Oidc"),
            InteractStart::CrossUser => write!(f, "CrossUser"),
        }
    }
}

impl PartialEq for InteractStart {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other), (InteractStart::Oidc, InteractStart::Oidc) | (InteractStart::CrossUser, InteractStart::CrossUser))
    }
}
