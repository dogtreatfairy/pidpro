// lib.rs - Public API for pidpro crate

pub mod database;
pub mod boards;
pub mod controller;

// Re-export commonly used types
pub use database::sql::SqlDbManager;
pub use controller::{ControllerSchema, pmode::PModeController, pid::PidController};
