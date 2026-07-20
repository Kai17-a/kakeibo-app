INSERT INTO incomes (category_id, transaction_date, amount, description)
VALUES (?, ?, ?, ?)
RETURNING id, created_at, updated_at, category_id, transaction_date, amount, description;
