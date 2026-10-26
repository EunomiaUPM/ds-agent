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
//! the gateway, the edge proxy's list of protocol routes), so both sides cannot drift apart.
//!
//! Each capability module says whether its routes are **external** (protocol: peers, wallets,
//! authorities; they reach the agent without a session) or **internal** (users, behind the OAuth
//! guard).
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
        pub const RELATIONS: &str = "/{id}/relations";
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
        pub const TOKEN_REFRESH: &str = "/token/{id}/refresh";
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

/// Catalog agent: its user API under `{api}/catalog-agent` (internal) and the DSP catalog protocol
/// under `dsp_common::paths::CATALOG`.
pub mod catalog {
    /// Paths shared by the entity routers (internal: users), relative to each entity's `PREFIX`.
    pub mod entity {
        pub const ROOT: &str = "/";
        pub const BATCH: &str = "/batch";
        pub const BY_ID: &str = "/{id}";
        /// The connector's main entity (catalog, data service).
        pub const MAIN: &str = "/main";
        /// The children of a catalog (datasets, data services).
        pub const BY_CATALOG: &str = "/catalog/{id}";
        /// Whether the resource conforms to each profile (`?profile=`, `?deep=true`).
        pub const DOCTOR: &str = "/{id}/doctor";
    }

    pub mod catalogs {
        pub const PREFIX: &str = "/catalogs";
    }

    pub mod data_services {
        pub const PREFIX: &str = "/data-services";
    }

    pub mod datasets {
        pub const PREFIX: &str = "/datasets";
    }

    pub mod dataset_series {
        pub const PREFIX: &str = "/dataset-series";
    }

    pub mod agents {
        pub const PREFIX: &str = "/agents";
    }

    pub mod catalog_records {
        pub const PREFIX: &str = "/catalog-records";
    }

    pub mod distributions {
        pub const PREFIX: &str = "/distributions";
        pub const BY_DATASET: &str = "/dataset/{id}";
        pub const BY_DATASET_AND_FORMAT: &str = "/dataset/{id}/format/{format}";
    }

    pub mod odrl_policies {
        pub const PREFIX: &str = "/odrl-policies";
        pub const BY_ENTITY: &str = "/entity/{entity_id}";
    }

    pub mod policy_templates {
        pub const PREFIX: &str = "/policy-templates";
        pub const BY_ID_AND_VERSION: &str = "/{id}/{version}";
        pub const INSTANTIATE_ODRL_OFFER: &str = "/instantiate-odrl-offer";
    }

    pub mod peer_catalogs {
        pub const PREFIX: &str = "/peer-catalogs";
        pub const ROOT: &str = "/";
        pub const BY_PEER: &str = "/{peer_id}";
    }

    pub mod dataset_offerings {
        pub const PREFIX: &str = "/dataset-offerings";
        pub const ROOT: &str = "/";
    }

    /// Read-only routes of each profile (public), relative to `{api}/catalog-agent`.
    pub mod public {
        pub const PREFIX: &str = "/public";
        pub const CATALOG: &str = "/catalog";
        pub const RESOURCE: &str = "/resources/{id}";
        pub const CONTEXT: &str = "/context.jsonld";
    }

    /// DSP catalog protocol (external: peers), under `dsp_common::paths::CATALOG`.
    pub mod dsp {
        pub const REQUEST: &str = "/request";
        pub const DATASET: &str = "/datasets/{id}";
    }

    /// Catalog requests to a peer (internal: users), under `dsp_common::paths::CATALOG`.
    pub mod rpc {
        pub const SETUP_CATALOG_REQUEST: &str = "/rpc/setup-catalog-request";
        pub const SETUP_DATASET_REQUEST: &str = "/rpc/setup-dataset-request";
    }
}

/// Negotiation agent: its user API under `{api}/negotiation-agent` (internal) and the DSP
/// negotiation protocol under `dsp_common::paths::NEGOTIATIONS`.
pub mod negotiation {
    /// Paths shared by the entity routers (internal: users), relative to each entity's `PREFIX`.
    pub mod entity {
        pub const ROOT: &str = "/";
        pub const BATCH: &str = "/batch";
        pub const BY_ID: &str = "/{id}";
        /// The records of a negotiation process.
        pub const BY_PROCESS: &str = "/process/{process_id}";
    }

    pub mod negotiation_messages {
        pub const PREFIX: &str = "/negotiation-messages";
    }

    pub mod negotiation_processes {
        pub const PREFIX: &str = "/negotiation-processes";
        pub const KEY: &str = "/{id}/key/{key_id}";
    }

    pub mod offers {
        pub const PREFIX: &str = "/offers";
        pub const BY_OFFER_ID: &str = "/offer-id/{offer_id}";
    }

    pub mod agreements {
        pub const PREFIX: &str = "/agreements";
        pub const BY_ASSIGNEE: &str = "/assignee/{assignee}";
        pub const BY_ASSIGNER: &str = "/assigner/{assigner}";
    }

    /// DSP negotiation protocol (external: peers), under `dsp_common::paths::NEGOTIATIONS`.
    pub mod dsp {
        pub const PROCESS: &str = "/{id}";
        pub const REQUEST: &str = "/request";
        pub const PROCESS_REQUEST: &str = "/{id}/request";
        pub const OFFERS: &str = "/offers";
        pub const PROCESS_OFFERS: &str = "/{id}/offers";
        pub const PROCESS_EVENTS: &str = "/{id}/events";
        pub const PROCESS_AGREEMENT: &str = "/{id}/agreement";
        pub const PROCESS_AGREEMENT_VERIFICATION: &str = "/{id}/agreement/verification";
        pub const PROCESS_TERMINATION: &str = "/{id}/termination";
    }

    /// Negotiation steps towards a peer (internal: users), under `dsp_common::paths::NEGOTIATIONS`.
    pub mod rpc {
        pub const SETUP_REQUEST_INIT: &str = "/rpc/setup-request-init";
        pub const SETUP_REQUEST: &str = "/rpc/setup-request";
        pub const SETUP_OFFER_INIT: &str = "/rpc/setup-offer-init";
        pub const SETUP_OFFER: &str = "/rpc/setup-offer";
        pub const SETUP_ACCEPTANCE: &str = "/rpc/setup-acceptance";
        pub const SETUP_AGREEMENT: &str = "/rpc/setup-agreement";
        pub const SETUP_VERIFICATION: &str = "/rpc/setup-verification";
        pub const SETUP_FINALIZATION: &str = "/rpc/setup-finalization";
        pub const SETUP_TERMINATION: &str = "/rpc/setup-termination";
        /// TCK trigger; only served in TCK mode.
        pub const TCK_REQUESTS: &str = "/tck/negotiations/requests";
    }

    /// The negotiation steps of the console (internal: users), under
    /// `dsp_common::paths::NEGOTIATIONS`.
    pub mod bff_rpc {
        pub const SETUP_REQUEST_INIT: &str = "/bff-rpc/setup-request-init";
        pub const SETUP_OFFER_INIT: &str = "/bff-rpc/setup-offer-init";
        pub const SETUP_ACCEPTANCE: &str = "/bff-rpc/setup-acceptance";
        pub const SETUP_AGREEMENT: &str = "/bff-rpc/setup-agreement";
        pub const SETUP_TERMINATION: &str = "/bff-rpc/setup-termination";
    }
}

/// Transfer agent: its user API under `{api}/transfer-agent` (internal) and the DSP transfer
/// protocol under `dsp_common::paths::TRANSFERS`.
pub mod transfer {
    /// Paths shared by the entity routers (internal: users), relative to each entity's `PREFIX`.
    pub mod entity {
        pub const ROOT: &str = "/";
        pub const BATCH: &str = "/batch";
        pub const BY_ID: &str = "/{id}";
        /// The records of a transfer process.
        pub const BY_PROCESS: &str = "/process/{process_id}";
    }

    pub mod transfer_processes {
        pub const PREFIX: &str = "/transfer-processes";
    }

    pub mod transfer_messages {
        pub const PREFIX: &str = "/transfer-messages";
    }

    /// DSP transfer protocol (external: peers), under `dsp_common::paths::TRANSFERS`.
    pub mod dsp {
        pub const REQUEST: &str = "/request";
        pub const PROCESS: &str = "/{id}";
        pub const PROCESS_START: &str = "/{id}/start";
        pub const PROCESS_COMPLETION: &str = "/{id}/completion";
        pub const PROCESS_TERMINATION: &str = "/{id}/termination";
        pub const PROCESS_SUSPENSION: &str = "/{id}/suspension";
    }

    /// Transfer steps towards a peer (internal: users), under `dsp_common::paths::TRANSFERS`.
    pub mod rpc {
        pub const SETUP_REQUEST: &str = "/rpc/setup-request";
        pub const SETUP_START: &str = "/rpc/setup-start";
        pub const SETUP_COMPLETION: &str = "/rpc/setup-completion";
        pub const SETUP_SUSPENSION: &str = "/rpc/setup-suspension";
        pub const SETUP_TERMINATION: &str = "/rpc/setup-termination";
        /// TCK trigger; only served in TCK mode.
        pub const TCK_REQUESTS: &str = "/tck/transfers/requests";
    }

    /// The transfer steps of the console (internal: users), under
    /// `dsp_common::paths::TRANSFERS`.
    pub mod bff_rpc {
        /// A request the provider starts on its own (`autoStart`).
        pub const SETUP_REQUEST: &str = "/bff-rpc/setup-request";
    }
}

/// Data plane: its control API (internal: users) under its prefix and the data proxy (external: the
/// peer pulling the data) under `HTTP_LISTENER_PATH`.
pub mod dataplane {
    pub mod dataplane_processes {
        pub const PREFIX: &str = "/dataplane-processes";
        pub const ROOT: &str = "/";
        pub const BATCH: &str = "/batch";
        pub const BY_ID: &str = "/{dataplane_id}";
        pub const INFO: &str = "/{dataplane_id}/info";
        pub const BY_TRANSFER_PROCESS: &str = "/transfer-process/{transfer_process_id}";
        pub const LOGS: &str = "/{dataplane_process_id}/logs";
        pub const EVENTS: &str = "/{dataplane_process_id}/events";
    }

    pub mod transfer_events {
        pub const PREFIX: &str = "/transfer-events";
        pub const BY_ID: &str = "/{event_id}";
    }

    /// Data proxy (external), under `HTTP_LISTENER_PATH`.
    pub mod proxy {
        pub const BY_DATAPLANE: &str = "/{data_plane_id}";
        pub const PATH: &str = "/{data_plane_id}/{*path}";
    }
}

/// Event bus: its API under `/api/v1/events` (internal: users).
pub mod events {
    /// The events themselves, at the root of the API.
    pub mod entries {
        pub const ROOT: &str = "/";
        /// Server-sent events.
        pub const STREAM: &str = "/stream";
        /// Read state of the caller.
        pub const UNREAD_COUNT: &str = "/unread-count";
        pub const READ_ALL: &str = "/read-all";
        pub const BY_ID: &str = "/{id}";
        pub const READ: &str = "/{id}/read";
        pub const DELIVERIES: &str = "/{id}/deliveries";
    }

    pub mod subscriptions {
        pub const PREFIX: &str = "/subscriptions";
        pub const ROOT: &str = "/";
        pub const BY_ID: &str = "/{id}";
    }

    /// Dead letter queue.
    pub mod dlq {
        pub const PREFIX: &str = "/dlq";
        pub const ROOT: &str = "/";
        pub const REPLAY_ALL: &str = "/replay-all";
        pub const BY_ID: &str = "/{id}";
        pub const REPLAY: &str = "/{id}/replay";
    }
}

/// Keystore: its API under `{api}/keystore` (internal: users).
pub mod keystore {
    pub mod parameters {
        pub const PREFIX: &str = "/parameters";
        pub const ROOT: &str = "/";
        pub const BY_KEY: &str = "/{*key}";
    }

    pub mod secrets {
        pub const PREFIX: &str = "/secrets";
        pub const ROOT: &str = "/";
        pub const BY_KEY: &str = "/{*key}";
    }

    pub mod config {
        pub const PREFIX: &str = "/config";
        pub const ROOT: &str = "/";
    }
}

/// Connector (templates and instances): its API under `{api}/connector` (internal: users).
pub mod connector {
    pub mod templates {
        pub const PREFIX: &str = "/templates";
        pub const ROOT: &str = "/";
        pub const BY_ID: &str = "/{id}";
        pub const BY_NAME_AND_VERSION: &str = "/{name}/{version}";
    }

    pub mod instances {
        pub const PREFIX: &str = "/instances";
        pub const ROOT: &str = "/";
        pub const BY_ID: &str = "/{id}";
        pub const BY_DISTRIBUTION: &str = "/distribution/{did}";
    }
}

/// Gateway (BFF) of the console (internal: users).
pub mod bff {
    /// The host's root, which redirects to the console.
    pub const ROOT: &str = "/";
    pub const CONSOLE: &str = "/admin";
    pub const CONSOLE_INDEX: &str = "/admin/";
    /// The console's API; also served under `API`.
    pub const CONSOLE_API: &str = "/admin/api";
    pub const API: &str = "/api";
    pub const FE_CONFIG: &str = "/fe-config";
    /// DSP requests of the console, forwarded to the agents.
    pub const DSP: &str = "/dsp/current/{service_prefix}/{*extra}";
    pub const WELL_KNOWN_RPC: &str = "/well-known/rpc/{*extra}";
    /// Forwarded to the agent that serves `service_prefix`.
    pub const SERVICE: &str = "/{service_prefix}";
    pub const SERVICE_PATH: &str = "/{service_prefix}/{*extra}";
    /// Who the session is.
    pub const ME: &str = "/me";
    /// The console's live updates: a WebSocket fed by the event bus.
    pub const LIVE: &str = "/live";
    /// A peer's DID document, fetched server side.
    pub const DID_JSON: &str = "/did-json/{url}";
    /// An authority's federated catalog, fetched server side.
    pub const FEDERATED_CATALOG: &str = "/federated-catalog/{url}";
}
