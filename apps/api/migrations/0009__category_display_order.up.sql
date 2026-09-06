ALTER TABLE expense_categories ADD COLUMN display_order INTEGER NOT NULL DEFAULT 0;
ALTER TABLE income_categories ADD COLUMN display_order INTEGER NOT NULL DEFAULT 0;

UPDATE expense_categories
SET display_order = (
  SELECT
    COUNT(*)
  FROM
    expense_categories AS sibling
  WHERE
    sibling.parent_category_id IS expense_categories.parent_category_id
    AND (
      sibling.name < expense_categories.name
      OR (sibling.name = expense_categories.name AND sibling.id < expense_categories.id)
    )
);

UPDATE income_categories
SET display_order = (
  SELECT
    COUNT(*)
  FROM
    income_categories AS sibling
  WHERE
    sibling.parent_category_id IS income_categories.parent_category_id
    AND (
      sibling.name < income_categories.name
      OR (sibling.name = income_categories.name AND sibling.id < income_categories.id)
    )
);
