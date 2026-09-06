SELECT
  id
  , created_at
  , updated_at
  , category_id
  , amount
FROM
  budgets
WHERE
  id = ?1;
