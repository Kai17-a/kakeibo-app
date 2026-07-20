SELECT
  id
  , created_at
  , updated_at
  , category_id
  , transaction_date
  , amount
  , description
FROM
  incomes
WHERE
  id = ?;
