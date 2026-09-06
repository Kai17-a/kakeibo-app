SELECT
  b.amount
  , c.name
  , COALESCE(SUM(CAST(e.amount AS INTEGER)), 0) AS spent_amount
FROM
  budgets AS b
INNER JOIN expense_categories AS c ON b.category_id = c.id
LEFT JOIN
  expenses AS e
  ON b.category_id = e.category_id AND STRFTIME('%Y-%m', e.transaction_date) = ?2
WHERE
  b.category_id = ?1
GROUP BY b.amount, c.name;
