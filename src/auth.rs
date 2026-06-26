use axum::http::HeaderMap;

use crate::errors::ApiError;

pub fn validate_api_key(headers: &HeaderMap, expected_api_key: &str) -> Result<(), ApiError> {
    let provided_api_key = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok());

    match provided_api_key {
        Some(key) if key == expected_api_key => Ok(()),
        _ => Err(ApiError::Unauthorized(
            "Invalid or missing API key".to_string(),
        )),
    }
}
