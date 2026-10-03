SELECT
  id
  , created_at
  , updated_at
  , transaction_date
  , amount
  , from_payment_method_id
  , to_payment_method_id
  , description
FROM
  transfers
WHERE
  id = ?;
