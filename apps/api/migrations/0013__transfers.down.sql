DROP INDEX idx_transfers_transaction_date;
DROP INDEX idx_transfers_to_payment_method_id;
DROP INDEX idx_transfers_from_payment_method_id;
DROP TABLE transfers;
ALTER TABLE payment_methods DROP COLUMN is_investment;
