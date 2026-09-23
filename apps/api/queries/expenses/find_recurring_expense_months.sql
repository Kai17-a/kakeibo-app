SELECT DISTINCT
  strftime('%Y-%m', transaction_date) AS month

FROM
  expenses

WHERE
  recurring_expense_id = ?
ORDER BY month;
