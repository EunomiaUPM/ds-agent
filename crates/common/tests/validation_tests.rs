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

use common::validation::{
    codes, violation, Path, Validator, ValidatorRegistry, Violation, Violations,
};

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

fn address_validator() -> Validator<Address> {
    Validator::<Address>::new()
        .ensure_not_empty("street", |a| a.street.as_deref())
        .ensure_not_empty("country", |a| a.country.as_deref())
}

fn item_validator() -> Validator<Item> {
    Validator::<Item>::new().ensure_urn("sku", |i| i.sku.as_deref())
}

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
        address: Address {
            street: None,
            country: None,
        },
        shipping_address: None,
        items: vec![],
        is_gift: false,
        gift_note: None,
    };

    let vs = validator.validate(&invalid).unwrap_err();
    assert_eq!(vs.len(), 2);
}

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
        address: Address {
            street: None,
            country: None,
        },
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
        shipping_address: Some(Address {
            street: None,
            country: None,
        }),
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
        address: Address {
            street: None,
            country: None,
        },
        shipping_address: None,
        items: vec![],
        is_gift: false,
        gift_note: None,
    };

    let vs = merged.validate(&order).unwrap_err();
    assert_eq!(vs.len(), 2, "both stage 0 rules ran together");
}

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
        address: Address {
            street: None,
            country: None,
        },
        shipping_address: None,
        items: vec![],
        is_gift: false,
        gift_note: None,
    };

    let vs = reg_a.validate(&"order", &invalid).unwrap_err();
    assert_eq!(vs.len(), 2, "both merged registrations ran");
}
