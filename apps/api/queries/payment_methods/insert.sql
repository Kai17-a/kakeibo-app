INSERT INTO payment_methods (name, description, initial_balance)
VALUES (?, ?, ?)
RETURNING id, created_at, updated_at, name, description, initial_balance
  , 0 AS income_total, 0 AS expense_total;
