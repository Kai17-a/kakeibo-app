INSERT INTO transfers (
  transaction_date, amount, from_payment_method_id, to_payment_method_id, description
)
VALUES (?, ?, ?, ?, ?)
RETURNING
  id, created_at, updated_at, transaction_date, amount
  , from_payment_method_id, to_payment_method_id, description;
