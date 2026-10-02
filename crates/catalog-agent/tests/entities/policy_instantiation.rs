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

//! NewPolicyInstantiationDto validation against a template: known parameters, required ones,
//! defaults and the restrictions of each type.

use catalog_agent::entities::odrl_policies::CatalogEntityTypes;
use catalog_agent::entities::policy_instantiation::NewPolicyInstantiationDto;
use catalog_agent::entities::policy_templates::PolicyTemplateAllowedDefaultValues::{
    Numerable, Stringable,
};
use catalog_agent::entities::policy_templates::{
    PolicyTemplateAllowedDefaultValues, PolicyTemplateDto,
};
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use urn::Urn;

use crate::support::builders::policy_template;

/// Template with a ranged integer with default, a selection without default and a
/// constrained string with default.
fn template() -> PolicyTemplateDto {
    policy_template(
        json!({
            "$maxUses": {"dataType": "integer", "restrictions": {"minValue": 1, "maxValue": 10}, "defaultValue": 3},
            "$region": {"dataType": "selection", "restrictions": {"values": ["eu", "us"]}},
            "$label": {"dataType": "string", "restrictions": {"regex": "^[a-z]+$", "maxLength": 8}, "defaultValue": "data"}
        }),
        json!({"permission": [{"action": "use"}]}),
    )
}

fn request(params: &[(&str, PolicyTemplateAllowedDefaultValues)]) -> NewPolicyInstantiationDto {
    NewPolicyInstantiationDto {
        id: "tpl-1".to_string(),
        version: "1".to_string(),
        parameters: params
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect::<HashMap<_, _>>(),
        entity_id: Urn::from_str("urn:uuid:00000000-0000-0000-0000-000000000100").unwrap(),
        entity_type: CatalogEntityTypes::Dataset,
        description: None,
    }
}

/// Parameters without value fall back to their defaults; only the required one must be given.
#[test]
fn defaults_cover_omitted_parameters() {
    let req = request(&[("$region", Stringable("eu".into()))]);
    assert!(req.validate_instantiation_request(&template()).is_ok());
}

/// A parameter the template does not define is rejected.
#[test]
fn unknown_parameter_is_rejected() {
    let req = request(&[
        ("$region", Stringable("eu".into())),
        ("$other", Stringable("x".into())),
    ]);
    assert!(req.validate_instantiation_request(&template()).is_err());
}

/// A parameter without default must be given.
#[test]
fn missing_required_parameter_is_rejected() {
    assert!(request(&[])
        .validate_instantiation_request(&template())
        .is_err());
}

/// An integer must be a number within its range.
#[test]
fn integer_out_of_range_or_not_a_number_is_rejected() {
    for value in [Numerable(0.0), Numerable(11.0), Stringable("5".into())] {
        let req = request(&[("$region", Stringable("eu".into())), ("$maxUses", value)]);
        assert!(req.validate_instantiation_request(&template()).is_err());
    }
    let ok = request(&[
        ("$region", Stringable("eu".into())),
        ("$maxUses", Numerable(10.0)),
    ]);
    assert!(ok.validate_instantiation_request(&template()).is_ok());
}

/// A selection must be one of the allowed values.
#[test]
fn selection_outside_allowed_values_is_rejected() {
    let req = request(&[("$region", Stringable("asia".into()))]);
    assert!(req.validate_instantiation_request(&template()).is_err());
}

/// A string must match its regex and length limit.
#[test]
fn string_breaking_regex_or_length_is_rejected() {
    for label in ["Upper", "waytoolongvalue"] {
        let req = request(&[
            ("$region", Stringable("eu".into())),
            ("$label", Stringable(label.into())),
        ]);
        assert!(
            req.validate_instantiation_request(&template()).is_err(),
            "{label}"
        );
    }
}
