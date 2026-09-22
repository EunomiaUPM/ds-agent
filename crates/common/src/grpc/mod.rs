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

//! Transport helpers shared by every gRPC adapter: status mapping, field parsing, paging and JSON.

pub mod field;
pub mod json;
pub mod page;
pub mod status;

pub use field::{InvalidField, ProtoEnum, ProtoField, ProtoFieldList};
pub use json::{JsonStruct, JsonStructExt, JsonValueExt};
pub use page::{PageMeta, PageParams};
pub use status::IntoStatus;
