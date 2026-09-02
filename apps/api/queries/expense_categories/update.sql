UPDATE expense_categories
SET
  name = ?1
  , description = ?2
  , parent_category_id = ?3
  , display_order = CASE WHEN parent_category_id IS NOT ?3 THEN (SELECT COALESCE(MAX(display_order), -1) + 1 FROM expense_categories WHERE parent_category_id IS ?3) ELSE display_order END
  , updated_at = current_timestamp
WHERE
  id = ?4
RETURNING id, created_at, updated_at, name, description, parent_category_id, display_order;
