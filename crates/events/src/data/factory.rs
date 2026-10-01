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

use std::sync::Arc;

use crate::data::repo::{
    EventDeadLetterRepo, EventDeliveryRepo, EventStoreRepo, EventSubscriptionRepo,
};

// Abstract factory trait providing constructor access to events repository trait objects.
pub trait DataFactory: Send + Sync {
    fn event_repository(&self) -> Arc<dyn EventStoreRepo>;
    fn subscription_repository(&self) -> Arc<dyn EventSubscriptionRepo>;
    fn delivery_repository(&self) -> Arc<dyn EventDeliveryRepo>;
    fn dlq_repository(&self) -> Arc<dyn EventDeadLetterRepo>;
}
