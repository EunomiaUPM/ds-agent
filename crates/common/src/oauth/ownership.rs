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

//! Who owns the agents' records and who may read or act on them.
//!
//! Every owned record keeps three columns: the user that created it (`user_id`), the role it
//! was created under (`user_role`; `role` is taken in several tables) and who else sees it
//! (`visibility`), bundled as [`Owner`]. The
//! rules are those of [`UserTrait`] and [`RoleTrait`], with one addition for records nobody
//! created:
//! - a record **a user creates** is theirs ([`Owner::private`]): it is acted on by whoever
//!   [reaches](UserTrait::reaches) it (its author, the roles above, the root);
//! - a record **a peer opens** (an incoming negotiation or transfer) has no user behind it: it
//!   is stamped with [`SYSTEM_USER_ID`] and the role of the peer's grant, and is acted on by
//!   whoever [handles](RoleTrait::handles) that role (the same role included);
//! - anyone **sees** a record it acts on, or one whose visibility is not `Private`.
//!
//! Repositories take an [`OwnerScope`] and turn it into a `WHERE` condition with
//! [`OwnerScope::condition`]; in-memory checks (a live event stream, a record already fetched)
//! use [`OwnerScope::admits`] or [`OwnershipTrait`].

use sea_orm::sea_query::Condition;
use sea_orm::ColumnTrait;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use ymir::errors::{Errors, Outcome};
use ymir::services::repo::postgres::listing::KeysetPager;
use ymir::types::participants::Visibility;

use super::{RolePath, RoleTrait, UserInfo, UserTrait, SYSTEM_USER_ID};

// ==============================================================================================
// Owner of a record
// ==============================================================================================

/// Owner of a record: who created it, under which role, and who else sees it. As the author of
/// the record it is a [`UserTrait`] too, e.g. to ask what a subscription's owner sees.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Owner {
    pub user_id: String,
    pub role: RolePath,
    pub visibility: Visibility,
}

impl Owner {
    pub fn new(user_id: impl Into<String>, role: RolePath, visibility: Visibility) -> Self {
        Self { user_id: user_id.into(), role, visibility }
    }

    /// A record `user` creates: theirs, under their role, seen by nobody else.
    pub fn private(user: &impl UserTrait) -> Self {
        Self::new(user.id(), user.role().clone(), Visibility::Private)
    }

    /// A record `user` creates with the visibility it chose (private if none).
    pub fn of(user: &impl UserTrait, visibility: Option<Visibility>) -> Self {
        Self::new(
            user.id(),
            user.role().clone(),
            visibility.unwrap_or(Visibility::Private),
        )
    }

    /// Owner of a record `user` creates asking for `requested`: the root (in-process flows that
    /// act for someone, like the dataplane engine for a transfer's owner) stamps the one it asks
    /// for; anyone else creates its own, keeping only the visibility it asked for.
    pub fn requested_by(user: &impl UserTrait, requested: Option<Owner>) -> Self {
        match requested {
            Some(owner) if user.is_root() => owner,
            other => Self::of(user, other.map(|owner| owner.visibility)),
        }
    }

    /// [`requested_by`](Self::requested_by) for create commands that carry the owner an
    /// in-process flow asks for (`owner`) and the visibility an API caller asks for.
    pub fn for_new(
        user: &impl UserTrait,
        owner: Option<Owner>,
        visibility: Option<Visibility>,
    ) -> Self {
        match owner {
            Some(owner) if user.is_root() => owner,
            other => Self::of(user, visibility.or(other.map(|owner| owner.visibility))),
        }
    }

    /// A record nobody created, handled by `role` and seen as `visibility` (what a peer opens).
    pub fn team(role: RolePath, visibility: Visibility) -> Self {
        Self::new(SYSTEM_USER_ID, role, visibility)
    }

    /// A record of the connector itself (seeders, the main catalog): the root's, public.
    pub fn connector() -> Self {
        Self::team(RolePath::root(), Visibility::Public)
    }

    /// Whether nobody created it, so it is acted on by role ([`Owner::team`]).
    pub fn is_team(&self) -> bool {
        self.user_id == SYSTEM_USER_ID
    }
}

impl RoleTrait for Owner {
    fn role(&self) -> &RolePath {
        &self.role
    }
}

impl UserTrait for Owner {
    fn id(&self) -> &str {
        &self.user_id
    }
}

/// A row with the owner columns `user_id`, `user_role` and `visibility`; implement it with
/// [`impl_owned!`](crate::impl_owned).
pub trait OwnedTrait {
    fn owner(&self) -> Owner;
}

/// Implements [`OwnedTrait`] for types with `user_id: String`, `user_role: RolePath` and
/// `visibility: Visibility` fields.
#[macro_export]
macro_rules! impl_owned {
    ($($type:ty),+ $(,)?) => {
        $(impl $crate::oauth::ownership::OwnedTrait for $type {
            fn owner(&self) -> $crate::oauth::Owner {
                $crate::oauth::Owner::new(
                    self.user_id.clone(),
                    self.user_role.clone(),
                    self.visibility.clone(),
                )
            }
        })+
    };
}

// ==============================================================================================
// Acting as a user
// ==============================================================================================

/// Role of [`acting_as`] users: any non-root path, so they never reach beyond their own records.
const ACTING_ROLE: &str = "/admin/acting";

/// An in-process actor with the identity of `user_id`, never the root: for per-user stores
/// (the keystore) that an engine reads or writes on behalf of a record's owner.
pub fn acting_as(user_id: &str) -> UserInfo {
    let role: RolePath = ACTING_ROLE.parse().expect("valid acting role path");
    UserInfo::new(user_id, None, role, Map::new())
}

// ==============================================================================================
// Rules over a record
// ==============================================================================================

/// The rules over owned records, for every [`UserTrait`].
pub trait OwnershipTrait: UserTrait {
    /// Whether it may act on (change, continue, delete) a record of `user_id` under `role`: it
    /// reaches it, or, for a record nobody created, it handles its role.
    fn acts_on(&self, user_id: &str, role: &RolePath) -> bool {
        if user_id == SYSTEM_USER_ID {
            self.handles(role)
        } else {
            self.reaches(user_id, role)
        }
    }

    /// Fails unless it [acts on](Self::acts_on) the record, which then looks like a missing one.
    fn ensure_acts_on(&self, user_id: &str, role: &RolePath, id: &str) -> Outcome<()> {
        if self.acts_on(user_id, role) {
            Ok(())
        } else {
            Err(Errors::missing_resource(id, "resource not found", None))
        }
    }

    /// Whether it sees the record: it acts on it, or its visibility is not `Private`.
    fn sees_record(&self, user_id: &str, role: &RolePath, visibility: &Visibility) -> bool {
        *visibility != Visibility::Private || self.acts_on(user_id, role)
    }

    /// [`acts_on`](Self::acts_on) over an [`Owner`].
    fn acts_on_owner(&self, owner: &Owner) -> bool {
        self.acts_on(&owner.user_id, &owner.role)
    }
}

impl<T: UserTrait + ?Sized> OwnershipTrait for T {}

// ==============================================================================================
// Scope of a query
// ==============================================================================================

/// Which owned records a query covers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnerScope {
    /// Every record: the root, and in-process flows that already know which record they want.
    All,
    /// The records of exactly one user (its keys and secrets, its own subscriptions).
    OwnedBy(String),
    /// The records a user [acts on](OwnershipTrait::acts_on).
    Acting { user_id: String, role: RolePath },
    /// The records a user [sees](OwnershipTrait::sees_record).
    Seeing { user_id: String, role: RolePath },
}

impl OwnerScope {
    /// The records `user` may act on; every record for the root.
    pub fn acting(user: &impl UserTrait) -> Self {
        if user.is_root() {
            return Self::All;
        }
        Self::Acting { user_id: user.id().to_string(), role: user.role().clone() }
    }

    /// The records `user` may read; every record for the root.
    pub fn seeing(user: &impl UserTrait) -> Self {
        if user.is_root() {
            return Self::All;
        }
        Self::Seeing { user_id: user.id().to_string(), role: user.role().clone() }
    }

    /// The records of `user_id` alone.
    pub fn owned_by(user_id: impl Into<String>) -> Self {
        Self::OwnedBy(user_id.into())
    }

    /// Whether a record of `user_id` under `role`, seen as `visibility`, is in the scope.
    pub fn admits(&self, user_id: &str, role: &RolePath, visibility: &Visibility) -> bool {
        match self {
            Self::All => true,
            Self::OwnedBy(owner) => owner == user_id,
            Self::Acting { user_id: me, role: my_role } => acts(me, my_role, user_id, role),
            Self::Seeing { user_id: me, role: my_role } => {
                *visibility != Visibility::Private || acts(me, my_role, user_id, role)
            }
        }
    }

    /// [`admits`](Self::admits) over an [`Owner`].
    pub fn admits_owner(&self, owner: &Owner) -> bool {
        self.admits(&owner.user_id, &owner.role, &owner.visibility)
    }

    /// The scope as a `WHERE` condition over a table's owner columns.
    pub fn condition<C: ColumnTrait>(
        &self,
        user_col: C,
        role_col: C,
        visibility_col: C,
    ) -> Condition {
        match self {
            Self::All => Condition::all(),
            Self::OwnedBy(owner) => Condition::all().add(user_col.eq(owner.as_str())),
            Self::Acting { user_id, role } => acting_condition(user_id, role, user_col, role_col),
            Self::Seeing { user_id, role } => Condition::any()
                .add(acting_condition(user_id, role, user_col, role_col))
                .add(visibility_col.ne(Visibility::Private)),
        }
    }
}

/// [`OwnershipTrait::acts_on`] for a non-root `me` under `my_role`.
fn acts(me: &str, my_role: &RolePath, user_id: &str, role: &RolePath) -> bool {
    if user_id == SYSTEM_USER_ID {
        role == my_role || role.is_below(my_role)
    } else {
        user_id == me || role.is_below(my_role)
    }
}

/// [`OwnershipTrait::acts_on`] for a non-root user as a condition: its own records, those under
/// a role below its own, and those nobody created under its very role.
fn acting_condition<C: ColumnTrait>(
    user_id: &str,
    role: &RolePath,
    user_col: C,
    role_col: C,
) -> Condition {
    Condition::any()
        .add(user_col.eq(user_id))
        .add(role_col.like(KeysetPager::below(role)))
        .add(
            Condition::all()
                .add(user_col.eq(SYSTEM_USER_ID))
                .add(role_col.eq(role.as_str())),
        )
}

#[cfg(test)]
mod tests {
    use serde_json::Map;
    use ymir::types::participants::Visibility;

    use super::{OwnerScope, OwnershipTrait};
    use crate::oauth::{RolePath, UserInfo, SYSTEM_USER_ID};

    fn role(path: &str) -> RolePath {
        path.parse().unwrap()
    }

    fn ana() -> UserInfo {
        UserInfo::new("ana", None, role("/admin/upm"), Map::new())
    }

    #[test]
    fn a_user_acts_on_its_own_and_below_not_on_its_peers() {
        assert!(ana().acts_on("ana", &role("/admin/upm")));
        assert!(ana().acts_on("bea", &role("/admin/upm/dit")));
        assert!(!ana().acts_on("bea", &role("/admin/upm")));
        assert!(!ana().acts_on("bea", &role("/admin/uc3m")));
    }

    #[test]
    fn records_nobody_created_are_acted_on_by_their_role() {
        assert!(ana().acts_on(SYSTEM_USER_ID, &role("/admin/upm")));
        assert!(ana().acts_on(SYSTEM_USER_ID, &role("/admin/upm/dit")));
        assert!(!ana().acts_on(SYSTEM_USER_ID, &role("/admin")));
        assert!(UserInfo::system().acts_on(SYSTEM_USER_ID, &role("/admin")));
    }

    #[test]
    fn scopes_admit_like_the_rules() {
        let acting = OwnerScope::acting(&ana());
        let seeing = OwnerScope::seeing(&ana());
        let theirs = role("/admin/uc3m");
        assert!(!acting.admits("bea", &theirs, &Visibility::Public));
        assert!(seeing.admits("bea", &theirs, &Visibility::Public));
        assert!(!seeing.admits("bea", &theirs, &Visibility::Private));
        assert!(acting.admits(SYSTEM_USER_ID, &role("/admin/upm"), &Visibility::Private));
        assert_eq!(OwnerScope::acting(&UserInfo::system()), OwnerScope::All);
        assert!(OwnerScope::owned_by("ana").admits("ana", &theirs, &Visibility::Private));
        assert!(!OwnerScope::owned_by("ana").admits("bea", &theirs, &Visibility::Public));
    }
}
