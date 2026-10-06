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

//! HTTP paths of the agents, shared by their routers and by whoever calls them (remote facades,
//! the gateway), so both sides cannot drift apart.
//!
//! Every path goes after the agent's API version (`get_api_version()`). Paths with a parameter
//! are axum templates; to build a URL, fill them with [`ymir::http::routes::fill`]. Routes of
//! ymir's own routers and of the OID4VP verifier live in [`ymir::http::routes`].

/// SSI auth agent: one module per capability, each with its mount `PREFIX` and its routes.
pub mod auth {
    /// The agent's wallet; its routes are ymir's ([`ymir::http::routes::wallet`]).
    pub mod wallet {
        pub const PREFIX: &str = "/wallet";
    }

    /// Participants (mates).
    pub mod mates {
        pub const PREFIX: &str = "/mates";
        pub const ROOT: &str = "/";
        pub const ALL: &str = "/all";
        pub const MYSELF: &str = "/myself";
        pub const BATCH: &str = "/batch";
        pub const BY_ID: &str = "/{id}";
    }

    /// Credential requests to an authority.
    pub mod vc_request {
        pub const PREFIX: &str = "/vc-request";
        /// GNAP interaction finish, pushed by the authority.
        pub const CALLBACK: &str = "/callback/{id}";
        pub const BEG: &str = "/beg";
        pub const ALL: &str = "/all";
        pub const BY_ID: &str = "/{id}";
        pub const DETAILS: &str = "/{id}/details";
        pub const OID4VCI: &str = "/oid4vci/{id}";
        pub const OID4VP: &str = "/oid4vp/{id}";
    }

    /// Onboarding with peers (grants sent).
    pub mod peer_connection {
        pub const PREFIX: &str = "/peer-connection";
        /// GNAP interaction finish, pushed by the peer.
        pub const CALLBACK: &str = "/callback/{id}";
        pub const CONNECT: &str = "/connect";
        pub const REQUEST_ALL: &str = "/request/all";
        pub const REQUEST: &str = "/request/{id}";
        pub const REQUEST_DETAILS: &str = "/request/{id}/details";
        pub const OID4VP: &str = "/oid4vp/{id}";
        /// The caller's token towards a peer, by the peer's participant id (grants facade).
        pub const TOKEN: &str = "/token/{id}";
    }

    /// GNAP gatekeeper (grants received). Its `PREFIX` is the authorization server advertised in
    /// the agent's DID document.
    pub mod gate {
        pub const PREFIX: &str = "/gate";
        /// Grant request of a peer.
        pub const ACCESS: &str = "/access";
        /// Continuation of a grant, by its continuation id.
        pub const CONTINUE: &str = "/continue/{id}";
        pub const TOKEN: &str = "/token/{id}";
        pub const REQUEST_ALL: &str = "/request/all";
        pub const REQUEST: &str = "/request/{id}";
        pub const REQUEST_DETAILS: &str = "/request/{id}/details";
        /// Check of a token a peer presents (grants facade).
        pub const TOKEN_VERIFY: &str = "/token/verify";
    }

    /// Gaia-X self-attestation.
    pub mod gaia {
        pub const PREFIX: &str = "/gaia";
        pub const GENERATE: &str = "/generate";
    }

    /// API documentation; its routes are ymir's ([`ymir::http::routes::openapi`]).
    pub mod docs {
        pub const PREFIX: &str = "/docs";
    }
}
