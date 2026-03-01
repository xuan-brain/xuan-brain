//! Database module for SQLite using SeaORM
//!
//! Provides database connection, entities, and migrations.

pub mod connection;
pub mod entities;
pub mod migration;

#[allow(unused_imports)]
pub use connection::init_sqlite_connection;
pub use sea_orm::DatabaseConnection;
