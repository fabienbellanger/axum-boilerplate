//! CLI module

use crate::config::Config;
use crate::databases;
use crate::errors::{CliError, CliResult};
use crate::models::user::{PasswordScorer, PasswordStrength, User, UserCreation};
use crate::repositories::user::UserRepository;
use clap::{Parser, Subcommand};
use std::io::{self, Write};

#[derive(Parser)]
#[clap(
    name = crate::APP_NAME,
    version = crate::APP_VERSION,
    author = crate::APP_AUTHOR
)]
// #[clap(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start server
    #[clap(about = "Start Web server", long_about = None)]
    Serve,

    /// Register user
    #[clap(about = "Create a new user with ADMIN role", long_about = None)]
    Register {
        /// User lastname
        #[clap(
            required = true,
            short = 'l',
            long,
            value_name = "Lastname",
            num_args = 1,
            help = "Lastname"
        )]
        lastname: String,

        /// User firstname
        #[clap(
            required = true,
            short = 'f',
            long,
            value_name = "Firstname",
            num_args = 1,
            help = "Firstname"
        )]
        firstname: String,

        /// User username (email)
        #[clap(
            required = true,
            short = 'u',
            long,
            value_name = "Email",
            num_args = 1,
            help = "Username (email)"
        )]
        username: String,

        /// User password (at least 8 characters)
        #[clap(
            required = true,
            short = 'p',
            long,
            value_name = "Password",
            num_args = 1,
            help = "Password (at least 8 characters)"
        )]
        password: String,
    },
}

/// Start CLI
pub async fn start() -> CliResult<()> {
    let args = Cli::parse();
    match &args.commands {
        Commands::Serve => crate::server::start_server()
            .await
            .map_err(|err| CliError::ServerError(err.to_string())),
        Commands::Register {
            lastname,
            firstname,
            username,
            password,
        } => register(lastname, firstname, username, password).await,
    }
}

/// Command that creates a new user in database
async fn register(lastname: &str, firstname: &str, username: &str, password: &str) -> CliResult<()> {
    // Load configuration
    // ------------------
    let config = Config::from_env().map_err(|err| CliError::ConfigError(err.to_string()))?;

    // MySQL pool creation
    // -------------------
    let pool = databases::init(&config).await?;

    let lastname = lastname.trim();
    let firstname = firstname.trim();
    let username = username.trim();
    let password = password.trim();

    // Check arguments
    // ---------------
    if lastname.is_empty() {
        return Err(CliError::Error(String::from("empty lastname")));
    } else if firstname.is_empty() {
        return Err(CliError::Error(String::from("empty firstname")));
    } else if !mailchecker::is_valid(username) {
        return Err(CliError::Error(String::from("invalid email (username)")));
    } else if password.len() < 8 {
        return Err(CliError::Error(String::from(
            "invalid password (at least 8 characters)",
        )));
    } else if !PasswordScorer::valid(password, PasswordStrength::Strong) {
        // For a user with ADMIN role, the password must be strong enough
        return Err(CliError::Error(String::from("password is not enought strong")));
    }

    // User validation
    // ---------------
    print!(
        "\nFirstname: {}\nLastname:  {}\nUsername:  {}\nPassword:  {}\n\nAre you sure that user information are correct? (Y/n) ",
        firstname, lastname, username, password
    );
    io::stdout().flush().map_err(|err| CliError::Error(err.to_string()))?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|err| CliError::Error(err.to_string()))?;
    input = input.trim().to_string();

    if input.to_lowercase() != *"y" && !input.is_empty() {
        std::process::exit(1);
    }
    println!();

    // Add user in database
    // --------------------
    let user = UserCreation {
        lastname: lastname.to_string(),
        firstname: firstname.to_string(),
        username: username.to_string(),
        password: password.to_string(),
        roles: Some(String::from("ADMIN")),
    };
    let mut user = User::new(user);
    UserRepository::create(&pool, &mut user)
        .await
        .map_err(|err| CliError::DatabaseError(err.to_string()))?;

    Ok(())
}
