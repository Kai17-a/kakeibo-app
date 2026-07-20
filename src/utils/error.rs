use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub struct AppError {
    message: String,
    status: StatusCode,
}

impl AppError {
    pub fn context(context: &str, error: impl Error) -> Self {
        Self {
            message: format!("{context}: {error}"),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn bad_request(message: &str) -> Self {
        Self {
            message: message.to_owned(),
            status: StatusCode::BAD_REQUEST,
        }
    }

    pub fn not_found(resource: &str, id: &str) -> Self {
        Self {
            message: format!("{resource} '{id}' was not found"),
            status: StatusCode::NOT_FOUND,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::context("Database operation failed", error)
    }
}

#[derive(Serialize)]
struct ErrorBody {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                message: self.message,
            }),
        )
            .into_response()
    }
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for AppError {}

#[cfg(test)]
mod tests {
    use super::AppError;
    use std::io;

    #[test]
    fn includes_context_and_source_error() {
        let error = AppError::context(
            "Failed to read configuration",
            io::Error::other("not found"),
        );

        assert_eq!(error.to_string(), "Failed to read configuration: not found");
    }
}
