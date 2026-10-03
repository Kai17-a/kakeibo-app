ALTER TABLE payment_methods ADD COLUMN is_investment INTEGER NOT NULL DEFAULT 0;

CREATE TABLE transfers (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(
      hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-'
      || '4' || substr(hex(randomblob(2)), 2) || '-'
      || substr('AB89', 1 + (abs(random()) % 4), 1)
      || substr(hex(randomblob(2)), 2) || '-' || hex(randomblob(6))
    )
  )
  , created_at TEXT NOT NULL DEFAULT current_timestamp
  , updated_at TEXT NOT NULL DEFAULT current_timestamp
  , transaction_date TEXT NOT NULL
  , amount TEXT NOT NULL
  , from_payment_method_id TEXT NOT NULL REFERENCES payment_methods (id)
  , to_payment_method_id TEXT NOT NULL REFERENCES payment_methods (id)
  , description TEXT -- noqa: RF04
  , CHECK (cast(amount AS INTEGER) >= 1)
  , CHECK (from_payment_method_id <> to_payment_method_id)
);

CREATE INDEX idx_transfers_from_payment_method_id
ON transfers (from_payment_method_id);
CREATE INDEX idx_transfers_to_payment_method_id
ON transfers (to_payment_method_id);
CREATE INDEX idx_transfers_transaction_date ON transfers (transaction_date);
