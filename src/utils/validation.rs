//! HTTP request validation module

use super::errors::{AppError, AppErrorCode, AppResult};
use crate::app_error;
use serde_json::json;
use validator::Validate;

/// Validate the HTTP request parameters
pub fn validate_request_data<T: Validate>(data: &T) -> AppResult<()> {
    match data.validate() {
        Ok(_) => Ok(()),
        Err(errors) => Err(app_error!(AppErrorCode::BadRequest, json!(errors).to_string())),
    }
}
