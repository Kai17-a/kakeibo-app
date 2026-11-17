UPDATE users SET
  password_hash = ?2
  , updated_at = current_timestamp
WHERE
  username = ?1;
