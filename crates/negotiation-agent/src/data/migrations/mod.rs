/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use sea_orm_migration::prelude::*;
mod m20251118_000001_negotiation_process;
mod m20251118_000002_negotiation_messages;
mod m20251118_000003_negotiation_process_identifiers;
mod m20251118_000004_offers;
mod m20251118_000005_agreements;

pub fn get_negotiation_agent_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20251118_000001_negotiation_process::Migration),
        Box::new(m20251118_000002_negotiation_messages::Migration),
        Box::new(m20251118_000003_negotiation_process_identifiers::Migration),
        Box::new(m20251118_000004_offers::Migration),
        Box::new(m20251118_000005_agreements::Migration),
    ]
}
