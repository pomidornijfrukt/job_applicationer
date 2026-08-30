use axum::{Json, response::{IntoResponse, Response}};
use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Data(#[from] DataError),

    #[error(transparent)]
    Parsing(#[from] ParsingError),

    #[error("The server couldn't process the request, please try with a different response/later")]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("couldn't get data")]
    GetError(#[source] sqlx::Error),

    #[error("couldn't insert data")]
    InsertError(#[source] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("no data found {details}")]
    NoDataError {
        details: String
    },

    #[error("invalid format: expected {expected}. actual error: {actual}")]
    InvalidFormat {
        expected: &'static str,
        actual: String,
    },
}

#[derive(Serialize)]
struct ErrorResponse {
    code: &'static str,
    message: &'static str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // `self` still contains all the detailed information.
        tracing::error!(error = ?self, "request failed");

        let (status, body) = match &self {
            AppError::Parsing(ParsingError::InvalidFormat { .. }) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    code: "INVALID_FORMAT",
                    message: "Invalid format provided in the request",
                },
            ),

            AppError::Parsing(ParsingError::NoDataError { .. }) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    code: "NO_DATA_FOUND",
                    message: "No data found for the given request",
                },
            ),

            AppError::Data(DataError::GetError { .. }) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    code: "COULD_NOT_GET_DATA",
                    message: "Could not get data",
                },
            ),

            AppError::Data(DataError::InsertError { .. }) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    code: "COULD_NOT_INSERT_DATA",
                    message: "Could not insert data",
                },
            ),

            AppError::Internal(anyhow::Error { .. }) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    code: "INTERNAL_SERVER_ERROR",
                    message: "Service doesn't seems to be healthy, try again later",
                },
            ),
        };

        (status, Json(body)).into_response()
    }
}