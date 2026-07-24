SELECT
  expenses.transaction_date
  , expenses.amount
  , expense_categories.name AS category_name
  , payment_methods.name AS payment_method_name
  , expenses.description
FROM
  expenses
INNER JOIN expense_categories ON expenses.category_id = expense_categories.id
INNER JOIN payment_methods ON expenses.payment_method_id = payment_methods.id
ORDER BY expenses.transaction_date ASC, expenses.id ASC;
