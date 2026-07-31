INSERT INTO recurring_expenses (
  name, amount, payment_day, start_date, end_date, category_id, payment_method_id
  , is_active, is_variable, description
)
VALUES (
  ?, ?, ?, ?, ?, ?
  , ?, ?, ?, ?
)
RETURNING
  id, created_at, updated_at, name, amount, payment_day, start_date, end_date
  , category_id, payment_method_id, is_active, is_variable, description;
