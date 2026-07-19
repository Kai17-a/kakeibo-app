mod database;
mod utils;

use crate::database::migration::migrate;
use std::process::ExitCode;
use tracing::{error, info};

#[tokio::main]
async fn main() -> ExitCode {
    utils::logging::init();
    info!("Starting application");

    if let Err(error) = migrate().await {
        error!(%error, "Failed to initialize the database");
        return ExitCode::FAILURE;
    }

    info!("Application initialized successfully");
    ExitCode::SUCCESS
}
