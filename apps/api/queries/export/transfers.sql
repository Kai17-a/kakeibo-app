SELECT
  transfers.transaction_date
  , transfers.amount
  , source.name AS from_payment_method_name
  , destination.name AS to_payment_method_name
  , transfers.description
FROM
  transfers
INNER JOIN payment_methods AS source ON transfers.from_payment_method_id = source.id
INNER JOIN payment_methods AS destination ON transfers.to_payment_method_id = destination.id
ORDER BY transfers.transaction_date, transfers.created_at, transfers.id;
