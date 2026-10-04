CREATE TABLE recurring_transfers (
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
  , name TEXT NOT NULL
  , amount TEXT NOT NULL
  , payment_day INTEGER NOT NULL
  , start_date TEXT NOT NULL
  , end_date TEXT
  , from_payment_method_id TEXT NOT NULL REFERENCES payment_methods (id)
  , to_payment_method_id TEXT NOT NULL REFERENCES payment_methods (id)
  , is_active INTEGER NOT NULL
  , description TEXT -- noqa: RF04
  , CHECK (cast(amount AS INTEGER) >= 1)
  , CHECK (payment_day BETWEEN 1 AND 31)
  , CHECK (from_payment_method_id <> to_payment_method_id)
);

CREATE INDEX idx_recurring_transfers_from_payment_method_id
ON recurring_transfers (from_payment_method_id);
CREATE INDEX idx_recurring_transfers_to_payment_method_id
ON recurring_transfers (to_payment_method_id);

ALTER TABLE transfers
ADD COLUMN recurring_transfer_id TEXT REFERENCES recurring_transfers (id);
CREATE INDEX idx_transfers_recurring_transfer_id
ON transfers (recurring_transfer_id);
