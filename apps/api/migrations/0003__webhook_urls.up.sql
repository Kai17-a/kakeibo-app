-- Webhook通知先URLテーブル
CREATE TABLE IF NOT EXISTS webhook_urls (
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
  )
  , created_at TEXT NOT NULL DEFAULT current_timestamp
  , updated_at TEXT NOT NULL DEFAULT current_timestamp
  , url TEXT NOT NULL -- noqa: RF04
  , description TEXT -- noqa: RF04
  , is_active INTEGER NOT NULL DEFAULT 1
);
