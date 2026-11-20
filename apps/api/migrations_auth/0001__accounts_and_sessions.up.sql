CREATE TABLE users (
  id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16))))
  , created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
  , updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
  , username TEXT NOT NULL UNIQUE COLLATE nocase
  , password_hash TEXT NOT NULL
);

-- Only a SHA-256 digest of each session token is stored, so a leaked database cannot be used
-- to hijack sessions. Times are Unix epoch seconds.
CREATE TABLE sessions (
  token_hash TEXT PRIMARY KEY
  , user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE
  , created_at INTEGER NOT NULL
  , last_seen_at INTEGER NOT NULL
  , expires_at INTEGER NOT NULL
);

CREATE INDEX sessions_user_id ON sessions (user_id);
CREATE INDEX sessions_expires_at ON sessions (expires_at);
