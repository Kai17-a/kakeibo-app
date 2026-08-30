SELECT
  id
  , created_at
  , updated_at
  , category_id
  , transaction_date
  , amount
  , recurring_income_id
  , description
FROM
  incomes
WHERE
  id = ?;
