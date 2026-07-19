UPDATE recurring_expenses
SET
  name = :name
  , amount = :amount
  , payment_day = :payment_day
  , start_date = :start_date
  , end_date = :end_date
  , category_id = :category_id
  , payment_method_id = :payment_method_id
  , is_active = :is_active
  , description = :description
  , updated_at = current_timestamp
WHERE
  id = :id
RETURNING
  id, created_at, updated_at, name, amount, payment_day, start_date, end_date
  , category_id, payment_method_id, is_active, description;
