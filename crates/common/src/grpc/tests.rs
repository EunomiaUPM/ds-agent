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

//! The gRPC adapter layer: proto field parsing, JSON to `prost_types::Struct`, page params and
//! the error to `Status` mapping.

use chrono::{TimeZone, Utc};
use prost_types::value::Kind;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tonic::Code;
use ymir::errors::{BadFormat, Errors};

use crate::errors::ResourceError;
use crate::grpc::{
    IntoStatus, InvalidField, JsonStruct, JsonStructExt, JsonValueExt, ListParams, PageMeta,
    PageParams, ProtoEnum, ProtoField, ProtoFieldList,
};
use crate::paginated_spec::{Paginated, Sort, DEFAULT_PAGE_LIMIT};

#[derive(Debug, PartialEq)]
enum Colour {
    Red,
    Blue,
}

impl TryFrom<i32> for Colour {
    type Error = ();
    fn try_from(v: i32) -> Result<Self, ()> {
        match v {
            0 => Ok(Self::Red),
            1 => Ok(Self::Blue),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Payload {
    name: String,
    count: u32,
    tags: Vec<String>,
}

/// An invalid field is InvalidArgument and its message starts with the field name.
#[test]
fn invalid_field_status_names_the_field() {
    let s = InvalidField::status("role", "unknown role: x");
    assert_eq!(s.code(), Code::InvalidArgument);
    assert_eq!(s.message(), "role: unknown role: x");
}

/// An empty proto string means the field is absent.
#[test]
fn non_empty_treats_empty_string_as_absent() {
    assert_eq!("".non_empty(), None);
    assert_eq!("abc".non_empty(), Some("abc"));
}

/// A URN field parses, and a bad or empty one names the field in the error.
#[test]
fn urn_parses_and_rejects_with_field_name() {
    assert_eq!("urn:ds:b".urn("id").unwrap().to_string(), "urn:ds:b");

    let err = "not-a-urn".urn("agreement_id").unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("agreement_id: invalid URN"));

    let err = "".urn("id").unwrap_err();
    assert_eq!(err.message(), "id: is required");
}

/// An optional URN maps empty to `None` and still rejects a malformed value.
#[test]
fn opt_urn_maps_empty_to_none() {
    assert!("".opt_urn("id").unwrap().is_none());
    assert!("urn:ds:b".opt_urn("id").unwrap().is_some());
    assert!("bad".opt_urn("id").is_err());
}

/// An RFC 3339 timestamp is converted to UTC; anything else is rejected.
#[test]
fn rfc3339_parses_to_utc() {
    let dt = "2026-01-02T03:04:05+02:00"
        .rfc3339("created_after")
        .unwrap();
    assert_eq!(dt, Utc.with_ymd_and_hms(2026, 1, 2, 1, 4, 5).unwrap());

    let err = "yesterday".rfc3339("created_after").unwrap_err();
    assert!(err.message().starts_with("created_after: invalid RFC3339"));
    assert!("".opt_rfc3339("created_after").unwrap().is_none());
}

/// A JSON string field parses, and invalid JSON names the field.
#[test]
fn json_parses_and_rejects() {
    assert_eq!("{\"a\":1}".json("properties").unwrap()["a"], 1);
    assert!("{"
        .json("properties")
        .unwrap_err()
        .message()
        .starts_with("properties: invalid JSON"));
    assert!("".opt_json("properties").unwrap().is_none());
}

/// `parsed` uses the target's `FromStr` and reports its error under the field name.
#[test]
fn parsed_uses_from_str_of_target() {
    assert_eq!(
        "created_at_asc".parsed::<Sort>("sort").unwrap(),
        Sort::CreatedAtAsc
    );
    assert_eq!("".opt_parsed::<Sort>("sort").unwrap(), None);
    assert_eq!("7".parsed::<u32>("limit").unwrap(), 7);

    let err = "sideways".parsed::<Sort>("sort").unwrap_err();
    assert_eq!(err.message(), "sort: unknown sort: sideways");
}

/// A list of URNs parses, and a bad entry is reported with its index.
#[test]
fn urns_parses_list_and_reports_index() {
    let ok = ["urn:ds:1".to_string(), "urn:ds:2".to_string()];
    assert_eq!(ok.urns("ids").unwrap().len(), 2);

    let bad = ["urn:ds:1".to_string(), "nope".to_string()];
    let err = bad.urns("ids").unwrap_err();
    assert!(err.message().starts_with("ids[1]: invalid URN"));
}

/// Every Sort variant survives Display and FromStr.
#[test]
fn sort_round_trips_through_display_and_from_str() {
    for s in [
        Sort::CreatedAtAsc,
        Sort::CreatedAtDesc,
        Sort::UpdatedAtAsc,
        Sort::UpdatedAtDesc,
    ] {
        assert_eq!(s.to_string().parse::<Sort>().unwrap(), s);
    }
    assert!("other".parse::<Sort>().is_err());
}

/// A proto enum decodes, and an unknown value names the field.
#[test]
fn proto_enum_decodes_and_rejects_unknown_values_naming_field() {
    assert_eq!(1.proto_enum::<Colour>("colour").unwrap(), Colour::Blue);

    let err = 7.proto_enum::<Colour>("colour").unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert_eq!(err.message(), "colour: unknown enum value 7");
}

/// Any JSON object survives the trip to a prost Struct and back.
#[test]
fn json_round_trips_through_struct() {
    let original = json!({
        "s": "x", "n": 1.5, "b": true, "z": null,
        "arr": [1, "two", false, {"k": "v"}],
        "obj": {"nested": {"deep": 3}}
    });
    let back = original.clone().into_prost_struct().into_json();
    assert_eq!(back, original);
}

/// JSON that is not an object becomes an empty Struct.
#[test]
fn non_object_json_yields_empty_struct() {
    assert!(json!([1, 2]).into_prost_struct().fields.is_empty());
    assert!(json!("s").into_prost_struct().fields.is_empty());
}

/// JSON scalars map to the matching prost kinds, and a missing kind reads as null.
#[test]
fn scalar_values_map_to_prost_kinds() {
    assert!(matches!(
        json!(null).into_prost_value().kind,
        Some(Kind::NullValue(_))
    ));
    assert!(matches!(
        json!(true).into_prost_value().kind,
        Some(Kind::BoolValue(true))
    ));
    assert!(
        matches!(json!("s").into_prost_value().kind, Some(Kind::StringValue(ref s)) if s == "s")
    );
    assert!(matches!(json!(2).into_prost_value().kind, Some(Kind::NumberValue(n)) if n == 2.0));
    assert_eq!(
        prost_types::Value { kind: None }.into_json(),
        serde_json::Value::Null
    );
}

/// A typed value round-trips through a Struct, and a shape mismatch is InvalidArgument.
#[test]
fn typed_round_trip_and_invalid_argument_on_shape_mismatch() {
    let payload = Payload {
        name: "a".into(),
        count: 3,
        tags: vec!["x".into()],
    };
    let s = JsonStruct::from_typed(&payload).unwrap();
    let back: Payload = JsonStruct::into_typed(s.clone(), "payload").unwrap();
    assert_eq!(back, payload);

    let back_ext: Payload = s.into_typed("payload").unwrap();
    assert_eq!(back_ext, payload);

    let wrong = json!({"name": "a"}).into_prost_struct();
    let err = JsonStruct::into_typed::<Payload>(wrong, "payload").unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("payload: "));
}

/// `value_from_typed` keeps lists and strings instead of forcing an object.
#[test]
fn value_from_typed_keeps_non_object_shapes() {
    let v = JsonStruct::value_from_typed(&vec!["a", "b"]).unwrap();
    match v.kind {
        Some(Kind::ListValue(l)) => assert_eq!(l.values.len(), 2),
        other => panic!("expected list, got {other:?}"),
    }
    let s = JsonStruct::value_from_typed(&"plain").unwrap();
    assert_eq!(s.kind, Some(Kind::StringValue("plain".into())));
}

/// A zero limit and empty strings mean the default page and sort.
#[test]
fn zero_limit_and_empty_strings_mean_defaults() {
    let (page, sort) = PageParams::from_proto(0, "", "").unwrap();
    assert_eq!(page.limit, DEFAULT_PAGE_LIMIT);
    assert_eq!(page.cursor, None);
    assert_eq!(sort, Sort::default());
}

/// Explicit limit, cursor and sort are passed through.
#[test]
fn explicit_values_are_passed_through() {
    let (page, sort) = PageParams::from_proto(5, "abc", "updated_at_asc").unwrap();
    assert_eq!(page.limit, 5);
    assert_eq!(page.cursor.as_deref(), Some("abc"));
    assert_eq!(sort, Sort::UpdatedAtAsc);
}

/// An unknown sort is InvalidArgument on the `sort` field.
#[test]
fn unknown_sort_is_invalid_argument() {
    let err = PageParams::from_proto(5, "", "sideways").unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert_eq!(err.message(), "sort: unknown sort: sideways");
}

/// A missing cursor or total becomes the proto default: empty string and zero.
#[test]
fn page_meta_maps_missing_cursor_and_total_to_proto_defaults() {
    let p: Paginated<u8> = Paginated::new(vec![1, 2], Some("next".into()), Some(42));
    assert_eq!(
        PageMeta::from(&p),
        PageMeta {
            next_cursor: "next".into(),
            total: 42
        }
    );

    let last: Paginated<u8> = Paginated::new(vec![3], None, None);
    assert_eq!(
        PageMeta::from(&last),
        PageMeta {
            next_cursor: String::new(),
            total: 0
        }
    );
}

/// ListParams pairs the filter with the parsed page and sort.
#[test]
fn list_params_pairs_filter_with_page_and_sort() {
    let params = ListParams::new("filter", 7, "c", "created_at_asc").unwrap();
    assert_eq!(params.filter, "filter");
    assert_eq!(params.page.limit, 7);
    assert_eq!(params.page.cursor.as_deref(), Some("c"));
    assert_eq!(params.sort, Sort::CreatedAtAsc);

    let err = ListParams::new((), 0, "", "bogus").unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
}

/// Not found maps to NotFound and keeps the reason.
#[test]
fn not_found_maps_to_not_found_with_reason() {
    let status = ResourceError::not_found("urn:x:1", "dataset").into_status();
    assert_eq!(status.code(), Code::NotFound);
    assert_eq!(status.message(), "dataset not found");
}

/// Forbidden maps to PermissionDenied.
#[test]
fn forbidden_maps_to_permission_denied() {
    assert_eq!(
        Errors::forbidden("nope", None).into_status().code(),
        Code::PermissionDenied
    );
}

/// Unauthorized maps to Unauthenticated.
#[test]
fn unauthorized_maps_to_unauthenticated() {
    assert_eq!(
        Errors::unauthorized("nope", None).into_status().code(),
        Code::Unauthenticated
    );
}

/// A bad format in what we received maps to InvalidArgument.
#[test]
fn bad_request_format_maps_to_invalid_argument() {
    assert_eq!(
        Errors::format(BadFormat::Received, "bad", None)
            .into_status()
            .code(),
        Code::InvalidArgument
    );
}

/// A bad format in what an upstream sent back maps to Unavailable.
#[test]
fn upstream_format_maps_to_unavailable() {
    assert_eq!(
        Errors::format(BadFormat::Sent, "bad", None)
            .into_status()
            .code(),
        Code::Unavailable
    );
}

/// Not implemented maps to Unimplemented.
#[test]
fn not_implemented_maps_to_unimplemented() {
    assert_eq!(
        Errors::not_impl("later", None).into_status().code(),
        Code::Unimplemented
    );
}

/// Internal errors map to Internal.
#[test]
fn internal_errors_map_to_internal() {
    assert_eq!(
        Errors::crazy("boom", None).into_status().code(),
        Code::Internal
    );
}
