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

//! CatalogExchangeRouter over the real projection service on a mocked closure service, and a
//! mocked import service: query parameters, media types, headers and status codes.

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use catalog_agent::entities::commands::ImportMode;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::http::catalog_exchange::CatalogExchangeRouter;
use catalog_agent::services::catalog_import::views::ImportReport;
use catalog_agent::services::catalog_import::MockCatalogImportServiceTrait;
use catalog_agent::services::closure::MockClosureServiceTrait;
use common::rdf::shacl::ShaclReport;
use common::rdf::RdfSyntax;
use common::test_utils::scopes::TestUsers;
use serde_json::Value;
use tower::ServiceExt;

use crate::support::builders::{id, provider_set};
use crate::support::context::TestContext;

fn closures() -> MockClosureServiceTrait {
    let mut closures = MockClosureServiceTrait::new();
    closures.expect_resource().returning(|_, id| {
        let (_, set) = provider_set();
        Ok(set.get(id.as_ref()).cloned().map(|root| (root, set)))
    });
    closures
}

async fn send(
    router: axum::Router,
    method: &str,
    uri: &str,
    headers: &[(&str, &str)],
    body: Vec<u8>,
) -> Response {
    let mut request = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let mut request = request.body(Body::from(body)).unwrap();
    request.extensions_mut().insert(TestUsers::root());
    router.oneshot(request).await.unwrap()
}

async fn text(response: Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn header<'a>(response: &'a Response, name: &str) -> &'a str {
    response
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
}

fn catalog_uri(profile: &str, query: &str) -> String {
    format!(
        "/{}/profiles/{profile}{query}",
        id(ResourceKind::Catalog, 1)
    )
}

/// A preview is JSON-LD with the profile link and how many datasets it shows of how many.
#[tokio::test]
async fn preview_has_counts_and_profile() {
    let ctx = TestContext::new().closures(closures()).build();
    let response = send(
        CatalogExchangeRouter::new(ctx).router(),
        "GET",
        &catalog_uri("dcat-ap", "?limit=1"),
        &[],
        vec![],
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(header(&response, "content-type"), "application/ld+json");
    assert_eq!(header(&response, "x-total-count"), "2");
    assert_eq!(header(&response, "x-shown-count"), "1");
    assert!(header(&response, "link").contains("rel=\"profile\""));
    assert!(header(&response, "content-disposition").is_empty());
    let body: Value = serde_json::from_str(&text(response).await).unwrap();
    assert!(body["@graph"].is_array());
}

/// A download is the whole document as an attachment, in the syntax `format` names.
#[tokio::test]
async fn download_is_an_attachment_in_turtle() {
    let ctx = TestContext::new().closures(closures()).build();
    let response = send(
        CatalogExchangeRouter::new(ctx).router(),
        "GET",
        &catalog_uri("dcat3", "?limit=1&download=true&format=turtle"),
        &[],
        vec![],
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        header(&response, "content-type"),
        "text/turtle; charset=utf-8"
    );
    assert_eq!(header(&response, "x-shown-count"), "2");
    assert!(header(&response, "content-disposition").starts_with("attachment; filename=\"catalog-"));
    assert!(text(response).await.contains("dcat:Dataset"));
}

/// Without `format`, the `Accept` header picks the syntax.
#[tokio::test]
async fn accept_picks_the_syntax() {
    let ctx = TestContext::new().closures(closures()).build();
    let response = send(
        CatalogExchangeRouter::new(ctx).router(),
        "GET",
        &catalog_uri("dsp", ""),
        &[("accept", "application/n-triples")],
        vec![],
    )
    .await;

    assert_eq!(
        header(&response, "content-type"),
        "application/n-triples; charset=utf-8"
    );
    assert!(text(response)
        .await
        .contains("<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>"));
}

fn report(dry_run: bool, imported: bool) -> ImportReport {
    ImportReport {
        profile: "dcat-ap".into(),
        dry_run,
        conforms: imported || dry_run,
        imported,
        validation: ShaclReport::new(vec!["base".into()], vec![]),
        issues: vec![],
        created: 0,
        updated: 0,
        resources: vec![],
    }
}

/// The query and the content type reach the import service; an import answers 200.
#[tokio::test]
async fn import_reads_query_and_content_type() {
    let mut imports = MockCatalogImportServiceTrait::new();
    imports
        .expect_import()
        .withf(|_, syntax, body, command| {
            *syntax == RdfSyntax::Turtle
                && body == b"<a> <b> <c> ."
                && command.profile == "dcat3"
                && command.mode == ImportMode::Create
                && command.checks == vec!["base".to_string(), "range".to_string()]
                && command.target.as_ref().map(ToString::to_string).as_deref()
                    == Some("urn:catalog:t")
                && !command.dry_run
        })
        .returning(|_, _, _, _| Ok(report(false, true)));
    let ctx = TestContext::new().imports(imports).build();

    let response = send(
        CatalogExchangeRouter::new(ctx).router(),
        "POST",
        "/import?profile=dcat3&mode=create&checks=base,%20range&target=urn:catalog:t",
        &[("content-type", "text/turtle")],
        b"<a> <b> <c> .".to_vec(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
}

/// A document that does not conform answers 422 with its report; a dry run always 200.
#[tokio::test]
async fn refused_import_is_unprocessable() {
    let mut imports = MockCatalogImportServiceTrait::new();
    imports
        .expect_import()
        .returning(|_, _, _, command| Ok(report(command.dry_run, false)));
    let ctx = TestContext::new().imports(imports).build();
    let router = CatalogExchangeRouter::new(ctx).router();

    let refused = send(
        router.clone(),
        "POST",
        "/import",
        &[("content-type", "application/ld+json")],
        b"{}".to_vec(),
    )
    .await;
    let dry = send(
        router,
        "POST",
        "/import?dryRun=true",
        &[("content-type", "application/ld+json")],
        b"{}".to_vec(),
    )
    .await;

    assert_eq!(refused.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body: Value = serde_json::from_str(&text(refused).await).unwrap();
    assert_eq!(body["validation"]["checks"][0], "base");
    assert_eq!(dry.status(), StatusCode::OK);
}

/// A body in no RDF syntax is refused before the service is asked.
#[tokio::test]
async fn unknown_content_type_is_refused() {
    let ctx = TestContext::new().build();
    let response = send(
        CatalogExchangeRouter::new(ctx).router(),
        "POST",
        "/import",
        &[("content-type", "text/csv")],
        b"a,b".to_vec(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
