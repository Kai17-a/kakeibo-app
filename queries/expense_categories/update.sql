UPDATE expense_categories
SET
  name = ?
  , description = ?
  , updated_at = current_timestamp
WHERE
  id = ?
RETURNING id, created_at, updated_at, name, description;
