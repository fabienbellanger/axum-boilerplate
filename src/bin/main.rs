use axum_boilerplate::{cli, errors::CliResult};

#[tokio::main]
async fn main() -> CliResult<()> {
    cli::start().await
}
