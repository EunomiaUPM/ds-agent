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

//! ODRL offers, agreements, rules and constraints as they travel in DSP messages.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::utils::get_urn;
// use sea_orm_migration::prelude::ValueType;

/// Policy type of an ODRL document.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum OdrlTypes {
    #[serde(rename = "Offer")]
    Offer,
    #[serde(rename = "Agreement")]
    Agreement,
}

/// Offer in a contract request: the full offer or just its id.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ContractRequestMessageOfferTypes {
    OfferMessage(OdrlMessageOffer),
    OfferId(ContractRequestMessageOfferOfferId),
}

/// Offer referenced by id only.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ContractRequestMessageOfferOfferId {
    #[serde(rename = "@id")]
    pub id: Urn,
}

/// Offer sent inside a DSP message, where `target` is required.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

/// Offer as the catalog stores it, where `target` is optional.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

/// Agreement reached at the end of a negotiation, with both parties and its target.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

/// ODRL profile, as one IRI or a list.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum OdrlProfile {
    Single(String),
    Multiple(Vec<String>),
}

/// Permission rule: an action, its constraints and an optional duty.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

/// Duty rule: an action the assignee must perform, with optional constraints.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OdrlDuty {
    #[serde(rename = "action")]
    pub action: OdrlAction,
    #[serde(rename = "constraint")]
    pub constraint: Option<Vec<OdrlConstraint>>,
}

/// Obligations and prohibitions share the duty shape.
pub type OdrlObligation = OdrlDuty;

/// ODRL action name, such as `use`.
pub type OdrlAction = String;

/// Constraint on a rule: a single comparison or a logical combination.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum OdrlConstraint {
    Atomic(OdrlAtomicConstraint),
    Logical(OdrlLogicalConstraint),
}

/// Logical combination of constraints; exactly one operator must be set.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

impl OdrlLogicalConstraint {
    /// Fails unless exactly one of `and`, `andSequence`, `or` or `xone` is set.
    pub fn validate(&self) -> Outcome<()> {
        let count = self.and.is_some() as usize
            + self.and_sequence.is_some() as usize
            + self.or.is_some() as usize
            + self.xone.is_some() as usize;
        if count != 1 {
            Err(Errors::format(
                BadFormat::Received,
                format!(
                    "Exactly one of 'and', 'andSequence', 'or' or 'xone' must be present, found {count}"
                ),
                None,
            ))
        } else {
            Ok(())
        }
    }
}

/// Single comparison: left operand, operator and right operand.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OdrlAtomicConstraint {
    #[serde(rename = "rightOperand")]
    pub right_operand: OdrlRightOperand,
    #[serde(rename = "leftOperand")]
    pub left_operand: OdrlLeftOperand,
    #[serde(rename = "operator")]
    pub operator: Operator,
}

/// ODRL comparison operators.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

/// Right operand of a constraint: a string, an object or a list.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum OdrlRightOperand {
    Str(String),
    Object(serde_json::Map<String, Value>),
    Array(Vec<Value>),
}

/// Left operand of a constraint, such as `dateTime`.
pub type OdrlLeftOperand = String;

/// Rules of a policy without its identity or target, as stored in templates and offers.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
