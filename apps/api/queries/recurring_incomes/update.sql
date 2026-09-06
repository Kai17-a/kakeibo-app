UPDATE recurring_incomes
SET
  name = ?, amount = ?, payment_day = ?, start_date = ?, end_date = ?, category_id = ?
  , is_active = ?, is_variable = ?, description = ?, updated_at = current_timestamp
WHERE
  id = ?
RETURNING
  id, created_at, updated_at, name, amount, payment_day, start_date, end_date
  , category_id, is_active, is_variable, description;
