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

//! MainCatalogService over mocked repositories and participant registry.

use catalog_agent::data::repo::resource::MockResourceRepoTrait;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::vocab::Vocab;
use common::facades::mates_facade::MockMatesFacadeTrait;
use common::test_utils::scopes::TestUsers;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::types::participants::ParticipantType;

use crate::support::builders::{data_service, main_catalog};
use crate::support::context::TestContext;

fn me() -> Mates {
    Mates {
        participant_id: "did:web:provider.example".into(),
        participant_nick: "provider".into(),
        participant_type: ParticipantType::Agent,
        base_url: "https://provider.example".into(),
        saved_at: chrono::Utc::now(),
        last_interaction: chrono::Utc::now(),
        extra_fields: serde_json::json!({}),
    }
}

/// A fresh connector gets a main catalog with its identity and a main data service at its DSP
/// endpoint, both the connector's.
#[tokio::test]
async fn fresh_connector_is_provisioned() {
    let mut repo = MockResourceRepoTrait::new();
    let stored = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mains = stored.clone();
    repo.expect_get_main().returning(move |kind| {
        Ok(mains.lock().unwrap().iter().find(|r: &&catalog_agent::entities::resource::CatalogResource| r.kind() == kind).cloned())
    });
    let parents = stored.clone();
    repo.expect_get_by_id()
        .returning(move |_, id| Ok(parents.lock().unwrap().iter().find(|r| r.id() == id).cloned()));
    let inserted = stored.clone();
    repo.expect_insert().returning(move |r| {
        inserted.lock().unwrap().push(r.clone());
        Ok(r.clone())
    });
    let mut mates = MockMatesFacadeTrait::new();
    mates.expect_get_me_mate().returning(|| Ok(me()));
    let ctx = TestContext::new().resource_repo(repo).mates(mates).build();

    let main = ctx.main_catalog().provision(&TestUsers::root()).await.unwrap();

    assert!(main.catalog.is_main() && main.data_service.is_main());
    assert_eq!(
        main.catalog.description().first_iri(Vocab::DSPACE_PARTICIPANT_ID),
        Some("did:web:provider.example")
    );
    assert!(main.data_service.description().first_iri(Vocab::DCAT_ENDPOINT_URL).unwrap().ends_with("/dsp/current"));
    assert_eq!(main.data_service.catalog_id(), Some(main.catalog.id()));
}

/// Provisioning twice keeps what there is.
#[tokio::test]
async fn provisioning_is_idempotent() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_main().returning(|kind| {
        Ok(Some(if kind == ResourceKind::Catalog { main_catalog(1) } else { data_service(2, 1) }))
    });
    let ctx = TestContext::new().resource_repo(repo).build();

    let main = ctx.main_catalog().provision(&TestUsers::root()).await.unwrap();

    assert_eq!(main.catalog.id(), main_catalog(1).id());
}
