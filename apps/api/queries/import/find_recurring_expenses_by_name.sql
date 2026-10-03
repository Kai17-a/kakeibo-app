SELECT
  recurring.id
  , recurring.name
  , recurring.payment_day
  , recurring.category_id
  , category.name AS category
  , recurring.payment_method_id
  , payment_method.name AS payment_method
  , recurring.is_variable
FROM
  recurring_expenses AS recurring
INNER JOIN expense_categories AS category
  ON recurring.category_id = category.id
INNER JOIN payment_methods AS payment_method
  ON recurring.payment_method_id = payment_method.id
WHERE
  recurring.name = ?1;
