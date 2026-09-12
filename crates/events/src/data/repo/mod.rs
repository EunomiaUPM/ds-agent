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

pub mod dead_letter;
pub mod delivery;
pub mod event;
pub mod subscription;

pub use dead_letter::*;
pub use delivery::*;
pub use event::*;
pub use subscription::*;

// Re-exports of domain records for backward compatibility with existing tests and modules
pub use crate::entities::commands::{CreateSubscriptionDto, UpdateSubscriptionDto};
pub use crate::entities::dead_letter::DeadLetterRecord;
pub use crate::entities::delivery::EventDeliveryRecord;
pub use crate::entities::subscription::{DeadLetterStatus, DeliveryStatus, SubscriptionRecord};

// Re-export concrete implementations for legacy path compatibility
pub use crate::data::in_memory::InMemoryEventBusRepo;
pub use crate::data::sea_orm::SeaOrmEventBusRepo;
