UPDATE expenses
SET
  transaction_date = ?
  , amount = ?
  , category_id = ?
  , payment_method_id = ?
  , recurring_expense_id = ?
  , description = ?
  , foreign_amount = NULL
  , currency_code = NULL
  , exchange_rate = NULL
  , exchange_rate_date = NULL
  , updated_at = current_timestamp
WHERE
  id = ?
RETURNING
  id, created_at, updated_at, transaction_date, amount, category_id
  , payment_method_id, recurring_expense_id, description
  , foreign_amount, currency_code, exchange_rate, exchange_rate_date;
