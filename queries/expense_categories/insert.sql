INSERT INTO expense_categories (name, description)
VALUES (:name, :description)
RETURNING id, created_at, updated_at, name, description;
