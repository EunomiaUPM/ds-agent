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

use common::grpc::{JsonStruct, JsonStructExt, JsonValueExt};
use prost_types::value::Kind;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tonic::Code;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Payload {
    name: String,
    count: u32,
    tags: Vec<String>,
}

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

#[test]
fn non_object_json_yields_empty_struct() {
    assert!(json!([1, 2]).into_prost_struct().fields.is_empty());
    assert!(json!("s").into_prost_struct().fields.is_empty());
}

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
