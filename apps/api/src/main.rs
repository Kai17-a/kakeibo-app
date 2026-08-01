use clap::Parser;
use kakeibo_app::{cli::Cli, database::migration::connect, router, utils};
use std::process::ExitCode;
use tokio::net::TcpListener;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::{Level, error, info};

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    utils::logging::init();
    info!("Starting application");

    let pool = match connect().await {
        Ok(pool) => pool,
        Err(error) => {
            error!(%error, "Failed to initialize the database");
            return ExitCode::FAILURE;
        }
    };

    let address = ("0.0.0.0", cli.port);
    let listener = match TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            error!(%error, "Failed to bind HTTP server");
            return ExitCode::FAILURE;
        }
    };
    info!(address = %format_args!("0.0.0.0:{}", cli.port), "Application initialized successfully");

    let app = router::incomes::create(pool.clone())
        .merge(router::expense_categories::create(pool.clone()))
        .merge(router::income_categories::create(pool.clone()))
        .merge(router::payment_methods::create(pool.clone()))
        .merge(router::recurring_expenses::create(pool.clone()))
        .merge(router::expenses::create(pool.clone()))
        .merge(router::export::create(pool.clone()))
        .merge(router::webhook_urls::create(pool.clone()))
        .merge(router::import::create(pool))
        .merge(router::health::create())
        .merge(router::redoc::create())
        .fallback_service(
            ServeDir::new("/app/public").fallback(ServeFile::new("/app/public/index.html")),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );
    if let Err(error) = axum::serve(listener, app).await {
        error!(%error, "HTTP server stopped unexpectedly");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
