//! Shared primitives used across the connector domain.
//!
//! | Module | Contents |
//! |---|---|
//! | [`secret_management`] | [`SecretSource`] and [`SecretString`] for credential storage |
//!
//! [`SecretSource`]: secret_management::SecretSource
//! [`SecretString`]: secret_management::SecretString

pub(crate) mod secret_management;

pub mod parameter_mutator {
    // Deprecated or removed. Use ParameterResolverBehavior instead.
}
