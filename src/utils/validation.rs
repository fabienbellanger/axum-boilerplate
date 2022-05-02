//! HTTP request validation module

use crate::errors::{AppError, AppResult};
use serde_json::json;
use validator::Validate;

/// Validate the HTTP request parameters
pub fn validate_request_data<T: Validate>(data: &T) -> AppResult<()> {
    match data.validate() {
        Ok(_) => Ok(()),
        Err(errors) => Err(AppError::BadRequest {
            message: json!(errors).to_string(),
        }),
    }
}
