ALTER TABLE expense_categories
ADD COLUMN parent_category_id TEXT REFERENCES expense_categories (id);

ALTER TABLE income_categories
ADD COLUMN parent_category_id TEXT REFERENCES income_categories (id);
