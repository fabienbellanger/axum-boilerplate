//! Forgotten password email module

use super::{send, Message, SmtpConfig};
use crate::errors::{AppError, AppResult};
use crate::{APP_NAME, TEMPLATES};
use serde::Serialize;
use tera::Context;

#[derive(Debug, Serialize)]
pub struct EmailContext {
    title: String,
    link: String,
}

impl EmailContext {
    /// New `EmailContext`
    pub fn new(base_url: String, token: String) -> Self {
        let link = format!("{}/{token}", base_url);

        Self {
            title: APP_NAME.to_owned(),
            link,
        }
    }
}

pub struct ForgottenPasswordEmail;

impl ForgottenPasswordEmail {
    /// Construct forgotten password email
    fn construct(base_url: String, token: String) -> AppResult<(String, String)> {
        let context = EmailContext::new(base_url, token);

        let html = TEMPLATES
            .render(
                "email/forgotten_password.html",
                &Context::from_serialize(&context).map_err(|err| AppError::InternalError {
                    message: err.to_string(),
                })?,
            )
            .map_err(|err| AppError::InternalError {
                message: err.to_string(),
            })?;

        let text = TEMPLATES
            .render(
                "email/forgotten_password.html",
                &Context::from_serialize(&context).map_err(|err| AppError::InternalError {
                    message: err.to_string(),
                })?,
            )
            .map_err(|err| AppError::InternalError {
                message: err.to_string(),
            })?;

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
        let (html, text) = Self::construct(base_url, token)?;

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
