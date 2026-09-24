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

//! Context document loader supporting local assets, in-memory caching, and remote fallback.

use axum::http::header::ACCEPT;
use axum::http::{HeaderMap, HeaderValue};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use ymir::services::client::ClientTrait;
use ymir::utils::http_client;

use json_syntax::{Parse, Value as JsonSyntaxValue};
use locspan::{Location, Meta, Span};
use sophia_iri::Iri;
use sophia_jsonld::json_ld::future::{BoxFuture, FutureExt};
use sophia_jsonld::json_ld::{self, Loader, LoadingResult, RemoteDocument};
use sophia_jsonld::vocabulary::ArcIri;
use tokio::sync::RwLock;

pub type JsonVal = JsonSyntaxValue<Location<ArcIri, Span>>;
pub type MetaVal = Meta<JsonVal, Location<ArcIri, Span>>;

/// Errors that may occur when resolving and loading JSON-LD context documents.
#[derive(thiserror::Error, Debug)]
pub enum RdfLoaderError {
    #[error("Document not found: {0}")]
    NotFound(String),
    #[error("Failed to fetch remote document '{0}': {1}")]
    Network(String, String),
    #[error("Failed to parse JSON-LD document from '{0}': {1}")]
    Parse(String, String),
}

/// A composite JSON-LD loader with local embedded assets, memory caching, and HTTP fallback.
#[derive(Clone, Debug)]
pub struct RdfContextLoader {
    assets: Arc<HashMap<String, MetaVal>>,
    cache: Arc<RwLock<HashMap<String, MetaVal>>>,
}

impl Default for RdfContextLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl RdfContextLoader {
    /// Creates an empty loader with in-memory caching and HTTP remote fallback.
    pub fn new() -> Self {
        Self {
            assets: Arc::new(HashMap::new()),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a raw JSON-LD context document on an owned loader (builder style).
    pub fn with_asset(mut self, url: &str, json_doc: &str) -> Result<Self, RdfLoaderError> {
        self.register_asset(url, json_doc)?;
        Ok(self)
    }

    /// Registers a raw JSON-LD context document under a specific canonical URL or filename.
    pub fn register_asset(&mut self, url: &str, json_doc: &str) -> Result<(), RdfLoaderError> {
        let iri_str = if url.contains("://") {
            url.to_string()
        } else {
            format!("file://local/{url}")
        };
        let parsed = Self::parse_doc(&iri_str, json_doc)?;
        let mut map = (*self.assets).clone();
        map.insert(url.to_string(), parsed.clone());
        if let Some(fname) = url.rsplit('/').next() {
            if fname != url {
                map.insert(fname.to_string(), parsed);
            }
        }
        self.assets = Arc::new(map);
        Ok(())
    }

    /// Preloads any `.jsonld` context files found in the specified filesystem directory.
    pub fn preload_dir<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<(), RdfLoaderError> {
        let path = dir_path.as_ref();
        if !path.exists() || !path.is_dir() {
            return Ok(());
        }

        let mut map = (*self.assets).clone();
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.extension().is_some_and(|ext| ext == "jsonld") {
                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                        let file_name = file_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or_default()
                            .to_string();
                        let fallback_url = format!("file://local/{}", file_name);
                        if let Ok(parsed) = Self::parse_doc(&fallback_url, &content) {
                            map.insert(file_name, parsed.clone());
                            map.insert(fallback_url, parsed);
                        }
                    }
                }
            }
        }
        self.assets = Arc::new(map);
        Ok(())
    }

    /// Checks if a context URL is currently held in the in-memory cache or assets.
    pub async fn is_cached(&self, url: &str) -> bool {
        if self.assets.contains_key(url) {
            return true;
        }
        if let Some(fname) = url.rsplit('/').next() {
            if self.assets.contains_key(fname) {
                return true;
            }
        }
        self.cache.read().await.contains_key(url)
    }

    /// Parses string content into a typed JSON syntax tree bound to the URL location.
    pub fn parse_doc(url_str: &str, doc_str: &str) -> Result<MetaVal, RdfLoaderError> {
        let iri: ArcIri = Iri::new_unchecked(Arc::from(url_str));
        JsonSyntaxValue::parse_str(doc_str, |span| Location::new(iri.clone(), span))
            .map_err(|e| RdfLoaderError::Parse(url_str.to_string(), e.to_string()))
    }

    /// Constructs a RemoteDocument wrapping the parsed JSON-LD syntax tree.
    fn to_remote_document(
        url: ArcIri,
        parsed: MetaVal,
    ) -> RemoteDocument<ArcIri, Location<ArcIri, Span>, JsonVal> {
        RemoteDocument::new(
            Some(url),
            Some("application/ld+json".parse().unwrap()),
            parsed,
        )
    }
}

impl Loader<ArcIri, Location<ArcIri, Span>> for RdfContextLoader {
    type Output = JsonVal;
    type Error = RdfLoaderError;

    fn load_with<'a>(
        &'a mut self,
        _vocabulary: &'a mut (impl Sync + Send + rdf_types::IriVocabularyMut<Iri = ArcIri>),
        url: ArcIri,
    ) -> BoxFuture<'a, LoadingResult<ArcIri, Location<ArcIri, Span>, Self::Output, Self::Error>>
    where
        ArcIri: 'a,
    {
        let url_str = url.as_str().to_string();
        let assets = Arc::clone(&self.assets);
        let cache = Arc::clone(&self.cache);

        async move {
            let asset_doc = assets.get(&url_str).or_else(|| {
                url_str
                    .rsplit('/')
                    .next()
                    .and_then(|fname| assets.get(fname))
            });
            if let Some(doc) = asset_doc {
                return Ok(Self::to_remote_document(url, doc.clone()));
            }

            {
                let read_guard = cache.read().await;
                if let Some(doc) = read_guard.get(&url_str) {
                    return Ok(Self::to_remote_document(url, doc.clone()));
                }
            }

            let mut headers = HeaderMap::new();
            headers.insert(
                ACCEPT,
                HeaderValue::from_static("application/ld+json, application/json"),
            );
            let resp = http_client()
                .get(&url_str, Some(headers))
                .await
                .map_err(|e| RdfLoaderError::Network(url_str.clone(), e.to_string()))?;

            if !resp.status().is_success() {
                return Err(RdfLoaderError::Network(
                    url_str.clone(),
                    format!("HTTP {}", resp.status()),
                ));
            }

            let text = resp
                .text()
                .await
                .map_err(|e| RdfLoaderError::Network(url_str.clone(), e.to_string()))?;

            let parsed = JsonSyntaxValue::parse_str(&text, |span| Location::new(url.clone(), span))
                .map_err(|e| RdfLoaderError::Parse(url_str.clone(), e.to_string()))?;

            {
                let mut write_guard = cache.write().await;
                write_guard.insert(url_str, parsed.clone());
            }

            Ok(Self::to_remote_document(url, parsed))
        }
        .boxed()
    }
}
