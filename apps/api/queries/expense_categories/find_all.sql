SELECT
  id
  , created_at
  , updated_at
  , name
  , description
  , parent_category_id
FROM
  expense_categories
ORDER BY name, id;
