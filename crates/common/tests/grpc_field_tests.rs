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

use chrono::{TimeZone, Utc};
use common::grpc::{InvalidField, ProtoField, ProtoFieldList};
use common::paginated_spec::Sort;
use tonic::Code;

#[test]
fn invalid_field_status_names_the_field() {
    let s = InvalidField::status("role", "unknown role: x");
    assert_eq!(s.code(), Code::InvalidArgument);
    assert_eq!(s.message(), "role: unknown role: x");
}

#[test]
fn non_empty_treats_empty_string_as_absent() {
    assert_eq!("".non_empty(), None);
    assert_eq!("abc".non_empty(), Some("abc"));
}

#[test]
fn urn_parses_and_rejects_with_field_name() {
    assert_eq!("urn:ds:b".urn("id").unwrap().to_string(), "urn:ds:b");

    let err = "not-a-urn".urn("agreement_id").unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("agreement_id: invalid URN"));

    let err = "".urn("id").unwrap_err();
    assert_eq!(err.message(), "id: is required");
}

#[test]
fn opt_urn_maps_empty_to_none() {
    assert!("".opt_urn("id").unwrap().is_none());
    assert!("urn:ds:b".opt_urn("id").unwrap().is_some());
    assert!("bad".opt_urn("id").is_err());
}

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

#[test]
fn urns_parses_list_and_reports_index() {
    let ok = ["urn:ds:1".to_string(), "urn:ds:2".to_string()];
    assert_eq!(ok.urns("ids").unwrap().len(), 2);

    let bad = ["urn:ds:1".to_string(), "nope".to_string()];
    let err = bad.urns("ids").unwrap_err();
    assert!(err.message().starts_with("ids[1]: invalid URN"));
}

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
