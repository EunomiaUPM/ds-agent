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

//! Declarative macros for defining, implementing, and emitting domain events.

// Declarative macro to define or implement domain events with static topics and source crates.
#[macro_export]
macro_rules! event {
    // Form 1: Inline struct definition with topic and source crate
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field_vis:vis $field:ident : $field_ty:ty),* $(,)?
        } => $topic:expr, $source_crate:expr
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $($field_vis $field : $field_ty),*
        }
        $crate::event!($name, $topic, $source_crate, 1);
    };

    // Form 2: Inline struct definition with topic, source crate, and schema version
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field_vis:vis $field:ident : $field_ty:ty),* $(,)?
        } => $topic:expr, $source_crate:expr, $version:expr
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $($field_vis $field : $field_ty),*
        }
        $crate::event!($name, $topic, $source_crate, $version);
    };

    // Form 3: Implementation for existing struct with explicit schema version
    ($type:ty, $topic:expr, $source_crate:expr, $version:expr) => {
        impl $crate::entities::event::Event for $type {
            fn event_name() -> &'static str {
                $topic
            }

            fn topic() -> $crate::entities::topic::Topic {
                $crate::entities::topic::Topic::new($topic).expect("valid static topic")
            }

            fn source_crate() -> &'static str {
                $source_crate
            }

            fn schema_version() -> u32 {
                $version
            }

            fn into_envelope(self) -> $crate::entities::envelope::EventEnvelope {
                let tenant_id = <Self as $crate::entities::event::Event>::tenant_id(&self);
                $crate::entities::envelope::EventEnvelope::new(
                    tenant_id,
                    <Self as $crate::entities::event::Event>::topic(),
                    $source_crate,
                    <Self as $crate::entities::event::Event>::schema_version(),
                    <Self as $crate::entities::event::Event>::correlation_id(&self),
                    serde_json::to_value(&self).expect("serializable event payload"),
                )
            }
        }

        impl $crate::entities::event::IntoEvent for $type {
            fn topic() -> $crate::entities::topic::Topic {
                <Self as $crate::entities::event::Event>::topic()
            }

            fn schema_version() -> u32 {
                <Self as $crate::entities::event::Event>::schema_version()
            }

            fn into_envelope(self) -> $crate::entities::envelope::EventEnvelope {
                <Self as $crate::entities::event::Event>::into_envelope(self)
            }
        }
    };

    // Form 4: Implementation for existing struct with default schema version (1)
    ($type:ty, $topic:expr, $source_crate:expr) => {
        $crate::event!($type, $topic, $source_crate, 1);
    };

    // Form 5: Implementation for existing struct with topic only
    ($type:ty, $topic:expr) => {
        $crate::event!($type, $topic, "events", 1);
    };
}

// Backward-compatible macro alias.
#[macro_export]
macro_rules! impl_into_event {
    ($($arg:tt)*) => {
        $crate::event!($($arg)*);
    };
}

// Ergonomic macro to publish events using hierarchical topics (<prefix>:<service>:<action>).
#[macro_export]
macro_rules! emit_action {
    ($bus:expr, $tenant:expr, $prefix:expr, $service:expr, $action:expr, $payload:expr) => {
        if let Some(bus) = &$bus {
            let topic = format!("{}{}:{}", $prefix, $service, $action);
            let source = $prefix.trim_end_matches(':');
            if let Err(e) = bus
                .emit_payload_with_tenant($tenant, &topic, source, $payload)
                .await
            {
                tracing::warn!("Failed to emit event {topic}: {e}");
            }
        }
    };
    ($bus:expr, $prefix:expr, $service:expr, $action:expr, $payload:expr) => {
        if let Some(bus) = &$bus {
            let topic = format!("{}{}:{}", $prefix, $service, $action);
            let source = $prefix.trim_end_matches(':');
            if let Err(e) = bus.emit_payload(&topic, source, $payload).await {
                tracing::warn!("Failed to emit event {topic}: {e}");
            }
        }
    };
}
