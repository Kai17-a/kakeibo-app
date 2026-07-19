SELECT
  id
  , created_at
  , updated_at
  , transaction_date
  , amount
  , category_id
  , payment_method_id
  , recurring_expense_id
  , description
FROM
  expenses
WHERE
  id = :id;
