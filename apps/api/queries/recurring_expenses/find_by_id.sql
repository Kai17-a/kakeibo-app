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
  , payment_method_id
  , is_active
  , is_variable
  , description
  , foreign_amount
  , currency_code
  , exchange_rate
FROM
  recurring_expenses
WHERE
  id = ?;
