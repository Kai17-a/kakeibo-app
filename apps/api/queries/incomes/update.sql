UPDATE incomes
SET
  category_id = ?
  , transaction_date = ?
  , amount = ?
  , payment_method_id = ?
  , recurring_income_id = ?
  , description = ?
  , updated_at = current_timestamp
WHERE
  id = ?
RETURNING
  id, created_at, updated_at, category_id, transaction_date, amount
  , payment_method_id, recurring_income_id, description;
