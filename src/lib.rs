pub mod config;
pub mod emails;
pub mod handlers;
pub mod layers;
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

lazy_static! {
    pub static ref TEMPLATES: Result<Tera, tera::Error> = {
        let mut tera = Tera::new("templates/**/*")?;
        tera.autoescape_on(vec![".html", ".txt"]);
        Ok(tera)
    };
}
