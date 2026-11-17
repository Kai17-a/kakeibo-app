ALTER TABLE payment_methods ADD COLUMN display_order INTEGER NOT NULL DEFAULT 0;

UPDATE payment_methods
SET display_order = (
  SELECT
    COUNT(*)
  FROM
    payment_methods AS sibling
  WHERE
    sibling.name < payment_methods.name
    OR (sibling.name = payment_methods.name AND sibling.id < payment_methods.id)
);
