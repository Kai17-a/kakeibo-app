INSERT INTO payment_methods (name, description)
VALUES (?, ?)
RETURNING id, created_at, updated_at, name, description;
