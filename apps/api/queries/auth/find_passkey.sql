SELECT
  id
  , user_id
  , name
  , credential
  , created_at
  , last_used_at
FROM
  passkeys
WHERE
  id = ?1 AND user_id = ?2;
