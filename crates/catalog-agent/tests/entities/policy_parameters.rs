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

//! Template parameter values: known parameters, required ones, defaults and the restrictions of
//! each type; also the placeholders of template content.

use catalog_agent::entities::policy_templates::placeholder::Placeholder;
use catalog_agent::entities::policy_templates::ParameterValue::{self, Boolean, List, Number, Text};
use catalog_agent::entities::policy_templates::PolicyTemplateDto;
use serde_json::{json, Value};
use std::collections::HashMap;

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

fn int(n: i64) -> ParameterValue {
    Number(n.into())
}

fn float(n: f64) -> ParameterValue {
    Number(serde_json::Number::from_f64(n).unwrap())
}

/// Template with one parameter of the given definition, named `$p`.
fn single(definition: Value) -> PolicyTemplateDto {
    policy_template(json!({ "$p": definition }), json!({"permission": [{"action": "use"}]}))
}

fn accepts(definition: Value, value: ParameterValue) -> bool {
    single(definition).resolve_values(&request(&[("$p", value)]))
        .is_ok()
}

fn request(params: &[(&str, ParameterValue)]) -> HashMap<String, Value> {
    params.iter().map(|(k, v)| (k.to_string(), v.to_json())).collect()
}

/// Parameters without value fall back to their defaults; only the required one must be given.
#[test]
fn defaults_cover_omitted_parameters() {
    let req = request(&[("$region", Text("eu".into()))]);
    assert!(template().resolve_values(&req).is_ok());
}

/// A parameter the template does not define is rejected.
#[test]
fn unknown_parameter_is_rejected() {
    let req = request(&[
        ("$region", Text("eu".into())),
        ("$other", Text("x".into())),
    ]);
    assert!(template().resolve_values(&req).is_err());
}

/// A parameter without default must be given.
#[test]
fn missing_required_parameter_is_rejected() {
    assert!(template().resolve_values(&request(&[]))
        .is_err());
}

/// An integer must be a number within its range.
#[test]
fn integer_out_of_range_or_not_a_number_is_rejected() {
    for value in [int(0), int(11), float(5.5), Text("5".into())] {
        let req = request(&[("$region", Text("eu".into())), ("$maxUses", value)]);
        assert!(template().resolve_values(&req).is_err());
    }
    let ok = request(&[
        ("$region", Text("eu".into())),
        ("$maxUses", int(10)),
    ]);
    assert!(template().resolve_values(&ok).is_ok());
}

/// A selection must be one of the allowed values.
#[test]
fn selection_outside_allowed_values_is_rejected() {
    let req = request(&[("$region", Text("asia".into()))]);
    assert!(template().resolve_values(&req).is_err());
}

/// A string must match its regex and length limit.
#[test]
fn string_breaking_regex_or_length_is_rejected() {
    for label in ["Upper", "waytoolongvalue"] {
        let req = request(&[
            ("$region", Text("eu".into())),
            ("$label", Text(label.into())),
        ]);
        assert!(
            template().resolve_values(&req).is_err(),
            "{label}"
        );
    }
}

/// A boolean takes true or false, not their text.
#[test]
fn boolean_takes_only_booleans() {
    let def = json!({"dataType": "boolean"});
    assert!(accepts(def.clone(), Boolean(true)));
    assert!(!accepts(def, Text("true".into())));
}

/// A date is YYYY-MM-DD and respects its bounds; a full date-time is not a date.
#[test]
fn date_uses_its_own_format_and_bounds() {
    let def = json!({"dataType": "date", "restrictions": {"minDate": "2026-01-01"}});
    assert!(accepts(def.clone(), Text("2026-11-30".into())));
    assert!(!accepts(def.clone(), Text("2025-12-31".into())));
    assert!(!accepts(def, Text("2026-11-30T00:00:00Z".into())));
}

/// A time is HH:MM or HH:MM:SS.
#[test]
fn time_uses_its_own_format() {
    let def = json!({"dataType": "time"});
    assert!(accepts(def.clone(), Text("09:30".into())));
    assert!(accepts(def.clone(), Text("09:30:15".into())));
    assert!(!accepts(def, Text("9h".into())));
}

/// A date-time is RFC 3339, under its camelCase name or the old lowercase one.
#[test]
fn date_time_is_rfc3339() {
    for name in ["dateTime", "datetime"] {
        let def = json!({"dataType": name});
        assert!(accepts(def.clone(), Text("2026-11-30T10:00:00+01:00".into())), "{name}");
        assert!(!accepts(def, Text("2026-11-30".into())), "{name}");
    }
}

/// A duration is an xsd:duration with at least one part.
#[test]
fn duration_is_xsd_duration() {
    let def = json!({"dataType": "duration"});
    for ok in ["P30D", "P1M", "PT12H", "P1Y2M3DT4H5M6.5S", "-P1D"] {
        assert!(accepts(def.clone(), Text(ok.into())), "{ok}");
    }
    for bad in ["P", "PT", "30D", "P1DT", "1 month"] {
        assert!(!accepts(def.clone(), Text(bad.into())), "{bad}");
    }
}

/// A list respects its item count and allowed values.
#[test]
fn list_checks_count_and_items() {
    let def = json!({"dataType": "list", "restrictions": {"minItems": 1, "maxItems": 2, "values": ["ES", "FR", "DE"]}});
    assert!(accepts(def.clone(), List(vec!["ES".into(), "FR".into()])));
    assert!(!accepts(def.clone(), List(vec![])));
    assert!(!accepts(def.clone(), List(vec!["ES".into(), "FR".into(), "DE".into()])));
    assert!(!accepts(def, List(vec!["IT".into()])));
}

/// An IRI needs a scheme or prefix and no spaces.
#[test]
fn iri_needs_scheme_or_prefix() {
    let def = json!({"dataType": "iri"});
    assert!(accepts(def.clone(), Text("gx:LegalParticipant".into())));
    assert!(accepts(def.clone(), Text("https://w3id.org/gaia-x/ovc/1/".into())));
    assert!(!accepts(def, Text("Legal Participant".into())));
}

/// Placeholders are filled with their JSON value and `$$` becomes a literal `$`.
#[test]
fn placeholders_fill_values_and_unescape_dollars() {
    let mut content = json!({"constraint": [
        {"leftOperand": "odrl:count", "operator": "lteq", "rightOperand": "$max"},
        {"leftOperand": "odrl:payAmount", "operator": "eq", "rightOperand": "$$5"},
        {"leftOperand": "x", "operator": "isAnyOf", "rightOperand": "$regions"}
    ]});
    assert_eq!(Placeholder::collect(&content), vec!["$max", "$regions"]);
    let values = [("$max".to_string(), json!(3)), ("$regions".to_string(), json!(["ES", "FR"]))]
        .into_iter()
        .collect();
    Placeholder::fill(&mut content, &values);
    assert_eq!(content["constraint"][0]["rightOperand"], json!(3));
    assert_eq!(content["constraint"][1]["rightOperand"], json!("$5"));
    assert_eq!(content["constraint"][2]["rightOperand"], json!(["ES", "FR"]));
}
