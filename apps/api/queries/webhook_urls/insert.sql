INSERT INTO webhook_urls (
  url, description, is_active
)
VALUES (
  ?, ?, ?
)
RETURNING
  id, created_at, updated_at, url, description, is_active;
