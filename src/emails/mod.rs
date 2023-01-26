//! Email helper module

pub mod forgotten_password;

use crate::app_error;
use crate::utils::errors::{AppError, AppErrorCode, AppResult};
use lettre::message::{header, MultiPart, SinglePart};
use lettre::{SmtpTransport, Transport};
use std::time::Duration;

pub struct Message {
    pub from: String,
    pub to_list: Vec<String>,
    pub subject: String,
    pub text_body: String,
    pub html_body: String,
}

pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub timeout: u64,
}

/// Initialize SMTP client
fn init(config: &SmtpConfig) -> SmtpTransport {
    let host = &config.host[..];
    let port = config.port;
    let timeout = match config.timeout {
        0 => None,
        t => Some(Duration::from_secs(t)),
    };

    SmtpTransport::builder_dangerous(host)
        .port(port)
        .timeout(timeout)
        .build()
}

/// Sends an email
pub fn send(config: &SmtpConfig, message: Message) -> AppResult<()> {
    let mailer = init(config);

    let mut email_builder = lettre::Message::builder()
        .subject(message.subject)
        .from(message.from.parse().map_err(|_| {
            app_error!(
                AppErrorCode::InternalError,
                "cannot send password reset email because: invalid from email".to_string()
            )
        })?);

    // Add destination emails
    for to in message.to_list {
        email_builder = email_builder.to(to.parse().map_err(|_| {
            app_error!(
                AppErrorCode::InternalError,
                "cannot send password reset email because: invalid to email".to_string()
            )
        })?)
    }

    let email = email_builder
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(message.text_body),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(message.html_body),
                ),
        )
        .map_err(|err| {
            app_error!(
                AppErrorCode::InternalError,
                format!("cannot send password reset email because: {err}")
            )
        })?;

    mailer.send(&email).map_err(|err| {
        app_error!(
            AppErrorCode::InternalError,
            format!("SMTP Error when sending password reset email: {err}")
        )
    })?;

    Ok(())
}
