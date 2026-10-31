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

//! Hosted data schemas: inferring, checking, versioning and publishing them.

pub mod service;
pub mod views;

use serde_json::Value;
use ymir::errors::Outcome;
use ymir::paginated_spec::{Page, Paginated, Sort};
use ymir::types::oauth::UserInfo;

use crate::entities::filters::SchemaFilter;
use crate::entities::schemas::{
    CheckSchemaUrlDto, InferSchemaDto, NewSchemaDto, NewSchemaVersionDto, ValidateSchemaDto,
};
use crate::services::schemas::views::{
    SchemaUrlCheckView, SchemaValidationView, SchemaVersionView, SchemaView,
};

#[mockall::automock]
#[async_trait::async_trait]
pub trait SchemaServiceTrait: Send + Sync {
    /// Page of schema versions the caller sees.
    async fn list(
        &self,
        user: &UserInfo,
        filter: &SchemaFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<SchemaView>>;
    /// Every version of a schema, newest first.
    async fn versions(&self, user: &UserInfo, id: &str) -> Outcome<Vec<SchemaView>>;
    /// One version with its content and the resources that use it.
    async fn version(&self, user: &UserInfo, id: &str, version: i32) -> Outcome<SchemaVersionView>;
    /// Checks the schema and stores it as version 1 of a new schema.
    async fn create(&self, user: &UserInfo, input: &NewSchemaDto) -> Outcome<SchemaView>;
    /// Stores the next version; earlier ones stay as they were.
    async fn add_version(
        &self,
        user: &UserInfo,
        id: &str,
        input: &NewSchemaVersionDto,
    ) -> Outcome<SchemaView>;
    /// Deletes a version, or the whole schema; a version a resource uses is a 409.
    async fn delete(&self, user: &UserInfo, id: &str, version: Option<i32>) -> Outcome<()>;
    fn infer(&self, input: &InferSchemaDto) -> Outcome<Value>;
    fn validate(&self, input: &ValidateSchemaDto) -> Outcome<SchemaValidationView>;
    /// Fetches an external URL and says whether it holds a JSON Schema.
    async fn check_url(&self, input: &CheckSchemaUrlDto) -> Outcome<SchemaUrlCheckView>;
    /// A version as anyone dereferences it: not private, `$id` set to its URL.
    async fn published(&self, id: &str, version: i32) -> Outcome<Value>;
    /// The public URL of a version.
    fn url(&self, id: &str, version: i32) -> String;
    /// Fails when `url` names a version hosted here that `user` cannot see; others pass.
    async fn check_reference(&self, user: &UserInfo, url: &str) -> Outcome<()>;
}
