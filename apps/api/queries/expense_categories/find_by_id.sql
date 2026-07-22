SELECT
  id
  , created_at
  , updated_at
  , name
  , description
FROM
  expense_categories
WHERE
  id = ?;
