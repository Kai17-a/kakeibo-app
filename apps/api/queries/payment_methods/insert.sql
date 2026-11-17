INSERT INTO payment_methods (name, description, initial_balance, is_investment, display_order)
VALUES (
  ?, ?, ?, ?, (
    SELECT
      COALESCE(MAX(display_order), -1) + 1
    FROM
      payment_methods
  )
)
RETURNING
  id, created_at, updated_at, name, description, initial_balance, is_investment, display_order
  , 0 AS income_total, 0 AS expense_total, 0 AS transfer_in_total, 0 AS transfer_out_total;
