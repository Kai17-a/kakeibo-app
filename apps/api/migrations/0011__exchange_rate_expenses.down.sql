DROP TABLE exchange_rates;
ALTER TABLE expenses DROP COLUMN exchange_rate_date;
ALTER TABLE expenses DROP COLUMN exchange_rate;
ALTER TABLE expenses DROP COLUMN currency_code;
ALTER TABLE expenses DROP COLUMN foreign_amount;
