UPDATE payment_methods
SET
  name = ?
  , description = ?
  , initial_balance = ?
  , updated_at = current_timestamp
WHERE
  id = ?
RETURNING id, created_at, updated_at, name, description, initial_balance
  , (SELECT COALESCE(SUM(CAST(amount AS INTEGER)), 0) FROM incomes
     WHERE payment_method_id = payment_methods.id) AS income_total
  , (SELECT COALESCE(SUM(CAST(amount AS INTEGER)), 0) FROM expenses
     WHERE payment_method_id = payment_methods.id) AS expense_total;
