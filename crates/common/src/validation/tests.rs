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

//! Rules, staged Validators and Violations, on sample message and order
//! types.

use crate::validation::{
    codes, violation, Path, Rule, Validator, Violation, Violations,
};

struct Msg {
    pid: Option<String>,
    format: Option<String>,
}

fn pid_present(m: &Msg) -> Result<(), Violations> {
    m.pid
        .as_ref()
        .map(|_| ())
        .ok_or_else(|| violation("pid", codes::MISSING, "is required"))
}

fn format_present(m: &Msg) -> Result<(), Violations> {
    m.format
        .as_ref()
        .map(|_| ())
        .ok_or_else(|| violation("format", codes::MISSING, "is required"))
}

fn pid_is_urn(m: &Msg) -> Result<(), Violations> {
    match m.pid.as_deref() {
        Some(p) if p.starts_with("urn:") => Ok(()),
        _ => Err(violation("pid", codes::MALFORMED, "must be a URN")),
    }
}

fn staged_validator() -> Validator<Msg> {
    Validator::new()
        .rule(pid_present)
        .rule(format_present)
        .then()
        .rule(pid_is_urn)
}

#[derive(Debug)]
struct Address {
    street: Option<String>,
    country: Option<String>,
}

#[derive(Debug)]
struct Item {
    sku: Option<String>,
}

#[derive(Debug)]
struct Order {
    order_id: Option<String>,
    amount: u32,
    address: Address,
    shipping_address: Option<Address>,
    items: Vec<Item>,
    is_gift: bool,
    gift_note: Option<String>,
}

fn empty_address() -> Address {
    Address {
        street: None,
        country: None,
    }
}

fn address_validator() -> Validator<Address> {
    Validator::<Address>::new()
        .ensure_not_empty("street", |a| a.street.as_deref())
        .ensure_not_empty("country", |a| a.country.as_deref())
}

fn item_validator() -> Validator<Item> {
    Validator::<Item>::new().ensure_urn("sku", |i| i.sku.as_deref())
}

/// A plain function is a rule.
#[test]
fn a_plain_function_is_a_rule() {
    let rules: Vec<Box<dyn Rule<Msg>>> = vec![Box::new(pid_present)];
    let with_pid = Msg {
        pid: Some("x".into()),
        format: None,
    };
    assert!(rules[0].check(&with_pid).is_ok());
    assert!(rules[0]
        .check(&Msg {
            pid: None,
            format: None
        })
        .is_err());
}

/// A closure that captures data is a rule too.
#[test]
fn a_closure_carrying_data_is_a_rule_too() {
    let forbidden = "urn:uuid:bad".to_string();
    let not_forbidden = move |m: &Msg| match &m.pid {
        Some(p) if *p == forbidden => Err(violation("pid", codes::NOT_ALLOWED, "is denied")),
        _ => Ok(()),
    };
    let rules: Vec<Box<dyn Rule<Msg>>> = vec![Box::new(not_forbidden)];
    assert!(rules[0]
        .check(&Msg {
            pid: Some("urn:uuid:bad".into()),
            format: None
        })
        .is_err());
}

/// DSP renders several reasons in one error, so a stage reports all of them.
#[test]
fn a_stage_reports_every_failure_not_just_the_first() {
    let out = staged_validator().validate(&Msg {
        pid: None,
        format: None,
    });
    assert_eq!(out.unwrap_err().len(), 2);
}

/// The second stage would say "must be a URN" about a pid that is not there.
#[test]
fn a_failed_stage_stops_the_next_one() {
    let out = staged_validator().validate(&Msg {
        pid: None,
        format: Some("f".into()),
    });
    let vs = out.unwrap_err();
    assert_eq!(vs.len(), 1);
    assert_eq!(vs.code(), Some(codes::MISSING));
}

/// Later stages run once the earlier ones pass.
#[test]
fn later_stages_run_when_the_earlier_ones_pass() {
    let out = staged_validator().validate(&Msg {
        pid: Some("nope".into()),
        format: Some("f".into()),
    });
    assert_eq!(out.unwrap_err().code(), Some(codes::MALFORMED));

    let ok = staged_validator().validate(&Msg {
        pid: Some("urn:uuid:cc".into()),
        format: Some("f".into()),
    });
    assert!(ok.is_ok());
}

/// Paths render like the document they point into.
#[test]
fn nested_paths_read_like_the_document() {
    let p = Path::field("dataAddress")
        .child("endpointProperties")
        .index(0)
        .child("name");
    assert_eq!(p.as_str(), "dataAddress.endpointProperties[0].name");
}

/// Stages run in dependency order, so the first failure is the representative code.
#[test]
fn the_representative_code_is_the_first_failure() {
    let mut vs = Violations::new();
    vs.push(Violation::new("consumerPid", codes::MISSING, "is required"));
    vs.push(Violation::new("format", codes::MALFORMED, "is not a URI"));
    assert_eq!(vs.code(), Some(codes::MISSING));
    assert_eq!(vs.len(), 2);
}

/// No violations is a pass with no code.
#[test]
fn an_empty_set_is_a_pass() {
    assert!(Violations::new().into_result().is_ok());
    assert!(Violations::new().code().is_none());
}

/// The offending value is only attached on request.
#[test]
fn a_value_is_never_attached_unless_asked_for() {
    let v = Violation::new("x", codes::MISSING, "is required");
    assert!(v.value.is_none());
    assert_eq!(v.with_value("42").value.as_deref(), Some("42"));
}

/// A prefix nests every path and shows up in the reasons, not in the bare messages.
#[test]
fn path_prefixing_and_reasons() {
    let mut vs = Violations::new();
    vs.push(Violation::new(
        Path::field("street"),
        codes::MISSING,
        "is required",
    ));
    let prefixed = vs.with_prefix("address");

    assert_eq!(
        prefixed.to_reasons(),
        vec!["address.street: is required".to_string()]
    );
    assert_eq!(prefixed.messages(), vec!["is required".to_string()]);
}

/// `ensure_not_empty` rejects blank strings and `ensure` a false predicate, in one pass.
#[test]
fn fluent_ensure_and_ensure_not_empty() {
    let validator = Validator::<Order>::new()
        .ensure_not_empty("order_id", |o| o.order_id.as_deref())
        .ensure("amount", "amount must be greater than zero", |o| {
            o.amount > 0
        });

    let invalid = Order {
        order_id: Some("   ".to_string()),
        amount: 0,
        address: empty_address(),
        shipping_address: None,
        items: vec![],
        is_gift: false,
        gift_note: None,
    };

    let vs = validator.validate(&invalid).unwrap_err();
    assert_eq!(vs.len(), 2);
}

/// A `when` rule only runs if its condition holds.
#[test]
fn conditional_when_rule() {
    let validator = Validator::<Order>::new().when(
        |o| o.is_gift,
        |o: &Order| {
            if o.gift_note.as_deref().unwrap_or("").is_empty() {
                Err(violation(
                    "gift_note",
                    codes::MISSING,
                    "gift note is required for gifts",
                ))
            } else {
                Ok(())
            }
        },
    );

    let non_gift = Order {
        order_id: Some("ord-1".into()),
        amount: 10,
        address: empty_address(),
        shipping_address: None,
        items: vec![],
        is_gift: false,
        gift_note: None,
    };
    assert!(validator.validate(&non_gift).is_ok());

    let gift_without_note = Order {
        is_gift: true,
        ..non_gift
    };
    assert!(validator.validate(&gift_without_note).is_err());
}

/// `field`, `field_opt` and `each` validate nested values and prefix their paths.
#[test]
fn nested_field_and_each_combinators() {
    let validator = Validator::<Order>::new()
        .field("address", |o| &o.address, address_validator())
        .field_opt(
            "shipping",
            |o| o.shipping_address.as_ref(),
            address_validator(),
        )
        .each("items", |o| &o.items, item_validator());

    let order = Order {
        order_id: Some("1".into()),
        amount: 10,
        address: Address {
            street: None,
            country: Some("ES".into()),
        },
        shipping_address: Some(empty_address()),
        items: vec![
            Item {
                sku: Some("urn:sku:123".into()),
            },
            Item {
                sku: Some("invalid-sku".into()),
            },
        ],
        is_gift: false,
        gift_note: None,
    };

    let vs = validator.validate(&order).unwrap_err();
    let reasons = vs.to_reasons();

    assert!(reasons.contains(&"address.street: field is required".to_string()));
    assert!(reasons.contains(&"shipping.street: field is required".to_string()));
    assert!(reasons.contains(&"shipping.country: field is required".to_string()));
    assert!(reasons.contains(&"items[1].sku: must be a valid URN".to_string()));
}

/// Merging two validators runs their first stages together.
#[test]
fn validator_merge_combines_stages() {
    let val_a = Validator::<Order>::new()
        .ensure_not_empty("order_id", |o| o.order_id.as_deref())
        .then()
        .ensure("amount", "must be positive", |o| o.amount > 0);

    let val_b = Validator::<Order>::new().ensure("is_gift", "must be a gift", |o| o.is_gift);

    let merged = val_a.merge(val_b);

    let order = Order {
        order_id: None,
        amount: 0,
        address: empty_address(),
        shipping_address: None,
        items: vec![],
        is_gift: false,
        gift_note: None,
    };

    let vs = merged.validate(&order).unwrap_err();
    assert_eq!(vs.len(), 2, "both stage 0 rules ran together");
}

mod staged_registry {
    //! ValidationRegistry: global stages across origins, fail-closed keys and attribution.

    use ymir::registry::{Origin, PluginRef};
    use crate::validation::{codes, violation, Stage, ValidationRegistry, Validator};

    fn plugin() -> Origin {
        Origin::Plugin(PluginRef {
            id: "p".into(),
            version: "1".into(),
            digest: "d".into(),
        })
    }

    fn fails(path: &'static str) -> Validator<u32> {
        Validator::new().rule(move |_: &u32| Err(violation(path, codes::NOT_ALLOWED, "no")))
    }

    fn passes() -> Validator<u32> {
        Validator::new().rule(|_: &u32| Ok(()))
    }

    /// A failing stage stops later stages, whoever registered them.
    #[test]
    fn a_failing_stage_stops_later_stages() {
        let reg = ValidationRegistry::new("t")
            .with("k", Stage::Shape, fails("shape"))
            .with("k", Stage::Domain, fails("domain"));
        reg.register("k", Stage::Profile, fails("profile"), plugin());
        let err = reg.validate(&"k", &0).unwrap_err();
        assert_eq!(err.to_reasons(), vec!["shape: no"]);
    }

    /// Within a stage every origin's failures are reported together, each with its origin.
    #[test]
    fn one_stage_accumulates_every_origin() {
        let reg = ValidationRegistry::new("t").with("k", Stage::Domain, fails("core"));
        reg.register("k", Stage::Domain, fails("plugin"), plugin());
        let err = reg.validate(&"k", &0).unwrap_err();
        let origins: Vec<_> = err.iter().map(|v| v.origin.clone()).collect();
        assert_eq!(err.len(), 2);
        assert_eq!(origins, vec![Some(Origin::Core), Some(plugin())]);
    }

    /// A key with only plugin or wildcard rules is rejected: core must register it.
    #[test]
    fn a_key_without_core_rules_fails_closed() {
        let reg = ValidationRegistry::new("t").with_any(Stage::Shape, passes());
        reg.register("k", Stage::Shape, passes(), plugin());
        let err = reg.validate(&"k", &0).unwrap_err();
        assert_eq!(err.code(), Some(codes::NOT_ALLOWED));
    }

    /// Wildcard rules apply to every registered key.
    #[test]
    fn wildcard_rules_apply_to_every_key() {
        let reg = ValidationRegistry::new("t")
            .with_any(Stage::Shape, fails("@context"))
            .with("a", Stage::Domain, passes())
            .with("b", Stage::Domain, passes());
        assert!(reg.validate(&"a", &0).is_err());
        assert!(reg.validate(&"b", &0).is_err());
    }

    /// Uninstalling a plugin removes its rules and leaves the core ones.
    #[test]
    fn unregistering_an_origin_removes_its_rules() {
        let reg = ValidationRegistry::new("t").with("k", Stage::Shape, passes());
        reg.register("k", Stage::Shape, fails("plugin"), plugin());
        assert!(reg.validate(&"k", &0).is_err());
        assert_eq!(reg.unregister_origin(&plugin()), 1);
        assert!(reg.validate(&"k", &0).is_ok());
    }

    /// Violations become one structured 400 error with a reason per failure.
    #[test]
    fn violations_become_structured_errors() {
        let reg = ValidationRegistry::new("t")
            .with("k", Stage::Shape, fails("a"))
            .with("k", Stage::Shape, fails("b"));
        let err = reg.validate(&"k", &0).unwrap_err().into_errors();
        let paths: Vec<_> = err.info().reasons.iter().map(|r| r.path.as_str()).collect();
        assert_eq!(paths, vec!["a", "b"]);
    }
}

mod json_schema {
    //! SchemaRule over the DSP 2025-1 negotiation schemas, resolved offline.

    use crate::validation::{Rule, SchemaRule, Stage, ValidationRegistry, Validator};
    use serde_json::{json, Value};

    const REQUEST: &str =
        include_str!("../../../negotiation-agent/dsp_schema/contract-request-message-schema.json");
    const CONTRACT: &str =
        include_str!("../../../negotiation-agent/dsp_schema/contract-schema.json");
    const DATASET: &str = include_str!("../../../catalog-agent/dsp_schema/dataset-schema.json");
    const CONTEXT: &str = include_str!("../dsp_common/common/context-schema.json");
    const EXAMPLE: &str =
        include_str!("../../../negotiation-agent/dsp_schema/example/contract-request-message.json");

    fn request_rule() -> SchemaRule {
        SchemaRule::with_resources(
            REQUEST,
            &[
                (
                    "https://w3id.org/dspace/2025/1/negotiation/contract-schema.json",
                    CONTRACT,
                ),
                (
                    "https://w3id.org/dspace/2025/1/catalog/dataset-schema.json",
                    DATASET,
                ),
                (
                    "https://w3id.org/dspace/2025/1/common/context-schema.json",
                    CONTEXT,
                ),
            ],
        )
        .unwrap()
    }

    /// The published example passes the published schema.
    #[test]
    fn official_example_is_valid() {
        let example: Value = serde_json::from_str(EXAMPLE).unwrap();
        assert!(request_rule().check(&example).is_ok());
    }

    /// Every schema failure is reported, each at the place it happened.
    #[test]
    fn broken_message_reports_each_failure() {
        let mut broken: Value = serde_json::from_str(EXAMPLE).unwrap();
        broken["@type"] = json!("ContractOfferMessage");
        broken["consumerPid"] = json!(42);
        let err = request_rule().check(&broken).unwrap_err();
        assert!(err.len() >= 2, "{err}");
        assert!(err.iter().any(|v| v.path.as_str() == "/consumerPid"));
        assert!(err.iter().any(|v| v.path.as_str() == "/@type"));
    }

    /// A reference that was not provided fails when the rule is built, never at validation.
    #[test]
    fn missing_reference_fails_at_build_time() {
        assert!(SchemaRule::new(REQUEST).is_err());
    }

    /// A schema rule plugs into a validator over a JSON field of the subject.
    #[test]
    fn schema_rule_runs_inside_a_registry() {
        struct Input {
            raw: Value,
        }
        let reg = ValidationRegistry::new("t").with(
            (),
            Stage::Shape,
            Validator::new().json(|i: &Input| &i.raw, request_rule()),
        );
        assert!(reg
            .validate(
                &(),
                &Input {
                    raw: json!({"@type": "Nope"})
                }
            )
            .is_err());
    }
}

mod state_machine {
    //! StateMachine: which messages a two-state table accepts.

    use std::fmt;

    use crate::validation::{codes, StateMachine};

    #[derive(Debug, PartialEq)]
    enum State {
        Open,
        Closed,
    }

    #[derive(Debug, PartialEq)]
    enum Message {
        Open,
        Close,
        Ping,
    }

    impl fmt::Display for State {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Debug::fmt(self, f)
        }
    }

    impl fmt::Display for Message {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Debug::fmt(self, f)
        }
    }

    const MACHINE: StateMachine<State, Message> = StateMachine::new(&[
        (Message::Open, &[None, Some(State::Closed)]),
        (Message::Close, &[Some(State::Open)]),
    ]);

    /// A message listed for the current state passes, the not-yet-opened state included.
    #[test]
    fn listed_transitions_pass() {
        assert!(MACHINE.check(None, &Message::Open, "state").is_ok());
        assert!(MACHINE
            .check(Some(&State::Closed), &Message::Open, "state")
            .is_ok());
        assert!(MACHINE
            .check(Some(&State::Open), &Message::Close, "state")
            .is_ok());
    }

    /// A listed message in a state it does not accept is refused, naming the state.
    #[test]
    fn unlisted_state_is_refused() {
        let err = MACHINE
            .check(Some(&State::Open), &Message::Open, "state")
            .unwrap_err();
        assert_eq!(err.code(), Some(codes::NOT_ALLOWED));
        assert!(err.to_string().contains("from state Open"));
        let err = MACHINE.check(None, &Message::Close, "state").unwrap_err();
        assert!(err.to_string().contains("from state <new>"));
    }

    /// A message missing from the table is not a lifecycle transition at all.
    #[test]
    fn unknown_message_is_refused() {
        let err = MACHINE
            .check(Some(&State::Open), &Message::Ping, "state")
            .unwrap_err();
        assert!(err.to_string().contains("not a lifecycle transition"));
    }
}
