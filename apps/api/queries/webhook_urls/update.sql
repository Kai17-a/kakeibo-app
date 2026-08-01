UPDATE webhook_urls
SET
  url = ?
  , description = ?
  , is_active = ?
  , updated_at = current_timestamp
WHERE
  id = ?
RETURNING
  id, created_at, updated_at, url, description, is_active;
