/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use sea_orm_migration::MigrationTrait;
use ymir::data::migrations::{
    m20250403_094651_business_mates, m20250403_094651_issuing, m20250403_094651_mates,
    m20250403_094651_recv_interaction, m20250403_094651_recv_request,
    m20250403_094651_recv_verification, m20250403_094651_req_interaction,
    m20250403_094651_req_request, m20250403_094651_req_vc, m20250403_094651_req_verification,
    m20250403_094651_token_requirements,
};

pub fn get_auth_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20250403_094651_req_request::Migration),
        Box::new(m20250403_094651_req_interaction::Migration),
        Box::new(m20250403_094651_token_requirements::Migration),
        Box::new(m20250403_094651_req_verification::Migration),
        Box::new(m20250403_094651_req_vc::Migration),
        Box::new(m20250403_094651_mates::Migration),
        Box::new(m20250403_094651_recv_request::Migration),
        Box::new(m20250403_094651_recv_verification::Migration),
        Box::new(m20250403_094651_business_mates::Migration),
        Box::new(m20250403_094651_recv_interaction::Migration),
        Box::new(m20250403_094651_issuing::Migration),
    ]
}
