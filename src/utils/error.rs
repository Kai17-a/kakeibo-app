use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub struct AppError {
    message: String,
}

impl AppError {
    pub fn context(context: &str, error: impl Error) -> Self {
        Self {
            message: format!("{context}: {error}"),
        }
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
