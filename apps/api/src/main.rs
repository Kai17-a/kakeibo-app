use clap::Parser;
use kakeibo_app::auth::{
    self, AuthState,
    config::{AuthConfig, AuthMode},
};
use kakeibo_app::service::backup::RestoreConfig;
use kakeibo_app::{
    cli::{Cli, Command},
    database::migration::connect,
    router, utils,
};
use std::{process::ExitCode, time::Duration};
use tokio::net::TcpListener;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::{Level, error, info, warn};

const EXPIRED_SESSION_CLEANUP_INTERVAL: Duration = Duration::from_secs(60 * 60);

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    if let Some(Command::Auth(command)) = cli.command {
        return auth::commands::run(command).await;
    }
    utils::logging::init();
    info!("Starting application");

    let auth_state = match auth_state().await {
        Ok(state) => state,
        Err(error) => {
            error!(%error, "Failed to initialize authentication");
            return ExitCode::FAILURE;
        }
    };

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

    let app = router::auth::create(auth_state.clone())
        .merge(router::incomes::create(pool.clone()))
        .merge(router::backup::create(
            pool.clone(),
            RestoreConfig::production(),
        ))
        .merge(router::budgets::create(pool.clone()))
        .merge(router::expense_categories::create(pool.clone()))
        .merge(router::income_categories::create(pool.clone()))
        .merge(router::payment_methods::create(pool.clone()))
        .merge(router::recurring_expenses::create(pool.clone()))
        .merge(router::recurring_incomes::create(pool.clone()))
        .merge(router::recurring_transfers::create(pool.clone()))
        .merge(router::expenses::create(pool.clone()))
        .merge(router::transfers::create(pool.clone()))
        .merge(router::export::create(pool.clone()))
        .merge(router::import::create(pool))
        .merge(router::health::create())
        .merge(router::redoc::create())
        .fallback_service(
            ServeDir::new("/app/public").fallback(ServeFile::new("/app/public/index.html")),
        );
    let app = auth::protect(app, auth_state).layer(
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

async fn auth_state() -> Result<AuthState, Box<dyn std::error::Error>> {
    let config = AuthConfig::from_env()?;
    info!(mode = config.mode.as_str(), "Authentication configured");
    if config.mode == AuthMode::None {
        return Ok(AuthState::disabled());
    }

    let pool = auth::database::connect(&config.database_path).await?;
    let state = AuthState::local(&config, pool);
    let service = state
        .service
        .clone()
        .expect("local authentication has a service");
    if service.count_users().await? == 0 {
        warn!(
            "No users are registered, so every API request will be rejected. \
             Add one with: kakeibo-app auth add-user <username>"
        );
    }
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(EXPIRED_SESSION_CLEANUP_INTERVAL);
        loop {
            interval.tick().await;
            match service.delete_expired_sessions(auth::now()).await {
                Ok(0) => {}
                Ok(count) => info!(count, "Deleted expired sessions"),
                Err(error) => warn!(%error, "Failed to delete expired sessions"),
            }
        }
    });
    Ok(state)
}
