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

//! RegistryService over mocked auth facades and closure service: selective & incremental dataset sync.

use catalog_agent::services::closure::MockClosureServiceTrait;
use common::facades::registry_facade::MockRegistryFacadeTrait;

use crate::support::builders::{catalog, data_service, dataset, distribution};
use crate::support::context::TestContext;

/// Saving non-dataset resources skips auth entirely; saving published/drafted datasets syncs only that dataset.
#[tokio::test]
async fn sync_saved_handles_datasets_incrementally_and_skips_other_kinds() {
    let mut facade = MockRegistryFacadeTrait::new();
    facade
        .expect_upsert_datasets()
        .times(1)
        .withf(|rows| rows.len() == 1 && rows[0].id == dataset(10, 1).id().to_string())
        .returning(|_| Ok(()));
    facade
        .expect_unregister_datasets()
        .times(1)
        .withf(|ids| *ids == [dataset(10, 1).id().to_string()])
        .returning(|_| Ok(()));

    let ctx = TestContext::new()
        .closures(MockClosureServiceTrait::new())
        .auth_registry(facade)
        .build();

    ctx.registry().sync_saved(&catalog(1)).await;
    ctx.registry()
        .sync_saved(&distribution(20, 10, 2, "JSON"))
        .await;
    ctx.registry().sync_saved(&data_service(2, 1)).await;

    let published = dataset(10, 1);
    ctx.registry().sync_saved(&published).await;
    ctx.registry()
        .sync_saved(&published.with_publication(true))
        .await;
}

/// Deleting a dataset unregisters only its id, deleting a distribution does nothing, and deleting a catalog resyncs.
#[tokio::test]
async fn sync_deleted_unregisters_dataset_skips_distribution_and_resyncs_on_catalog() {
    let mut facade = MockRegistryFacadeTrait::new();
    facade
        .expect_unregister_datasets()
        .times(1)
        .withf(|ids| *ids == [dataset(10, 1).id().to_string()])
        .returning(|_| Ok(()));
    facade
        .expect_sync_datasets()
        .times(1)
        .withf(|rows| rows.is_empty())
        .returning(|_| Ok(()));

    let mut closures = MockClosureServiceTrait::new();
    closures.expect_main_catalog().times(1).returning(|_| {
        Ok((
            catalog(1),
            catalog_agent::entities::resource_set::ResourceSet::new(),
        ))
    });

    let ctx = TestContext::new()
        .closures(closures)
        .auth_registry(facade)
        .build();

    ctx.registry()
        .sync_deleted(&distribution(20, 10, 2, "JSON"))
        .await;
    ctx.registry().sync_deleted(&dataset(10, 1)).await;
    ctx.registry().sync_deleted(&catalog(1)).await;
}

/// Importing batches published datasets into one upsert and drafted datasets into one remove without loading closures.
#[tokio::test]
async fn sync_imported_batches_datasets_without_loading_main_catalog() {
    let mut facade = MockRegistryFacadeTrait::new();
    facade
        .expect_upsert_datasets()
        .times(1)
        .withf(|rows| rows.len() == 1 && rows[0].id == dataset(10, 1).id().to_string())
        .returning(|_| Ok(()));
    facade
        .expect_unregister_datasets()
        .times(1)
        .withf(|ids| *ids == [dataset(11, 1).id().to_string()])
        .returning(|_| Ok(()));

    let ctx = TestContext::new()
        .closures(MockClosureServiceTrait::new())
        .auth_registry(facade)
        .build();

    ctx.registry()
        .sync_imported(&[
            catalog(1),
            dataset(10, 1),
            dataset(11, 1).with_publication(true),
            distribution(20, 10, 2, "JSON"),
        ])
        .await;
}
