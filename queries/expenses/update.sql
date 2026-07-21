UPDATE expenses
SET
  transaction_date = ?
  , amount = ?
  , category_id = ?
  , payment_method_id = ?
  , recurring_expense_id = ?
  , description = ?
  , updated_at = current_timestamp
WHERE
  id = ?
RETURNING
  id, created_at, updated_at, transaction_date, amount, category_id
  , payment_method_id, recurring_expense_id, description;
