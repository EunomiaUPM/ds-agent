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

//! Transport helpers shared by every gRPC adapter: status mapping, field parsing, paging and JSON.
//!
//! A gRPC adapter is a thin layer: it parses the proto request into domain types, calls the
//! same service the HTTP adapter calls, and maps the result back. This module covers the
//! parts every adapter repeats: [`ProtoField`] for proto3 string fields, [`ListParams`] for
//! list requests, [`PageMeta`] for list responses, [`JsonStruct`] for `google.protobuf.Struct`
//! payloads and [`IntoStatus`] for errors. Authentication lives in `crate::oauth::grpc`.
//!
//! ## 1. A handler
//!
//! ```rust,ignore
//! use common::grpc::{IntoStatus, ListParams, ProtoField};
//!
//! async fn get_dataset_by_id(
//!     &self,
//!     request: Request<GetByIdRequest>,
//! ) -> Result<Response<DatasetResponse>, Status> {
//!     let scope = self.auth.scope(request.metadata()).await?;
//!     let id = request.into_inner().id.urn("id")?;
//!     let dto = self
//!         .service
//!         .get_dataset_by_id(&scope, &id)
//!         .await
//!         .map_err(Errors::into_status)?;
//!     Ok(Response::new(dto.into()))
//! }
//! ```
//!
//! ## 2. Parsing fields
//!
//! proto3 has no null strings, so `""` means absent. [`ProtoField`] reads a `str` as required
//! or optional URN, RFC 3339 date, JSON or any `FromStr` type. Failures are `INVALID_ARGUMENT`
//! with the field name in front (`"created_after: invalid RFC3339 ..."`).
//!
//! ```rust,ignore
//! use common::grpc::{InvalidField, ProtoEnum, ProtoField, ProtoFieldList};
//!
//! let id = req.id.urn("id")?;                                    // required
//! let after = req.created_after.opt_rfc3339("created_after")?;   // "" is None
//! let author = req.author.non_empty().map(str::to_owned);
//! let state: State = req.state.proto_enum("state")?;            // i32 to prost enum
//! let ids = req.ids.urns("ids")?;                               // names "ids[3]" on failure
//! let content = req.content.ok_or_else(|| InvalidField::status("content", "is required"))?;
//! ```
//!
//! ## 3. List requests and responses
//!
//! Every list request carries `limit`, `cursor` and `sort`. Implement `TryFrom` into
//! [`ListParams`] with the resource filter; `limit == 0` and empty strings fall back to the
//! defaults of [`crate::paginated_spec`]. On the way out, [`PageMeta`] turns `Paginated`
//! into the proto fields (`""` for no next page, `0` for unknown total).
//!
//! ```rust,ignore
//! impl TryFrom<ListPolicyTemplatesRequest> for ListParams<PolicyTemplateFilter> {
//!     type Error = Status;
//!
//!     fn try_from(req: ListPolicyTemplatesRequest) -> Result<Self, Status> {
//!         let filter = PolicyTemplateFilter {
//!             tenant_id: None,
//!             id: req.id.non_empty().map(str::to_owned),
//!             version: req.version.non_empty().map(str::to_owned),
//!             author: req.author.non_empty().map(str::to_owned),
//!             created_after: req.created_after.opt_rfc3339("created_after")?,
//!             created_before: req.created_before.opt_rfc3339("created_before")?,
//!         };
//!         Self::new(filter, req.limit, &req.cursor, &req.sort)
//!     }
//! }
//!
//! let meta = PageMeta::from(&paginated);
//! let response = PolicyTemplateListResponse {
//!     items,
//!     next_cursor: meta.next_cursor,
//!     total: meta.total,
//! };
//! ```
//!
//! ## 4. JSON payloads
//!
//! Free-form JSON (ODRL content, parameters, localized text) travels as
//! `google.protobuf.Struct` or `Value`. [`JsonStruct`] converts typed values both ways, and
//! the extension traits convert raw JSON. Whole numbers come back as integers, so typed
//! integer fields survive the round trip; integers beyond `f64` precision do not.
//!
//! ```rust,ignore
//! use common::grpc::{JsonStruct, JsonStructExt};
//!
//! let content: OdrlPolicyInfo = req.content.unwrap_or_default().into_typed("content")?;
//! let proto = PolicyTemplate {
//!     content: Some(JsonStruct::from_typed(&dto.content)?),
//!     title: dto.title.map(|t| JsonStruct::value_from_typed(&t)).transpose()?,
//!     // ...
//! };
//! ```
//!
//! ## 5. Errors
//!
//! [`IntoStatus`] maps an `Errors` by its HTTP status: 404 to `NOT_FOUND`, 403 to
//! `PERMISSION_DENIED`, 401 to `UNAUTHENTICATED`, 400 and 422 to `INVALID_ARGUMENT`, 409 to
//! `ALREADY_EXISTS`, 412 to `FAILED_PRECONDITION`, 429 to `RESOURCE_EXHAUSTED`, 501 to
//! `UNIMPLEMENTED`, 502 and 503 to `UNAVAILABLE`, anything else to `INTERNAL`.

pub mod field;
pub mod json;
pub mod page;
pub mod status;

pub use field::{InvalidField, ProtoEnum, ProtoField, ProtoFieldList};
pub use json::{JsonStruct, JsonStructExt, JsonValueExt};
pub use page::{ListParams, PageMeta, PageParams};
pub use status::IntoStatus;

#[cfg(test)]
mod tests;
