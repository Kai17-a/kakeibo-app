ALTER TABLE expenses ADD COLUMN foreign_amount TEXT;
ALTER TABLE expenses ADD COLUMN currency_code TEXT;
ALTER TABLE expenses ADD COLUMN exchange_rate TEXT;
ALTER TABLE expenses ADD COLUMN exchange_rate_date TEXT;

CREATE TABLE exchange_rates (
  target_date TEXT NOT NULL
  , base_currency TEXT NOT NULL
  , quote_currency TEXT NOT NULL
  , rate TEXT NOT NULL
  , effective_date TEXT NOT NULL
  , source TEXT NOT NULL
  , fetched_at TEXT NOT NULL DEFAULT current_timestamp
  , PRIMARY KEY (target_date, base_currency, quote_currency)
);
