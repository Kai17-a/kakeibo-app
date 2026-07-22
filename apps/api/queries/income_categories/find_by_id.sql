SELECT
  id
  , created_at
  , updated_at
  , name
  , description
FROM
  income_categories
WHERE
  id = ?;
