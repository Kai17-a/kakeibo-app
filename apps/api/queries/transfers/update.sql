UPDATE transfers
SET
  transaction_date = ?, amount = ?, from_payment_method_id = ?
  , to_payment_method_id = ?, description = ?, updated_at = current_timestamp
WHERE
  id = ?
RETURNING
  id, created_at, updated_at, transaction_date, amount
  , from_payment_method_id, to_payment_method_id, description;
