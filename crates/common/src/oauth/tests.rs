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

//! The access rule on `UserInfo`, the two validators and the provider config, without
//! transport.

use serde_json::Map;
use ymir::errors::Errors;

use crate::oauth::{
    FixedUserValidator, ProxiedTokenValidator, RolePath, OauthTokenValidatorTrait,
    RoleTrait, UserInfo, UserTrait,
};
use crate::config::OauthConfig;

/// Unsigned JWT as oauth2-proxy would forward it from Keycloak: `sub` 0b6f-uuid, email
/// ana@upm.es, preferred_username ana, role [/admin/upm/dit, /admin/upm/dit/gsi].
const FORWARDED_JWT: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIwYjZmLXV1aWQiLCJlbWFpbCI6ImFuYUB1cG0uZXMiLCJwcmVmZXJyZWRfdXNlcm5hbWUiOiJhbmEiLCJyb2xlIjpbIi9hZG1pbi91cG0vZGl0IiwiL2FkbWluL3VwbS9kaXQvZ3NpIl0sImlzcyI6Imh0dHBzOi8va2MiLCJleHAiOjQxMDI0NDQ4MDB9.c2ln";

fn role(path: &str) -> RolePath {
    path.parse().unwrap()
}

fn user(user_id: &str, path: &str) -> UserInfo {
    UserInfo::new(user_id, None, role(path), Map::new())
}

/// A user reaches its own records and those created under a role below its own, never those
/// of a sibling or of someone above, nor those of another user sharing its role.
#[test]
fn reaches_own_and_below_but_not_peers_or_above() {
    let ana = user("ana", "/admin/upm/dit");
    assert!(ana.reaches("ana", &role("/admin/upm/dit")));
    assert!(ana.reaches("bea", &role("/admin/upm/dit/gsi")));
    assert!(ana.reaches("edu", &role("/admin/upm/dit/gsi/lab")));
    assert!(!ana.reaches("fran", &role("/admin/upm/dit")));
    assert!(!ana.reaches("dani", &role("/admin/upm")));
    assert!(!ana.reaches("eva", &role("/admin/upm/die")));
    // A sibling whose name starts like the caller's role is not below it.
    assert!(!ana.reaches("gus", &role("/admin/upm/ditx")));
    assert!(!ana.reaches("hugo", &role("/admin/acme/dit")));
}

/// The root `/admin` reaches every record, including other roots'.
#[test]
fn root_reaches_everything() {
    let root = UserInfo::system();
    assert!(root.is_root());
    assert_eq!(root.role(), &role("/admin"));
    assert!(root.reaches("anyone", &role("/admin/upm")));
    assert!(root.reaches("other-root", &RolePath::root()));
}

/// An unreachable record looks like a missing one, so probing reveals nothing.
#[test]
fn ensure_reaches_hides_unreachable_records_as_not_found() {
    let ana = user("ana", "/admin/upm/dit");
    assert!(ana.ensure_reaches("bea", &role("/admin/upm/dit/gsi"), "r1").is_ok());
    let err = ana
        .ensure_reaches("dani", &role("/admin/upm"), "r2")
        .unwrap_err();
    assert!(matches!(err, Errors::MissingResourceError { .. }), "{err:?}");
}

// /// Identities are managed only below the caller's role; the root manages any of them.
// #[test]
// fn manages_only_roles_below_and_root_manages_all() {
//     let ana = user("ana", "/admin/upm/dit");
//     assert!(ana.manages(&role("/admin/upm/dit/gsi")));
//     assert!(!ana.manages(&role("/admin/upm/dit")));
//     assert!(!ana.manages(&role("/admin/upm")));
//     assert!(ana.require_manages(&role("/admin/upm/dit")).is_err());

//     let root = UserInfo::system();
//     assert!(root.manages(&RolePath::root()));
//     assert!(root.manages(&role("/admin/upm")));
// }

/// The proxied validator reads the forwarded token: subject, email, login name and the highest
/// of its roles; without a token it is a 401.
#[tokio::test]
async fn proxied_validator_reads_the_forwarded_token() {
    let user = ProxiedTokenValidator
        .validate_token(Some(FORWARDED_JWT))
        .await
        .unwrap();
    assert_eq!(user.id(), "0b6f-uuid");
    assert_eq!(user.email(), Some("ana@upm.es"));
    assert_eq!(user.username(), Some("ana"));
    assert_eq!(user.role(), &role("/admin/upm/dit"));

    assert!(ProxiedTokenValidator.validate_token(None).await.is_err());
    assert!(ProxiedTokenValidator.validate_token(Some("not-a-jwt")).await.is_err());
}

/// The fixed validator is the configured user, token or not.
#[tokio::test]
async fn fixed_validator_ignores_the_token() {
    let validator = FixedUserValidator::new(user("dev", "/admin"));
    assert_eq!(validator.validate_token(None).await.unwrap().id(), "dev");
    assert_eq!(
        validator.validate_token(Some("anything")).await.unwrap().id(),
        "dev"
    );
}

/// The provider block: `static` carries the user itself, `keycloak` and `built_in` nothing.
#[test]
fn oauth_config_reads_each_provider() {
    let static_cfg: OauthConfig = serde_json::from_value(serde_json::json!({
        "provider": "static",
        "user_id": "dev",
        "email": "dev@local",
        "role": "/admin/upm"
    }))
    .unwrap();
    let OauthConfig::Static(user) = static_cfg else {
        panic!("expected static");
    };
    assert_eq!(user.id(), "dev");
    assert_eq!(user.role(), &role("/admin/upm"));
    assert!(user.extra().get("provider").is_none());

    let keycloak: OauthConfig =
        serde_json::from_value(serde_json::json!({ "provider": "keycloak" })).unwrap();
    assert!(matches!(keycloak, OauthConfig::Keycloak));
    let built_in: OauthConfig =
        serde_json::from_value(serde_json::json!({ "provider": "built_in" })).unwrap();
    assert!(matches!(built_in, OauthConfig::BuiltIn));
}
