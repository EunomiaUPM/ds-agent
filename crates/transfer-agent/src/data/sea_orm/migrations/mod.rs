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

use sea_orm_migration::MigrationTrait;

mod m20260515_000001_transfer_processes;
mod m20260515_000002_transfer_messages;
mod m20260515_000003_transfer_identifiers;

pub fn get_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20260515_000001_transfer_processes::Migration),
        Box::new(m20260515_000002_transfer_messages::Migration),
        Box::new(m20260515_000003_transfer_identifiers::Migration),
    ]
}
