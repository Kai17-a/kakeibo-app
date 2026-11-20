UPDATE sessions SET
  last_seen_at = ?2
  , expires_at = ?3
WHERE
  token_hash = ?1;
