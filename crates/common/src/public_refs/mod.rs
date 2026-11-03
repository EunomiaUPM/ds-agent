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

//! `GET /open/{urn}`: any URN the process holds, dereferenced as its public JSON-LD document.
//! Each agent contributes a resolver, the gateway remote ones; the first that knows it answers.

pub mod document;
pub mod module;
pub mod remote;
pub mod router;

use urn::Urn;
use ymir::errors::Outcome;

pub use document::{PublicDocument, PublicQuery};
pub use module::PublicRefsModule;
pub use remote::RemotePublicResolver;
pub use router::PublicRefsRouter;

/// One agent's part of the public references.
#[mockall::automock]
#[async_trait::async_trait]
pub trait PublicResolverTrait: Send + Sync {
    /// The public document behind `urn`; none when this agent does not hold it.
    async fn resolve(&self, urn: &Urn, query: &PublicQuery) -> Outcome<Option<PublicDocument>>;
}

#[cfg(test)]
mod tests;
