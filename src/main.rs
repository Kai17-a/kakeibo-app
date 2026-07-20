use kakeibo_app::{database::migration::connect, router, utils};
use std::process::ExitCode;
use tokio::net::TcpListener;
use tracing::{error, info};

#[tokio::main]
async fn main() -> ExitCode {
    utils::logging::init();
    info!("Starting application");

    let pool = match connect().await {
        Ok(pool) => pool,
        Err(error) => {
            error!(%error, "Failed to initialize the database");
            return ExitCode::FAILURE;
        }
    };

    let address = ("0.0.0.0", 8000);
    let listener = match TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            error!(%error, "Failed to bind HTTP server");
            return ExitCode::FAILURE;
        }
    };
    info!(
        address = "0.0.0.0:8000",
        "Application initialized successfully"
    );

    if let Err(error) = axum::serve(listener, router::incomes::create(pool)).await {
        error!(%error, "HTTP server stopped unexpectedly");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
