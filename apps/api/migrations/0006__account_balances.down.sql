DROP INDEX idx_expenses_payment_method_id;
DROP INDEX idx_incomes_payment_method_id;

ALTER TABLE incomes DROP COLUMN payment_method_id;
ALTER TABLE payment_methods DROP COLUMN initial_balance;
