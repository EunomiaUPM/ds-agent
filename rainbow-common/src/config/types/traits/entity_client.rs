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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::config::types::{EntityClientConfig, DisplayInfo};

pub trait EntityClientTrait {
    fn client_config(&self) -> &EntityClientConfig;
    fn get_clas_id(&self) -> &str {
        &self.client_config().class_id
    }
    fn get_display_info(&self) -> Option<&DisplayInfo> {
        self.client_config().display.as_ref()
    }
}
