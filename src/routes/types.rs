use core::fmt;
use std::fmt::Display;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sui_types::{crypto::Signature, transaction::TransactionData};

#[derive(Serialize, Deserialize, Debug)]
pub struct SponsoredTxArgs {
    pub signature: Signature,
    pub tx_data: TransactionData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRegistration {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResult {
    pub res: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiError {
    pub data: String,
}

impl std::error::Error for ApiError {} 

impl Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Api Error data: {}", self.data)
    }
}

impl IntoResponse for ApiResult {
    fn into_response(self) -> Response {
        (StatusCode::OK, self).into_response()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.data.to_string()).into_response()
    }
}

impl IntoResponse for UserRegistration {
    fn into_response(self) -> Response {
        (StatusCode::OK, self).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(value: sqlx::Error) -> Self {
        ApiError {
            data: value.to_string(),
        }
    }
}

impl From<argon2::password_hash::Error> for ApiError {
    fn from(value: argon2::password_hash::Error) -> Self {
        ApiError {
            data: value.to_string(),
        }
    }
}
