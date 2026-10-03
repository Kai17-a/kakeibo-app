UPDATE payment_methods
SET
  name = ?
  , description = ?
  , initial_balance = ?
  , is_investment = ?
  , updated_at = current_timestamp
WHERE
  id = ?
RETURNING
  id, created_at, updated_at, name, description, initial_balance, is_investment
  , (
    SELECT
      coalesce(sum(cast(incomes.amount AS INTEGER)), 0)
    FROM
      incomes
    WHERE
      incomes.payment_method_id = payment_methods.id
  ) AS income_total
  , (
    SELECT
      coalesce(sum(cast(expenses.amount AS INTEGER)), 0)
    FROM
      expenses
    WHERE
      expenses.payment_method_id = payment_methods.id
  ) AS expense_total
  , (
    SELECT
      coalesce(sum(cast(transfers.amount AS INTEGER)), 0)
    FROM
      transfers
    WHERE
      transfers.to_payment_method_id = payment_methods.id
  ) AS transfer_in_total
  , (
    SELECT
      coalesce(sum(cast(transfers.amount AS INTEGER)), 0)
    FROM
      transfers
    WHERE
      transfers.from_payment_method_id = payment_methods.id
  ) AS transfer_out_total;
