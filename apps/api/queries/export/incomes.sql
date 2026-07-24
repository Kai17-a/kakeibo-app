SELECT
  incomes.transaction_date
  , incomes.amount
  , income_categories.name AS category_name
  , incomes.description
FROM
  incomes
INNER JOIN income_categories ON incomes.category_id = income_categories.id
ORDER BY incomes.transaction_date ASC, incomes.id ASC;
