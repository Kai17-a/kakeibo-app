INSERT INTO payment_methods (name, description, initial_balance, is_investment)
VALUES (?, ?, ?, ?)
RETURNING
  id, created_at, updated_at, name, description, initial_balance, is_investment
  , 0 AS income_total, 0 AS expense_total, 0 AS transfer_in_total, 0 AS transfer_out_total;
