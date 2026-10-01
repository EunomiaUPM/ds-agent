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

use common::{str_id, urn_id};
use compact_str::CompactString;

// URN-based identifiers ─────────────────────────────────────────────────────
urn_id!(TransferProcessId, gen = "transfer-process");
urn_id!(MessageId, gen = "transfer-message");
urn_id!(ParticipantId);

// String-based identifiers ──────────────────────────────────────────────────
str_id!(CorrelationId, CompactString);
str_id!(RequestId, CompactString, gen);
str_id!(IdempotencyKey, CompactString, gen);
str_id!(TenantId, String, gen);
