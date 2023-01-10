use axum_boilerplate::{config::cli, utils::errors::CliResult};

#[tokio::main]
async fn main() -> CliResult<()> {
    cli::start().await
}
