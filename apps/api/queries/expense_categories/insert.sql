INSERT INTO expense_categories (name, description, parent_category_id, display_order)
VALUES (
  ?, ?, ?3, (
    SELECT
      COALESCE(MAX(display_order), -1) + 1
    FROM
      expense_categories
    WHERE
      parent_category_id IS ?3
  )
)
RETURNING id, created_at, updated_at, name, description, parent_category_id, display_order;
