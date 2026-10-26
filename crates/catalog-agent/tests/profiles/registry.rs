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

//! ProfileRegistry: adapters by key, in order, one per key.

use std::sync::Arc;

use catalog_agent::profiles::dcat3::Dcat3;
use catalog_agent::profiles::dsp::DspProfileAdapter;
use catalog_agent::profiles::shared::dcat::DcatAdapter;
use catalog_agent::profiles::ProfileRegistry;

/// Adapters come back in registration order and by key.
#[test]
fn adapters_are_found_by_key_in_order() {
    let registry = ProfileRegistry::default();
    registry.register(Arc::new(DspProfileAdapter::new()));
    registry.register(Arc::new(DcatAdapter::<Dcat3>::default()));

    assert_eq!(registry.keys(), vec!["dsp", "dcat3"]);
    assert!(registry.get("dcat3").is_some());
    assert!(registry.get("geodcat-ap").is_none());
}

/// Registering a key again replaces the adapter, it never doubles it.
#[test]
fn a_key_is_registered_once() {
    let registry = ProfileRegistry::default();
    registry.register(Arc::new(DspProfileAdapter::new()));
    registry.register(Arc::new(DspProfileAdapter::new()));

    assert_eq!(registry.all().len(), 1);
}
