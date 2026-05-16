pub mod config;
pub mod data;
pub mod entities;
pub mod http;
pub mod services;
pub mod setup;

pub use data::sea_orm::migrations::get_oauth_migrations;
