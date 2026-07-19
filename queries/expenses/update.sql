UPDATE expenses
SET
  transaction_date = :transaction_date
  , amount = :amount
  , category_id = :category_id
  , payment_method_id = :payment_method_id
  , recurring_expense_id = :recurring_expense_id
  , description = :description
  , updated_at = current_timestamp
WHERE
  id = :id
RETURNING
  id, created_at, updated_at, transaction_date, amount, category_id
  , payment_method_id, recurring_expense_id, description;
