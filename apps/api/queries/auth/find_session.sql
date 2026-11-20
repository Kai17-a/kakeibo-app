SELECT
  sessions.user_id
  , users.username
  , sessions.last_seen_at
  , sessions.expires_at
FROM
  sessions
INNER JOIN users ON sessions.user_id = users.id
WHERE
  sessions.token_hash = ?1;
