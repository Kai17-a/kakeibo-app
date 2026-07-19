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
  , description
FROM
  recurring_expenses
ORDER BY start_date DESC, id;
