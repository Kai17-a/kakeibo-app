SELECT
  id
  , created_at
  , updated_at
  , name
  , description
FROM
  payment_methods
WHERE
  id = ?;
