CREATE TABLE recurring_incomes (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4'
      || substr(hex(randomblob(2)), 2) || '-'
      || substr('AB89', 1 + (abs(random()) % 4), 1)
      || substr(hex(randomblob(2)), 2) || '-' || hex(randomblob(6)))
  )
  , created_at TEXT NOT NULL DEFAULT current_timestamp
  , updated_at TEXT NOT NULL DEFAULT current_timestamp
  , name TEXT NOT NULL
  , amount TEXT NOT NULL
  , payment_day INTEGER NOT NULL
  , start_date TEXT NOT NULL
  , end_date TEXT
  , category_id TEXT NOT NULL REFERENCES income_categories (id)
  , is_active INTEGER NOT NULL
  , is_variable INTEGER NOT NULL DEFAULT 0
  , description TEXT
);

ALTER TABLE incomes
  ADD COLUMN recurring_income_id TEXT REFERENCES recurring_incomes (id);
