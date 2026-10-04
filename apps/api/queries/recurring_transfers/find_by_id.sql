SELECT
  id
  , created_at
  , updated_at
  , name
  , amount
  , payment_day
  , start_date
  , end_date
  , from_payment_method_id
  , to_payment_method_id
  , is_active
  , description
FROM
  recurring_transfers
WHERE
  id = ?;
