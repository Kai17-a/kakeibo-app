SELECT
  id
  , created_at
  , updated_at
  , name
  , amount
  , payment_day
  , start_date
  , end_date
  , category_id
  , is_active
  , is_variable
  , description
FROM
  recurring_incomes
WHERE
  id = ?;
