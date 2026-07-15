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

use sea_orm::Value;
use std::fmt::Display;
use ymir::errors::Errors;

pub mod catalog_definition;
pub mod dataservice_definition;
pub mod dataset_definition;
pub mod distribution_definition;

pub enum EntityTypes {
    Catalog,
    Dataset,
    DataService,
    Distribution,
}

pub trait CatalogDspTraitDefinition {}

impl TryFrom<&str> for EntityTypes {
    type Error = Errors;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Catalog" => Ok(EntityTypes::Catalog),
            "Dataset" => Ok(EntityTypes::Dataset),
            "DataService" => Ok(EntityTypes::DataService),
            "Distribution" => Ok(EntityTypes::Distribution),
            _ => Err(Errors::parse("Entity not recognised", None)),
        }
    }
}

impl Display for EntityTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            EntityTypes::Catalog => "Catalog".to_string(),
            EntityTypes::Dataset => "Dataset".to_string(),
            EntityTypes::DataService => "DataService".to_string(),
            EntityTypes::Distribution => "Distribution".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl From<EntityTypes> for Value {
    fn from(value: EntityTypes) -> Self {
        Self::String(Some(Box::new(value.to_string())))
    }
}
