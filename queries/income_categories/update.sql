UPDATE income_categories
SET
  name = :name
  , description = :description
  , updated_at = current_timestamp
WHERE
  id = :id
RETURNING id, created_at, updated_at, name, description;
