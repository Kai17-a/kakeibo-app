UPDATE recurring_expenses
SET
  name = ?
  , amount = ?
  , payment_day = ?
  , start_date = ?
  , end_date = ?
  , category_id = ?
  , payment_method_id = ?
  , is_active = ?
  , description = ?
  , updated_at = current_timestamp
WHERE
  id = ?
RETURNING
  id, created_at, updated_at, name, amount, payment_day, start_date, end_date
  , category_id, payment_method_id, is_active, description;
