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

use axum::Router;

/// Mount `sub` on `router`: nested under `path`, or merged when `path` is
/// empty (axum forbids nesting at `/`; an empty path means "at this level").
pub(crate) fn mount(router: Router, path: &str, sub: Router) -> Router {
    if path.is_empty() {
        router.merge(sub)
    } else {
        router.nest(path, sub)
    }
}
