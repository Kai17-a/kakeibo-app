use chrono::{Duration, NaiveDate};
use reqwest::Client;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::{
    collections::HashMap, error::Error, fmt, future::Future, pin::Pin, sync::Arc,
    time::Duration as StdDuration,
};

const BASE_CURRENCY: &str = "USD";
const QUOTE_CURRENCY: &str = "JPY";
const SOURCE: &str = "frankfurter.dev";

#[derive(Debug, Clone, PartialEq)]
pub struct RateQuote {
    pub rate: f64,
    pub effective_date: String,
}

#[derive(Debug)]
pub enum ExchangeRateError {
    InvalidTargetMonth(String),
    Provider(String),
    Database(sqlx::Error),
    NoRateFound { target_date: String },
}

impl fmt::Display for ExchangeRateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTargetMonth(month) => write!(f, "invalid target month: {month}"),
            Self::Provider(message) => write!(f, "exchange-rate provider failed: {message}"),
            Self::Database(error) => write!(f, "exchange-rate cache failed: {error}"),
            Self::NoRateFound { target_date } => {
                write!(
                    f,
                    "no exchange rate found within seven days before {target_date}"
                )
            }
        }
    }
}

impl Error for ExchangeRateError {}

pub trait ExchangeRateProvider: Send + Sync {
    fn fetch<'a>(
        &'a self,
        date: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<RateQuote>, ExchangeRateError>> + Send + 'a>>;
}

#[derive(Clone)]
pub struct FrankfurterProvider {
    client: Client,
    base_url: String,
}

impl FrankfurterProvider {
    pub fn new() -> Result<Self, ExchangeRateError> {
        Self::with_base_url("https://api.frankfurter.dev")
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Result<Self, ExchangeRateError> {
        let client = Client::builder()
            .timeout(StdDuration::from_secs(10))
            .build()
            .map_err(|error| ExchangeRateError::Provider(error.to_string()))?;
        Ok(Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_owned(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct FrankfurterResponse {
    base: String,
    date: String,
    rates: HashMap<String, f64>,
}

impl ExchangeRateProvider for FrankfurterProvider {
    fn fetch<'a>(
        &'a self,
        date: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<RateQuote>, ExchangeRateError>> + Send + 'a>>
    {
        Box::pin(async move {
            let response = self
                .client
                .get(format!("{}/v1/{date}?from=USD&to=JPY", self.base_url))
                .send()
                .await
                .map_err(|error| ExchangeRateError::Provider(error.to_string()))?;
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }
            if !response.status().is_success() {
                return Err(ExchangeRateError::Provider(format!(
                    "HTTP status {}",
                    response.status()
                )));
            }
            let body: FrankfurterResponse = response
                .json()
                .await
                .map_err(|error| ExchangeRateError::Provider(error.to_string()))?;
            let rate = body.rates.get(QUOTE_CURRENCY).copied().ok_or_else(|| {
                ExchangeRateError::Provider("response did not contain JPY".to_owned())
            })?;
            if body.base != BASE_CURRENCY || body.date != date || !rate.is_finite() || rate <= 0.0 {
                return Err(ExchangeRateError::Provider(
                    "response contained invalid exchange-rate data".to_owned(),
                ));
            }
            Ok(Some(RateQuote {
                rate,
                effective_date: body.date,
            }))
        })
    }
}

#[derive(Clone)]
pub struct ExchangeRateService {
    pool: SqlitePool,
    provider: Arc<dyn ExchangeRateProvider>,
}

impl ExchangeRateService {
    pub fn new(pool: SqlitePool) -> Result<Self, ExchangeRateError> {
        Ok(Self {
            pool,
            provider: Arc::new(FrankfurterProvider::new()?),
        })
    }

    pub fn with_provider(pool: SqlitePool, provider: Arc<dyn ExchangeRateProvider>) -> Self {
        Self { pool, provider }
    }

    pub async fn resolve(&self, month: &str) -> Result<RateQuote, ExchangeRateError> {
        let target = NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d")
            .map_err(|_| ExchangeRateError::InvalidTargetMonth(month.to_owned()))?;
        let target_date = target.to_string();
        if let Some(cached) = self.cached(&target_date).await? {
            return Ok(cached);
        }

        for days_back in 0..=7 {
            let date = target - Duration::days(days_back);
            let date_string = date.to_string();
            match self.provider.fetch(&date_string).await? {
                Some(quote) => {
                    sqlx::query(
                        "INSERT INTO exchange_rates (target_date, base_currency, quote_currency, rate, effective_date, source) VALUES (?, ?, ?, ?, ?, ?)",
                    )
                    .bind(&target_date)
                    .bind(BASE_CURRENCY)
                    .bind(QUOTE_CURRENCY)
                    .bind(quote.rate.to_string())
                    .bind(&quote.effective_date)
                    .bind(SOURCE)
                    .execute(&self.pool)
                    .await
                    .map_err(ExchangeRateError::Database)?;
                    return Ok(quote);
                }
                None => continue,
            }
        }
        Err(ExchangeRateError::NoRateFound { target_date })
    }

    async fn cached(&self, target_date: &str) -> Result<Option<RateQuote>, ExchangeRateError> {
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT rate, effective_date FROM exchange_rates WHERE target_date = ? AND base_currency = ? AND quote_currency = ?",
        )
        .bind(target_date)
        .bind(BASE_CURRENCY)
        .bind(QUOTE_CURRENCY)
        .fetch_optional(&self.pool)
        .await
        .map_err(ExchangeRateError::Database)?;
        row.map(|(rate, effective_date)| {
            rate.parse::<f64>()
                .map(|rate| RateQuote {
                    rate,
                    effective_date,
                })
                .map_err(|_| ExchangeRateError::Provider("cached rate is invalid".to_owned()))
        })
        .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::{Arc, Mutex};

    struct FakeProvider {
        calls: Arc<Mutex<Vec<String>>>,
        responses: HashMap<String, Option<RateQuote>>,
    }
    impl ExchangeRateProvider for FakeProvider {
        fn fetch<'a>(
            &'a self,
            date: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<Option<RateQuote>, ExchangeRateError>> + Send + 'a>>
        {
            Box::pin(async move {
                self.calls.lock().unwrap().push(date.to_owned());
                Ok(self.responses.get(date).cloned().flatten())
            })
        }
    }
    async fn pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE exchange_rates (target_date TEXT NOT NULL, base_currency TEXT NOT NULL, quote_currency TEXT NOT NULL, rate TEXT NOT NULL, effective_date TEXT NOT NULL, source TEXT NOT NULL, fetched_at TEXT NOT NULL DEFAULT current_timestamp, PRIMARY KEY (target_date, base_currency, quote_currency))").execute(&pool).await.unwrap();
        pool
    }
    fn provider(
        responses: HashMap<String, Option<RateQuote>>,
    ) -> (Arc<Mutex<Vec<String>>>, Arc<dyn ExchangeRateProvider>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        (calls.clone(), Arc::new(FakeProvider { calls, responses }))
    }
    #[tokio::test]
    async fn falls_back_to_previous_available_business_day() {
        let mut responses = HashMap::new();
        responses.insert("2026-09-01".into(), None);
        responses.insert(
            "2026-08-31".into(),
            Some(RateQuote {
                rate: 147.2,
                effective_date: "2026-08-31".into(),
            }),
        );
        let (calls, provider) = provider(responses);
        let service = ExchangeRateService::with_provider(pool().await, provider);
        assert_eq!(
            service.resolve("2026-09").await.unwrap().effective_date,
            "2026-08-31"
        );
        assert_eq!(calls.lock().unwrap().len(), 2);
    }
    #[tokio::test]
    async fn uses_cache_without_calling_provider_again() {
        let pool = pool().await;
        sqlx::query("INSERT INTO exchange_rates VALUES ('2026-09-01','USD','JPY','147.2','2026-09-01','test',current_timestamp)").execute(&pool).await.unwrap();
        let (calls, provider) = provider(HashMap::new());
        let service = ExchangeRateService::with_provider(pool, provider);
        assert_eq!(service.resolve("2026-09").await.unwrap().rate, 147.2);
        assert!(calls.lock().unwrap().is_empty());
    }
    #[tokio::test]
    async fn returns_failure_when_no_rate_exists() {
        let (calls, provider) = provider(HashMap::new());
        let service = ExchangeRateService::with_provider(pool().await, provider);
        assert!(matches!(
            service.resolve("2026-09").await,
            Err(ExchangeRateError::NoRateFound { .. })
        ));
        assert_eq!(calls.lock().unwrap().len(), 8);
    }
}
