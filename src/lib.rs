pub mod cli;
pub mod config;
pub mod databases;
pub mod emails;
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
extern crate lazy_static;

#[macro_use]
extern crate tracing;

extern crate chrono;
extern crate serde;

use tera::Tera;

/// Application name
pub const APP_NAME: &str = "Axum Boilerplate";
/// Application author name (for CLI)
pub const APP_AUTHOR: &str = "Fabien Bellanger";
/// Application version (for CLI)
pub const APP_VERSION: &str = "v0.1.0";

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html", ".txt"]);
        tera
    };
}
