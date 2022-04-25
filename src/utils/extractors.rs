//! Custom Axum extractors

use axum::http::header::HeaderValue;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
};

/// Request ID extractor from HTTP headers
pub struct ExtractRequestId(pub HeaderValue);

#[async_trait]
impl<B> FromRequest<B> for ExtractRequestId
where
    B: Send,
{
    type Rejection = ();

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match req.headers().get("x-request-id") {
            Some(id) => Ok(ExtractRequestId(id.clone())),
            _ => Ok(ExtractRequestId(HeaderValue::from_static(""))),
        }
    }
}
