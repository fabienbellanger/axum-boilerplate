pub mod cli;
pub mod config;
pub mod database;
pub mod errors;
pub mod handlers;
pub mod layers;
pub mod logger;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod server;
pub mod utils;

#[macro_use]
extern crate tracing;

extern crate chrono;
extern crate serde;

pub const APP_NAME: &str = "Axum Boilerplate";
pub const APP_AUTHOR: &str = "Fabien Bellanger";
pub const APP_VERSION: &str = "v0.1.0";
