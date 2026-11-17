-- One row per registered passkey. `id` is the WebAuthn credential ID (base64url) and
-- `credential` the serialized webauthn-rs Passkey, which holds the public key and counter.
CREATE TABLE passkeys (
  id TEXT PRIMARY KEY
  , user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE
  , name TEXT NOT NULL
  , credential TEXT NOT NULL
  , created_at INTEGER NOT NULL
  , last_used_at INTEGER
);

CREATE INDEX passkeys_user_id ON passkeys (user_id);
