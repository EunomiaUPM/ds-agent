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

//! Protocol specification types for connector interactions.
//!
//! A [`ProtocolSpec`] wraps either an [`HttpSpec`] or a [`KafkaSpec`] and
//! describes the concrete transport used for a Pull data-access or a Push
//! subscribe / unsubscribe call.
//!
//! All string-typed fields in these specs support `{{__PARAM__}}` placeholders.
//!
//! Processing of these specs is handled via the [`ConnectorTemplateWalker`]
//! in the [`parameters`] module.
//!
//! [`ConnectorTemplateWalker`]: crate::entities::parameters::template_walker::ConnectorTemplateWalker
//! [`parameters`]: crate::entities::parameters

pub mod http;
pub mod kafka;

pub use http::*;
pub use kafka::*;

use serde::{Deserialize, Serialize};

/// A connector protocol: either HTTP or Kafka.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "protocol")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtocolSpec {
    Http(HttpSpec),
    Kafka(KafkaSpec),
}
