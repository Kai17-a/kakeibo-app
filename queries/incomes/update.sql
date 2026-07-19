UPDATE incomes
SET
  category_id = :category_id
  , transaction_date = :transaction_date
  , amount = :amount
  , description = :description
  , updated_at = current_timestamp
WHERE
  id = :id
RETURNING id, created_at, updated_at, category_id, transaction_date, amount, description;
