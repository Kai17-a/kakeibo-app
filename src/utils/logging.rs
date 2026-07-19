use tracing_subscriber::{EnvFilter, fmt, fmt::format::FmtSpan};

const DEFAULT_LOG_FILTER: &str = "info";

pub fn init() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER));

    fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();
}
