//! Forgotten password email module

use super::{send, Message, SmtpConfig};
use crate::errors::{AppError, AppErrorCode, AppResult};
use crate::{app_error, APP_NAME, TEMPLATES};
use serde::Serialize;
use tera::Context;

#[derive(Debug, Serialize)]
pub struct EmailContext {
    title: String,
    link: String,
}

impl EmailContext {
    /// New `EmailContext`
    pub fn new(base_url: String, token: String) -> AppResult<Self> {
        let link = format!("{}/{token}", base_url);

        match validator::validate_url(&link) {
            true => Ok(Self {
                title: format!("{} - Forgotten password", APP_NAME.to_owned()),
                link,
            }),
            false => Err(app_error!(
                AppErrorCode::InternalError,
                "cannot send password reset email because: invalid link"
            )),
        }
    }
}

pub struct ForgottenPasswordEmail;

impl ForgottenPasswordEmail {
    /// Construct forgotten password email body
    fn construct_body(base_url: String, token: String) -> AppResult<(String, String)> {
        let context = EmailContext::new(base_url, token)?;

        let html = TEMPLATES
            .render(
                "email/forgotten_password.html",
                &Context::from_serialize(&context)
                    .map_err(|err| app_error!(AppErrorCode::InternalError, err.to_string()))?,
            )
            .map_err(|err| app_error!(AppErrorCode::InternalError, err.to_string()))?;

        let text = TEMPLATES
            .render(
                "email/forgotten_password.txt",
                &Context::from_serialize(&context)
                    .map_err(|err| app_error!(AppErrorCode::InternalError, err.to_string()))?,
            )
            .map_err(|err| app_error!(AppErrorCode::InternalError, err.to_string()))?;

        Ok((html, text))
    }

    /// Send forgotten password email
    pub fn send(
        smtp_config: &SmtpConfig,
        base_url: String,
        email_from: String,
        email_to: String,
        token: String,
    ) -> AppResult<()> {
        let subject = format!("[{APP_NAME}] Forgotten password");
        let (html, text) = Self::construct_body(base_url, token)?;

        send(
            smtp_config,
            Message {
                from: email_from,
                to_list: vec![email_to],
                subject,
                text_body: text,
                html_body: html,
            },
        )
    }
}
