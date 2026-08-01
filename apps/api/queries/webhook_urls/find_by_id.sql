SELECT
  id
  , created_at
  , updated_at
  , url
  , description
  , is_active
FROM
  webhook_urls
WHERE
  id = ?;
