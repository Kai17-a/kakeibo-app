SELECT
  EXISTS(
    SELECT
      1
    FROM
      expenses
    WHERE
      recurring_expense_id = ?1
      AND STRFTIME('%Y-%m', transaction_date) = ?2
  ) AS exists_flag;
