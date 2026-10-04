UPDATE recurring_transfers
SET
  name = ?, amount = ?, payment_day = ?, start_date = ?, end_date = ?
  , from_payment_method_id = ?, to_payment_method_id = ?, is_active = ?
  , description = ?, updated_at = current_timestamp
WHERE
  id = ?
RETURNING
  id, created_at, updated_at, name, amount, payment_day, start_date, end_date
  , from_payment_method_id, to_payment_method_id, is_active, description;
