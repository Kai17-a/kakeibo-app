INSERT INTO expense_categories (name, description, parent_category_id)
VALUES (?, ?, ?)
RETURNING id, created_at, updated_at, name, description, parent_category_id;
