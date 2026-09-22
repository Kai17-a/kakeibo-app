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
  , foreign_amount
  , currency_code
  , exchange_rate
  , exchange_rate_date
FROM
  expenses
ORDER BY transaction_date DESC, id ASC;
