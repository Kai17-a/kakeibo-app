UPDATE payment_methods
SET
  name = ?
  , description = ?
  , updated_at = current_timestamp
WHERE
  id = ?
RETURNING id, created_at, updated_at, name, description;
