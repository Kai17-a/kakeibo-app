-- 支出カテゴリテーブル
CREATE TABLE expense_categories (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(
      hex(randomblob(4))
      || '-'
      || hex(randomblob(2))
      || '-'
      || '4'
      || substr(hex(randomblob(2)), 2)
      || '-'
      || substr('AB89', 1 + (abs(random()) % 4), 1)
      || substr(hex(randomblob(2)), 2)
      || '-'
      || hex(randomblob(6))
    )
  ),
  created_at TEXT NOT NULL DEFAULT current_timestamp,
  updated_at TEXT NOT NULL DEFAULT current_timestamp,
  name TEXT NOT NULL, -- noqa: RF04
  description TEXT -- noqa: RF04
);

-- 支払方法テーブル
CREATE TABLE payment_methods (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(
      hex(randomblob(4))
      || '-'
      || hex(randomblob(2))
      || '-'
      || '4'
      || substr(hex(randomblob(2)), 2)
      || '-'
      || substr('AB89', 1 + (abs(random()) % 4), 1)
      || substr(hex(randomblob(2)), 2)
      || '-'
      || hex(randomblob(6))
    )
  ),
  created_at TEXT NOT NULL DEFAULT current_timestamp,
  updated_at TEXT NOT NULL DEFAULT current_timestamp,
  name TEXT NOT NULL, -- noqa: RF04
  description TEXT -- noqa: RF04
);

-- 定期支出設定テーブル
CREATE TABLE recurring_expenses (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(
      hex(randomblob(4))
      || '-'
      || hex(randomblob(2))
      || '-'
      || '4'
      || substr(hex(randomblob(2)), 2)
      || '-'
      || substr('AB89', 1 + (abs(random()) % 4), 1)
      || substr(hex(randomblob(2)), 2)
      || '-'
      || hex(randomblob(6))
    )
  ),
  created_at TEXT NOT NULL DEFAULT current_timestamp,
  updated_at TEXT NOT NULL DEFAULT current_timestamp,
  name TEXT NOT NULL, -- noqa: RF04
  amount TEXT NOT NULL, -- 言語側でDecimal管理
  payment_day INTEGER NOT NULL,
  start_date TEXT NOT NULL,
  end_date TEXT,
  category_id TEXT NOT NULL REFERENCES expense_categories (id),
  payment_method_id TEXT NOT NULL REFERENCES payment_methods (id),
  is_active INTEGER NOT NULL,
  description TEXT -- noqa: RF04
);

-- 支出テーブル
CREATE TABLE expenses (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(
      hex(randomblob(4))
      || '-'
      || hex(randomblob(2))
      || '-'
      || '4'
      || substr(hex(randomblob(2)), 2)
      || '-'
      || substr('AB89', 1 + (abs(random()) % 4), 1)
      || substr(hex(randomblob(2)), 2)
      || '-'
      || hex(randomblob(6))
    )
  ),
  created_at TEXT NOT NULL DEFAULT current_timestamp,
  updated_at TEXT NOT NULL DEFAULT current_timestamp,
  transaction_date TEXT NOT NULL,
  amount TEXT NOT NULL, -- 言語側でDecimal管理
  category_id TEXT NOT NULL REFERENCES expense_categories (id),
  payment_method_id TEXT NOT NULL REFERENCES payment_methods (id),
  recurring_expense_id TEXT REFERENCES recurring_expenses (id),
  description TEXT -- noqa: RF04
);
