INSERT INTO recurring_transfers (
  name, amount, payment_day, start_date, end_date, from_payment_method_id
  , to_payment_method_id, is_active, description
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
RETURNING
  id, created_at, updated_at, name, amount, payment_day, start_date, end_date
  , from_payment_method_id, to_payment_method_id, is_active, description;
