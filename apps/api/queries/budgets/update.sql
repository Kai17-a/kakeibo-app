UPDATE budgets SET category_id = ?1, amount = ?2, updated_at = current_timestamp WHERE id = ?3 RETURNING id, created_at, updated_at, category_id, amount;
