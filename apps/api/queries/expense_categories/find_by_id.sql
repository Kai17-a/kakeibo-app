SELECT
  id
  , created_at
  , updated_at
  , name
  , description
  , parent_category_id
  , display_order
FROM
  expense_categories
WHERE
  id = ?;
