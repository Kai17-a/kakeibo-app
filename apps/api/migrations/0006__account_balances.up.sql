ALTER TABLE payment_methods ADD COLUMN initial_balance TEXT;
ALTER TABLE incomes ADD COLUMN payment_method_id TEXT REFERENCES payment_methods (id);

CREATE INDEX idx_incomes_payment_method_id ON incomes (payment_method_id);
CREATE INDEX idx_expenses_payment_method_id ON expenses (payment_method_id);
