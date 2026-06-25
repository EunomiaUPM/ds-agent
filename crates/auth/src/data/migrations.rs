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
use ymir::data::migrations::received;
use ymir::data::migrations::sent;
use ymir::data::migrations::shared;

pub fn get_auth_migrations() -> Vec<Box<dyn MigrationTrait>> {
    let mut m = vec![
        // Shared: picks individuales
        Box::new(shared::participant::Migration) as Box<dyn MigrationTrait>,
        Box::new(shared::issuance::Migration),
    ];
    // Received y sent: bloques enteros
    m.extend(received::get_recv_migrations());
    m.extend(sent::get_sent_migrations());
    m
}
