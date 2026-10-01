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

//! Authentication, authorization, RBAC, and transport middleware.
//!
//! Every request carries a bearer token issued by the `oauth` crate. The transport adapters
//! validate it through an [`OauthTokenValidator`] into [`Claims`], and turn the claims plus the
//! optional `x-tenant-id` header into an [`AccessScope`]. Services take the scope as their first
//! argument and use it for every authorization decision. [`ServiceHttpClient`] covers the other
//! direction: calls from one agent to another with a service token.
//!
//! ## 1. Roles and tenants
//!
//! The token's `sub` is the caller's tenant and its role is one of [`RbacRole`]:
//!
//! | Role | Reads | Writes |
//! |---|---|---|
//! | `Admin` | every tenant, or only the one named in `x-tenant-id` | any tenant |
//! | `Owner` | its own tenant | its own tenant |
//! | `Reader` | its own tenant | none |
//!
//! A non-admin that sends an `x-tenant-id` other than its own gets a 403. An admin that sends
//! one is pinned to that tenant and sees nothing else.
//!
//! ## 2. Protecting an HTTP router
//!
//! Put [`AuthHttpMiddleware::run`] as a route layer with the validator as state. It accepts the
//! token from the `Authorization` header or a `token`/`access_token` query parameter, stores the
//! claims in the request and adds the usual security headers to the response. Handlers then
//! take an [`AccessScope`] as an extractor.
//!
//! ```rust,ignore
//! use axum::middleware::from_fn_with_state;
//! use common::auth::http::{AuthHttpMiddleware, ExtractedHeaders};
//! use common::auth::AccessScope;
//!
//! pub(crate) fn router(self) -> Router {
//!     Router::new()
//!         .route("/", get(Self::handle_list))
//!         .with_state(self.clone())
//!         .route_layer(from_fn_with_state(self.validator, AuthHttpMiddleware::run))
//! }
//!
//! async fn handle_list(
//!     State(s): State<Self>,
//!     scope: AccessScope,
//!     headers: ExtractedHeaders,
//!     Query(q): Query<ClientQuery>,
//! ) -> AppResult<(HeaderMap, Json<Paginated<ClientView>>)> {
//!     let (filter, page, sort) = q.into_domain();
//!     let result = s.client_svc.list_clients(&scope, &filter, &page, &sort).await?;
//!     // Echoes x-request-id and x-correlation-id, and adds x-total-count.
//!     Ok((headers.response_headers_paged(result.total), Json(result)))
//! }
//! ```
//!
//! [`AuthClaims`] extracts the raw claims when a handler needs them instead of a scope.
//!
//! [`AuthHttpMiddleware::run`]: http::AuthHttpMiddleware::run
//! [`AuthClaims`]: http::AuthClaims
//!
//! ## 3. Authorizing in the service layer
//!
//! The scope answers the questions a use case asks. Reads filter by `tenant_filter()`, which is
//! `None` for an unpinned admin. Creates resolve their tenant with `resolve_create_tenant`,
//! which also checks write permission. Records owned by another tenant are hidden as a 404.
//!
//! ```rust,ignore
//! async fn create(&self, scope: &AccessScope, cmd: &NewSecretCommand) -> Outcome<SecretEntry> {
//!     // Non-admins are forced into their own tenant; admins may pick one.
//!     let tenant = scope.resolve_create_tenant(cmd.tenant_id.as_deref())?;
//!     self.repo.create_secret(&tenant, cmd).await
//! }
//!
//! async fn list(&self, scope: &AccessScope, ids: &[String]) -> Outcome<Vec<Dataset>> {
//!     self.repo.get_batch_datasets(scope.tenant_filter().map(str::to_string), ids).await
//! }
//!
//! async fn get(&self, scope: &AccessScope, id: &str) -> Outcome<Grant> {
//!     let grant = self.repo.get(id).await?;
//!     scope.ensure_visible(&grant.tenant_id, id)?;
//!     Ok(grant)
//! }
//! ```
//!
//! Other checks: `require_write`, `require_admin`, `require_read_tenant`,
//! `require_write_tenant` and `resolve_query_tenant` for list filters sent by the caller.
//!
//! ## 4. gRPC
//!
//! [`GrpcAuth`] does the same from tonic metadata, returning a `Status` on failure.
//!
//! ```rust,ignore
//! use common::auth::grpc::GrpcAuth;
//!
//! let auth = GrpcAuth::new(validator);
//!
//! async fn list_policy_templates(
//!     &self,
//!     request: Request<ListPolicyTemplatesRequest>,
//! ) -> Result<Response<PolicyTemplateListResponse>, Status> {
//!     let scope = self.auth.scope(request.metadata()).await?;
//!     // ...
//! }
//! ```
//!
//! [`GrpcAuth`]: grpc::GrpcAuth
//!
//! ## 5. In-process calls
//!
//! A local facade has no request to read a token from. It builds the scope the HTTP extractor
//! would have built for the service token: `AccessScope::service(tenant)` pins it to one tenant,
//! and `AccessScope::service_cross_tenant(home)` sends no tenant and sees all of them.
//!
//! ```rust,ignore
//! let mate = self.mates.get_by_id(&AccessScope::service(&tenant_id), mate_id).await?;
//! ```
//!
//! ## 6. Calling another agent
//!
//! [`ServiceHttpClient`] gets a token through the client credentials grant, caches it until
//! shortly before it expires and drops it on a 401. Pass the tenant to act on, or `None` to
//! act as the service client itself. One instance lives in the root context.
//!
//! ```rust,ignore
//! // No tenant: the service token reads across tenants.
//! async fn get_agreement(&self, agreement_id: &Urn) -> Outcome<AgreementView> {
//!     let url = format!("{}/{agreement_id}", self.agreements_url);
//!     self.service_client.get_json(&url, None).await
//! }
//!
//! // Acting on one tenant, ignoring the response body.
//! self.service_client.post(&url, Some(tenant_id), &body).await?;
//! ```
//!
//! ## 7. Validators and rules
//!
//! [`OauthTokenValidator`] is the port the `oauth` crate implements; tests can mock it.
//! [`AuthValidators`] holds the checks the adapters run on decoded claims and tenant ids, built
//! from the atomic rules in [`AuthRules`] on top of [`crate::validation`].

pub mod access;
pub mod claims;
pub mod grpc;
pub mod http;
pub mod rules;
pub mod service_client;
pub mod token;
pub mod validators;

pub use access::{AccessScope, Rbac};
pub use claims::{Claims, RbacRole};
pub use rules::AuthRules;
pub use service_client::ServiceHttpClient;
pub use token::{OauthTokenValidator, TokenVerifier};
pub use validators::AuthValidators;

/// Header / metadata key carrying the bearer token.
pub const AUTHORIZATION_HEADER: &str = "authorization";
/// Header / metadata key selecting the tenant the caller acts on.
pub const TENANT_HEADER: &str = "x-tenant-id";
