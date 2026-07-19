INSERT INTO income_categories (name, description)
VALUES (:name, :description)
RETURNING id, created_at, updated_at, name, description;
