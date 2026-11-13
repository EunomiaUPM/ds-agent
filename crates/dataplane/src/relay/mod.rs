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

//! Supervised Kafka `PUSH` relay engine (`Kafka -> HTTP sink`), distributed per-transfer
//! leases, and crash-recovery reconciliation worker.

pub mod consumer;
pub mod lease;
pub mod recovery;
pub mod supervisor;
pub mod worker;

pub use consumer::{
    KafkaConsumerConfig, KafkaConsumerFactory, KafkaConsumerSession, KafkaRecord,
    RdkafkaConsumerFactory,
};
pub use lease::{KafkaLeaseManager, DEFAULT_KAFKA_LEASE_TTL_MS};
pub use recovery::KafkaRecoveryWorker;
pub use supervisor::KafkaRelaySupervisor;
pub use worker::{KafkaRelayTaskSpec, KafkaRelayWorker, DEFAULT_KAFKA_RELAY_BACKOFF_MS};
