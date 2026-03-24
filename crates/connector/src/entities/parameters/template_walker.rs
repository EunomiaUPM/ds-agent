/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

//! Shared structural traversal trait for [`ConnectorTemplateDto`] walkers.
//!
//! [`ConnectorTemplateWalker`] is the single place where the
//! authentication + interaction tree is traversed.  Both read-only
//! (parameter extraction) and mutable (template resolution) visitors
//! implement this trait; they only need to supply three leaf callbacks.
//!
//! # Traversal order
//!
//! 1. `authentication` fields
//! 2. `interaction` fields (pull → data_access; push → subscribe then unsubscribe)
//!
//! [`ConnectorTemplateDto`]: crate::entities::connector_template::ConnectorTemplateDto

use crate::entities::auth_config::AuthenticationConfig;
use crate::entities::connector_template::ConnectorTemplateDto;
use crate::entities::interaction::{InteractionConfig, PullLifecycle, PushLifecycle};
use crate::entities::parameters::{TemplateMapString, TemplateVecString};
use crate::entities::resource::{HttpSpec, KafkaSpec};
use crate::ProtocolSpec;
use ymir::errors::Outcome;

/// Structural traversal over the templatable fields of a [`ConnectorTemplateDto`].
///
/// Implementors supply the three leaf callbacks; the authentication and
/// interaction routing is provided as default implementations so the
/// tree-walking logic is defined exactly once.
///
/// # Associated type
///
/// `Error` lets each implementor declare its own error type:
///
/// | Implementor | `Error` |
/// |---|---|
/// | [`TemplateResolverVisitor`] | `anyhow::Error` |
/// | [`ParameterExtractorVisitor`] | `std::convert::Infallible` |
///
/// [`TemplateResolverVisitor`]: super::template_resolver_visitor::TemplateResolverVisitor
/// [`ParameterExtractorVisitor`]: super::template_parameters_visitor::ParameterExtractorVisitor
pub trait ConnectorTemplateWalker {
    // -------------------------------------------------------------------------
    // Leaf operations — implementors must supply these.
    // -------------------------------------------------------------------------

    fn on_string(&mut self, field: &mut String) -> Outcome<()>;
    fn on_vec_string(&mut self, field: &mut TemplateVecString) -> Outcome<()>;
    fn on_map_string(&mut self, field: &mut TemplateMapString) -> Outcome<()>;

    // -------------------------------------------------------------------------
    // Structural traversal — defined once here, shared by all implementors.
    // -------------------------------------------------------------------------

    fn walk(&mut self, template: &mut ConnectorTemplateDto) -> Outcome<()> {
        self.walk_auth(&mut template.authentication)?;
        self.walk_interaction(&mut template.interaction)?;
        Ok(())
    }

    fn walk_auth(&mut self, auth: &mut AuthenticationConfig) -> Outcome<()> {
        match auth {
            AuthenticationConfig::NoAuth => {}
            AuthenticationConfig::BasicAuth(c) => {
                self.on_string(&mut c.username)?;
            }
            // BearerToken.token is a SecretString — intentionally not walked.
            AuthenticationConfig::BearerToken { .. } => {}
            AuthenticationConfig::ApiKey { key, .. } => {
                self.on_string(key)?;
            }
            AuthenticationConfig::OAuth2 {
                token_url,
                client_id,
                scopes,
                ..
            } => {
                self.on_string(token_url)?;
                self.on_string(client_id)?;
                self.on_vec_string(scopes)?;
            }
        }
        Ok(())
    }

    fn walk_interaction(&mut self, interaction: &mut InteractionConfig) -> Outcome<()> {
        match interaction {
            InteractionConfig::Pull(lc) => self.walk_pull(lc),
            InteractionConfig::Push(lc) => self.walk_push(lc),
        }
    }

    fn walk_pull(&mut self, lc: &mut PullLifecycle) -> Outcome<()> {
        self.walk_protocol(&mut lc.data_access)
    }

    fn walk_push(&mut self, lc: &mut PushLifecycle) -> Outcome<()> {
        self.walk_protocol(&mut lc.subscribe)?;
        if let Some(unsub) = &mut lc.unsubscribe {
            self.walk_protocol(unsub)?;
        }
        Ok(())
    }

    fn walk_protocol(&mut self, spec: &mut ProtocolSpec) -> Outcome<()> {
        match spec {
            ProtocolSpec::Http(s) => self.walk_http(s),
            ProtocolSpec::Kafka(s) => self.walk_kafka(s),
        }
    }

    fn walk_http(&mut self, spec: &mut HttpSpec) -> Outcome<()> {
        self.on_string(&mut spec.url_template)?;
        self.on_vec_string(&mut spec.method)?;
        if let Some(headers) = &mut spec.headers {
            self.on_map_string(headers)?;
        }
        if let Some(body) = &mut spec.body_template {
            self.on_string(body)?;
        }
        Ok(())
    }

    fn walk_kafka(&mut self, spec: &mut KafkaSpec) -> Outcome<()> {
        self.on_vec_string(&mut spec.brokers)?;
        self.on_string(&mut spec.topic)?;
        if let Some(group_id) = &mut spec.group_id {
            self.on_string(group_id)?;
        }
        Ok(())
    }
}
