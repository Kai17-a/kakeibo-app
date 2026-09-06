SELECT
  id
  , created_at
  , updated_at
  , name
  , description
  , initial_balance
  , (
    SELECT
      COALESCE(SUM(CAST(incomes.amount AS INTEGER)), 0)
    FROM
      incomes
    WHERE
      incomes.payment_method_id = payment_methods.id
  ) AS income_total
  , (
    SELECT
      COALESCE(SUM(CAST(expenses.amount AS INTEGER)), 0)
    FROM
      expenses
    WHERE
      expenses.payment_method_id = payment_methods.id
  ) AS expense_total
FROM
  payment_methods
WHERE
  id = ?;
