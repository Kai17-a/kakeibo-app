INSERT INTO income_categories (name, description)
VALUES (?, ?)
RETURNING id, created_at, updated_at, name, description;
