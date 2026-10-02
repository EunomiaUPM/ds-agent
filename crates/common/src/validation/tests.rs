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

//! Rules, staged Validators, ValidatorRegistry and Violations, on sample message and order
//! types.

use crate::validation::{
    codes, violation, Path, Rule, Validator, ValidatorRegistry, Violation, Violations,
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

/// Registering the same key twice runs both validators.
#[test]
fn registering_twice_composes_instead_of_replacing() {
    let mut reg: ValidatorRegistry<&str, Msg> = ValidatorRegistry::new();
    reg.register("Request", Validator::new().rule(pid_present));
    reg.register("Request", Validator::new().rule(format_present));
    let vs = reg
        .validate(
            &"Request",
            &Msg {
                pid: None,
                format: None,
            },
        )
        .unwrap_err();
    assert_eq!(vs.len(), 2, "both registrations ran");
}

/// A key without a registered validator is rejected.
#[test]
fn an_unregistered_key_is_rejected() {
    let reg: ValidatorRegistry<&str, Msg> = ValidatorRegistry::new();
    let subject = Msg {
        pid: Some("urn:uuid:cc".into()),
        format: Some("f".into()),
    };
    assert!(reg.validate(&"Unknown", &subject).is_err());
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

/// Merging registries composes the validators registered under the same key.
#[test]
fn registry_merge_combines_message_validators() {
    let mut reg_a: ValidatorRegistry<&str, Order> = ValidatorRegistry::new();
    reg_a.register(
        "order",
        Validator::<Order>::new().ensure_not_empty("order_id", |o| o.order_id.as_deref()),
    );

    let mut reg_b: ValidatorRegistry<&str, Order> = ValidatorRegistry::new();
    reg_b.register(
        "order",
        Validator::<Order>::new().ensure("amount", "positive", |o| o.amount > 0),
    );

    reg_a.merge(reg_b);
    assert!(reg_a.has_key(&"order"));

    let invalid = Order {
        order_id: None,
        amount: 0,
        address: empty_address(),
        shipping_address: None,
        items: vec![],
        is_gift: false,
        gift_note: None,
    };

    let vs = reg_a.validate(&"order", &invalid).unwrap_err();
    assert_eq!(vs.len(), 2, "both merged registrations ran");
}
