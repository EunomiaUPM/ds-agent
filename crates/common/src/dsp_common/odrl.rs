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

use serde::{Deserialize, Serialize};
use serde_json::Value;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::utils::get_urn;
// use sea_orm_migration::prelude::ValueType;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum OdrlTypes {
    #[serde(rename = "Offer")]
    Offer,
    #[serde(rename = "Agreement")]
    Agreement,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ContractRequestMessageOfferTypes {
    OfferMessage(OdrlMessageOffer),
    OfferId(ContractRequestMessageOfferOfferId),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ContractRequestMessageOfferOfferId {
    #[serde(rename = "@id")]
    pub id: Urn,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OdrlMessageOffer {
    // PolicyClass
    #[serde(rename = "@id")]
    pub id: Urn,
    #[serde(rename = "profile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<OdrlProfile>,
    #[serde(rename = "permission")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<Vec<OdrlPermission>>, // anyof
    #[serde(rename = "obligation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obligation: Option<Vec<OdrlObligation>>,
    // MessageOffer
    #[serde(rename = "@type")]
    pub _type: OdrlTypes,
    #[serde(rename = "prohibition")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prohibition: Option<Vec<OdrlObligation>>,
    // Offer
    #[serde(rename = "target")]
    pub target: Urn, // anyof
    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Default for OdrlMessageOffer {
    fn default() -> Self {
        Self {
            id: get_urn(None),
            profile: None,
            permission: None,
            obligation: None,
            _type: OdrlTypes::Offer,
            prohibition: None,
            target: get_urn(None),
            description: None,
        }
    }
}

/// Offer is PolicyClass + MessageOffer - Offer
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OdrlOffer {
    // PolicyClass
    #[serde(rename = "@id")]
    pub id: Urn,
    #[serde(rename = "profile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<OdrlProfile>,
    #[serde(rename = "permission")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<Vec<OdrlPermission>>, // anyof
    #[serde(rename = "obligation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obligation: Option<Vec<OdrlObligation>>,
    // MessageOffer
    #[serde(rename = "@type")]
    pub _type: OdrlTypes,
    #[serde(rename = "prohibition")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prohibition: Option<Vec<OdrlObligation>>,
    // Offer
    #[serde(rename = "target")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Urn>, // anyof// anyof
    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Default for OdrlOffer {
    fn default() -> Self {
        OdrlOffer {
            id: get_urn(None),
            profile: None,
            permission: None,
            obligation: None,
            _type: OdrlTypes::Offer,
            prohibition: None,
            target: None,
            description: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OdrlAgreement {
    // PolicyClass
    #[serde(rename = "@id")]
    pub id: Urn,
    #[serde(rename = "profile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<OdrlProfile>,
    #[serde(rename = "permission")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<Vec<OdrlPermission>>, // anyof
    #[serde(rename = "obligation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obligation: Option<Vec<OdrlObligation>>,
    // Agreement
    #[serde(rename = "@type")]
    pub _type: OdrlTypes,
    #[serde(rename = "target")]
    pub target: Urn,
    #[serde(rename = "assigner")]
    pub assigner: String,
    #[serde(rename = "assignee")]
    pub assignee: String,
    #[serde(rename = "timestamp")]
    pub timestamp: Option<String>,
    #[serde(rename = "prohibition")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prohibition: Option<Vec<OdrlObligation>>, // anyof
    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Default for OdrlAgreement {
    fn default() -> OdrlAgreement {
        Self {
            id: get_urn(None),
            profile: None,
            permission: None,
            obligation: None,
            _type: OdrlTypes::Agreement,
            target: get_urn(None),
            assigner: "".to_string(),
            assignee: "".to_string(),
            timestamp: None,
            prohibition: None,
            description: None,
        }
    }
}

// ODRL Profile type
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum OdrlProfile {
    Single(String),
    Multiple(Vec<String>),
}

/// OdrlPermission
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OdrlPermission {
    #[serde(rename = "action")]
    pub action: OdrlAction,
    #[serde(rename = "constraint")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint: Option<Vec<OdrlConstraint>>,
    #[serde(rename = "duty")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duty: Option<OdrlDuty>,
}

/// OdrlDuty
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OdrlDuty {
    #[serde(rename = "action")]
    pub action: OdrlAction,
    #[serde(rename = "constraint")]
    pub constraint: Option<Vec<OdrlConstraint>>,
}

/// OdrlObligation
pub type OdrlObligation = OdrlDuty;

/// OdrlAction
pub type OdrlAction = String;

/// OdrlConstraint
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum OdrlConstraint {
    Atomic(OdrlAtomicConstraint),
    Logical(OdrlLogicalConstraint),
}

/// LogicalConstraint permite una de las siguientes propiedades: "and", "andSequence", "or" o "xone".
/// Se usan Option para cada una;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OdrlLogicalConstraint {
    #[serde(rename = "and")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub and: Option<Vec<OdrlConstraint>>,
    #[serde(rename = "andSequence", skip_serializing_if = "Option::is_none")]
    pub and_sequence: Option<Vec<OdrlConstraint>>,
    #[serde(rename = "or")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub or: Option<Vec<OdrlConstraint>>,
    #[serde(rename = "xone")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xone: Option<Vec<OdrlConstraint>>,
}

/// the rule that exactly one must be present is validated externally.
/// let constraint: LogicalConstraint = serde_json::from_str(json_data)?;
/// constraint.validate()?; // if it fails, an error is returned.
impl OdrlLogicalConstraint {
    pub fn validate(&self) -> Outcome<()> {
        let count = self.and.is_some() as usize
            + self.and_sequence.is_some() as usize
            + self.or.is_some() as usize
            + self.xone.is_some() as usize;
        if count != 1 {
            Err(Errors::format(
                BadFormat::Received,
                format!(
                    "Exactly one of 'and', 'andSequence', 'or' or 'xone' must be present, found {}",
                    count
                ),
                None,
            ))
        } else {
            Ok(())
        }
    }
}

// AtomicConstraint defines the three required fields: rightOperand, leftOperand and operator.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OdrlAtomicConstraint {
    #[serde(rename = "rightOperand")]
    pub right_operand: OdrlRightOperand,
    #[serde(rename = "leftOperand")]
    pub left_operand: OdrlLeftOperand,
    #[serde(rename = "operator")]
    pub operator: Operator,
}

// Operator is defined as an enum with allowed values.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum Operator {
    #[serde(rename = "eq")]
    Eq,
    #[serde(rename = "gt")]
    Gt,
    #[serde(rename = "gteq")]
    Gteq,
    #[serde(rename = "lteq")]
    Lteq,
    #[serde(rename = "hasPart")]
    HasPart,
    #[serde(rename = "isA")]
    IsA,
    #[serde(rename = "isAllOf")]
    IsAllOf,
    #[serde(rename = "isAnyOf")]
    IsAnyOf,
    #[serde(rename = "isNoneOf")]
    IsNoneOf,
    #[serde(rename = "isPartOf")]
    IsPartOf,
    #[serde(rename = "lt")]
    Lt,
    #[serde(rename = "termLteq")]
    TermLteq,
    #[serde(rename = "neq")]
    Neq,
}

// RightOperand is defined to accept string, object or array.
// serde_json::Value is used to allow this variability.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum OdrlRightOperand {
    Str(String),
    Object(serde_json::Map<String, Value>),
    Array(Vec<Value>),
}

// LeftOperand es un string.
pub type OdrlLeftOperand = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OdrlPolicyInfo {
    #[serde(rename = "profile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<OdrlProfile>,
    #[serde(rename = "permission")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<Vec<OdrlPermission>>, // anyof
    #[serde(rename = "obligation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obligation: Option<Vec<OdrlObligation>>,
    #[serde(rename = "prohibition")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prohibition: Option<Vec<OdrlObligation>>, // anyof
    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
