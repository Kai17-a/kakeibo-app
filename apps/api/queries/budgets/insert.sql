INSERT INTO budgets(category_id, amount) VALUES(?1, ?2) RETURNING id, created_at, updated_at, category_id, amount;
