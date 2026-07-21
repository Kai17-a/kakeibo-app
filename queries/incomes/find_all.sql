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
ORDER BY transaction_date DESC, id ASC;
