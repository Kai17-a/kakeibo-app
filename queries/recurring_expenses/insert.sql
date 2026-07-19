INSERT INTO recurring_expenses (
  name, amount, payment_day, start_date, end_date, category_id, payment_method_id
  , is_active, description
)
VALUES (
  :name, :amount, :payment_day, :start_date, :end_date, :category_id
  , :payment_method_id, :is_active, :description
)
RETURNING
  id, created_at, updated_at, name, amount, payment_day, start_date, end_date
  , category_id, payment_method_id, is_active, description;
