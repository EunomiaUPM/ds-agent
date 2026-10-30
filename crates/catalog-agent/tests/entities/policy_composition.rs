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

//! CompositionResolver and OfferComposer over an in-memory library: references by position,
//! checks before storing, flattening, and offers added up from parts.

use std::collections::HashMap;
use std::str::FromStr;

use catalog_agent::entities::policy_composition::composer::OfferComposer;
use catalog_agent::entities::policy_composition::resolver::CompositionResolver;
use catalog_agent::entities::policy_composition::{OfferPart, Slot, TemplateLibrary};
use catalog_agent::entities::policy_templates::{ControlPoint, PolicyTemplateDto, PolicyTemplateKind};
use serde_json::{json, Value};
use urn::Urn;

use crate::support::builders::piece;

/// `expires-after`, `credential-type`, `no-redistribution`, `use` and `members-only`.
fn library() -> TemplateLibrary {
    TemplateLibrary::new([
        piece(
            "expires-after",
            "constraint",
            json!({"leftOperand": "odrl:elapsedTime", "operator": "lteq", "rightOperand": "$duration"}),
            json!({"$duration": {"dataType": "duration", "defaultValue": "P30D"}}),
            json!(["transfer", "validity"]),
        ),
        piece(
            "credential-type",
            "constraint",
            json!({"leftOperand": "ovc:credentialSubjectType", "operator": "eq", "rightOperand": "$type"}),
            json!({"$type": {"dataType": "iri"}}),
            json!(["negotiation"]),
        ),
        piece(
            "no-redistribution",
            "rule",
            json!({"prohibition": {"action": "distribute"}}),
            json!({}),
            json!(["duties"]),
        ),
        piece(
            "use",
            "rule",
            json!({"permission": {"action": "use"}}),
            json!({}),
            json!(["transfer"]),
        ),
        piece(
            "members-only",
            "policy",
            json!({"permission": [{"action": "use", "constraint": [
                {"@ref": "credential-type@1", "with": {"$type": "$member"}}
            ]}]}),
            json!({"$member": {"dataType": "iri", "defaultValue": "gx:LegalParticipant"}}),
            json!([]),
        ),
    ])
}

fn get(library: &TemplateLibrary, name: &str) -> PolicyTemplateDto {
    let (id, version) = name.split_once('@').unwrap();
    library
        .get(&catalog_agent::entities::policy_composition::TemplateRef {
            id: id.into(),
            version: version.into(),
            with: HashMap::new(),
        })
        .unwrap()
        .clone()
}

fn values(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

/// References are listed with the slot they sit in: a constraint, a rule list, an include.
#[test]
fn references_are_listed_by_slot() {
    let content = json!({
        "permission": [{"@ref": "use@1"}, {"action": "read", "constraint": [{"and": [{"@ref": "expires-after@1"}]}]}],
        "@include": [{"@ref": "members-only@1"}]
    });
    let found = CompositionResolver::references(PolicyTemplateKind::Policy, &content).unwrap();
    let slots: Vec<(String, Slot)> = found.into_iter().map(|(r, s)| (r.name(), s)).collect();
    assert_eq!(
        slots,
        vec![
            ("members-only@1".into(), Slot::Include),
            ("use@1".into(), Slot::Rule("permission")),
            ("expires-after@1".into(), Slot::Constraint),
        ]
    );
}

/// A reference where no template can go, or an `@include` inside a rule, is refused.
#[test]
fn misplaced_references_are_refused() {
    let in_operand = json!({"permission": [{"action": "use", "constraint": [
        {"leftOperand": "x", "operator": "eq", "rightOperand": {"@ref": "use@1"}}
    ]}]});
    let include_in_rule = json!({"permission": [{"action": "use", "@include": [{"@ref": "use@1"}]}]});
    let no_version = json!({"permission": [{"@ref": "use"}]});
    for content in [in_operand, include_in_rule, no_version] {
        assert!(CompositionResolver::references(PolicyTemplateKind::Policy, &content).is_err(), "{content}");
    }
}

/// A rule's content holds exactly one rule list.
#[test]
fn rule_content_has_one_list() {
    let two = json!({"permission": {"action": "use"}, "prohibition": {"action": "sell"}});
    assert!(CompositionResolver::references(PolicyTemplateKind::Rule, &two).is_err());
}

/// A policy with an include and references flattens to the ODRL written by hand.
#[test]
fn composed_policy_flattens_like_handwritten_odrl() {
    let library = library();
    let template = piece(
        "research",
        "policy",
        json!({
            "permission": [{"action": "use", "constraint": [
                {"leftOperand": "odrl:purpose", "operator": "eq", "rightOperand": "research"},
                {"@ref": "expires-after@1", "with": {"$duration": "$days"}}
            ]}],
            "prohibition": [{"@ref": "no-redistribution@1"}],
            "@include": [{"@ref": "members-only@1"}]
        }),
        json!({"$days": {"dataType": "duration"}}),
        json!([]),
    );
    let values = template.resolve_values(&values(&[("$days", json!("P7D"))])).unwrap();
    let flat = CompositionResolver::new(&library).expand(&template, &values).unwrap();
    assert_eq!(
        flat,
        json!({
            "permission": [
                {"action": "use", "constraint": [
                    {"leftOperand": "odrl:purpose", "operator": "eq", "rightOperand": "research"},
                    {"leftOperand": "odrl:elapsedTime", "operator": "lteq", "rightOperand": "P7D"}
                ]},
                {"action": "use", "constraint": [
                    {"leftOperand": "ovc:credentialSubjectType", "operator": "eq", "rightOperand": "gx:LegalParticipant"}
                ]}
            ],
            "prohibition": [{"action": "distribute"}]
        })
    );
}

/// A piece of the wrong kind for its slot, or a rule from another list, is refused.
#[test]
fn piece_of_the_wrong_kind_is_refused() {
    let library = library();
    let resolver = CompositionResolver::new(&library);
    let constraint_as_rule = json!({"permission": [{"@ref": "expires-after@1"}]});
    let prohibition_as_permission = json!({"permission": [{"@ref": "no-redistribution@1"}]});
    for content in [constraint_as_rule, prohibition_as_permission] {
        assert!(resolver.check(PolicyTemplateKind::Policy, &content, &HashMap::new()).is_err(), "{content}");
    }
}

/// Before storing: required parameters must be bound, only existing ones, and only to declared
/// parameters of the parent.
#[test]
fn bindings_are_checked_before_storing() {
    let library = library();
    let resolver = CompositionResolver::new(&library);
    let declared: HashMap<_, _> = serde_json::from_value(json!({"$t": {"dataType": "iri"}})).unwrap();
    let unbound = json!({"leftOperand": "a", "operator": "eq", "rightOperand": "b", "and": [{"@ref": "credential-type@1"}]});
    let unknown = json!({"and": [{"@ref": "credential-type@1", "with": {"$type": "x:y", "$other": 1}}]});
    let undeclared = json!({"and": [{"@ref": "credential-type@1", "with": {"$type": "$nope"}}]});
    let fine = json!({"and": [{"@ref": "credential-type@1", "with": {"$type": "$t"}}]});
    for content in [unbound, unknown, undeclared] {
        assert!(resolver.check(PolicyTemplateKind::Constraint, &content, &declared).is_err(), "{content}");
    }
    assert!(resolver.check(PolicyTemplateKind::Constraint, &fine, &declared).is_ok());
}

/// A value bound to a piece is checked against the piece's own parameter type.
#[test]
fn bound_values_follow_the_piece_types() {
    let library = library();
    let template = piece(
        "bad",
        "policy",
        json!({"permission": [{"action": "use", "constraint": [{"@ref": "expires-after@1", "with": {"$duration": "one month"}}]}]}),
        json!({}),
        json!([]),
    );
    assert!(CompositionResolver::new(&library).expand(&template, &HashMap::new()).is_err());
}

/// A cycle between pieces is reported, not followed forever.
#[test]
fn cycles_are_reported() {
    let library = TemplateLibrary::new([
        piece("a", "constraint", json!({"and": [{"@ref": "b@1"}]}), json!({}), json!([])),
        piece("b", "constraint", json!({"or": [{"@ref": "a@1"}]}), json!({}), json!([])),
    ]);
    let a = get(&library, "a@1");
    assert!(CompositionResolver::new(&library).expand(&a, &HashMap::new()).is_err());
}

/// Control points add up over every piece below a template.
#[test]
fn control_points_add_up() {
    let library = library();
    let points: Vec<ControlPoint> = library.control_points(&get(&library, "members-only@1")).into_iter().collect();
    assert_eq!(points, vec![ControlPoint::Negotiation]);
}

fn parts(values: Vec<Value>) -> Vec<OfferPart> {
    values.iter().enumerate().map(|(i, v)| OfferPart::parse(v, &format!("parts[{i}]")).unwrap()).collect()
}

/// An offer adds up a policy template, a rule, a constraint joining every permission, another
/// offer and hand-written ODRL.
#[test]
fn offer_parts_add_up() {
    let library = library();
    let other = Urn::from_str("urn:odrl-policy:00000000-0000-0000-0000-000000000009").unwrap();
    let offers = HashMap::from([(other.clone(), json!({"obligation": [{"action": "delete"}]}))]);
    let composer = OfferComposer::new(&library, &offers);
    let parts = parts(vec![
        json!({"@ref": "members-only@1", "with": {"$member": "gx:Member"}}),
        json!({"@ref": "expires-after@1", "with": {"$duration": "P1M"}, "into": "permission"}),
        json!({"@ref": "no-redistribution@1"}),
        json!({"@offer": other.to_string()}),
        json!({"permission": [{"action": "read"}]}),
    ]);
    let expiry = json!({"leftOperand": "odrl:elapsedTime", "operator": "lteq", "rightOperand": "P1M"});
    assert_eq!(
        composer.compose(&parts).unwrap(),
        json!({
            "permission": [
                {"action": "use", "constraint": [
                    {"leftOperand": "ovc:credentialSubjectType", "operator": "eq", "rightOperand": "gx:Member"},
                    expiry
                ]},
                {"action": "read", "constraint": [expiry]}
            ],
            "prohibition": [{"action": "distribute"}],
            "obligation": [{"action": "delete"}]
        })
    );
    let points: Vec<ControlPoint> = composer.control_points(&parts).unwrap().into_iter().collect();
    assert_eq!(
        points,
        vec![ControlPoint::Negotiation, ControlPoint::Transfer, ControlPoint::Validity, ControlPoint::Duties]
    );
}

/// A constraint part needs `into` and rules to join; `into` is only for constraints.
#[test]
fn constraint_parts_need_into_and_rules() {
    let library = library();
    let offers = HashMap::new();
    let composer = OfferComposer::new(&library, &offers);
    let without_into = parts(vec![json!({"@ref": "use@1"}), json!({"@ref": "expires-after@1"})]);
    let into_a_rule = parts(vec![json!({"@ref": "use@1", "into": "permission"})]);
    let nothing_to_join = parts(vec![json!({"@ref": "expires-after@1", "into": "prohibition"}), json!({"@ref": "use@1"})]);
    for parts in [without_into, into_a_rule, nothing_to_join] {
        assert!(composer.compose(&parts).is_err(), "{parts:?}");
    }
}

/// An offer part the caller cannot see fails like a missing one.
#[test]
fn unseen_offer_part_is_refused() {
    let library = library();
    let offers = HashMap::new();
    let parts = parts(vec![json!({"@offer": "urn:odrl-policy:00000000-0000-0000-0000-000000000001"})]);
    assert!(OfferComposer::new(&library, &offers).compose(&parts).is_err());
}

/// The library the connector seeds, read from `static/policy_templates` as stored versions.
fn seeded_library() -> Vec<PolicyTemplateDto> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../static/policy_templates");
    let mut templates = Vec::new();
    for entry in std::fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        let draft: catalog_agent::entities::policy_templates::NewPolicyTemplateDto =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        draft.validate_dto().unwrap_or_else(|e| panic!("{path:?}: {e}"));
        let mut stored = piece("x", draft.kind.as_str(), draft.stored_content().unwrap(), json!({}), json!([]));
        stored.id = draft.id.clone().unwrap();
        stored.version = draft.version.clone().unwrap();
        stored.parameters = draft.parameters.clone();
        stored.control_points = draft.control_points.clone();
        templates.push(stored);
    }
    templates
}

/// Every seeded piece is valid and references only seeded pieces with the bindings they need.
#[test]
fn seeded_library_is_consistent() {
    let templates = seeded_library();
    let library = TemplateLibrary::new(templates.clone());
    let resolver = CompositionResolver::new(&library);
    assert_eq!(templates.len(), 13);
    for template in &templates {
        resolver
            .check(template.kind, &template.content, &template.parameters)
            .unwrap_or_else(|e| panic!("{}: {e}", template.reference()));
    }
}

/// The seeded research policy flattens with its defaults to purpose, expiry and no redistribution.
#[test]
fn seeded_research_policy_flattens() {
    let library = TemplateLibrary::new(seeded_library());
    let research = get(&library, "research-30-days@1.0");
    let values = research.resolve_values(&HashMap::new()).unwrap();
    assert_eq!(
        CompositionResolver::new(&library).expand(&research, &values).unwrap(),
        json!({
            "permission": [{"action": "use", "constraint": [
                {"leftOperand": "odrl:purpose", "operator": "eq", "rightOperand": "research"},
                {"leftOperand": "odrl:elapsedTime", "operator": "lteq", "rightOperand": "P30D"}
            ]}],
            "prohibition": [{"action": "distribute"}]
        })
    );
    let points: Vec<ControlPoint> = library.control_points(&research).into_iter().collect();
    assert_eq!(
        points,
        vec![ControlPoint::Negotiation, ControlPoint::Transfer, ControlPoint::Validity, ControlPoint::Duties]
    );
}
