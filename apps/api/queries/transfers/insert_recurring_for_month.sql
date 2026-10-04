WITH RECURSIVE recurring_months (
  recurring_transfer_id, month, end_month, amount, payment_day
  , from_payment_method_id, to_payment_method_id, description
) AS (
  SELECT
    id
    , date(substr(start_date, 1, 7) || '-01')
    , min(
      date('now', 'localtime', 'start of month')
      , coalesce(date(substr(end_date, 1, 7) || '-01'), '9999-12-01')
    )
    , amount
    , payment_day
    , from_payment_method_id
    , to_payment_method_id
    , description
  FROM
    recurring_transfers
  WHERE
    is_active = 1
    AND date(substr(start_date, 1, 7) || '-01') <= min(
      date('now', 'localtime', 'start of month')
      , coalesce(date(substr(end_date, 1, 7) || '-01'), '9999-12-01')
    )

  UNION ALL

  SELECT
    recurring_transfer_id
    , date(month, '+1 month')
    , end_month
    , amount
    , payment_day
    , from_payment_method_id
    , to_payment_method_id
    , description
  FROM
    recurring_months
  WHERE
    month < end_month
)

INSERT INTO transfers (
  transaction_date, amount, from_payment_method_id, to_payment_method_id
  , recurring_transfer_id, description
)
SELECT
  date(
    month
    , '+' || (
      min(
        payment_day
        , cast(strftime('%d', date(month, '+1 month', '-1 day')) AS INTEGER)
      ) - 1
    ) || ' days'
  ) AS transaction_date
  , amount
  , from_payment_method_id
  , to_payment_method_id
  , recurring_transfer_id
  , description
FROM
  recurring_months
WHERE
  NOT EXISTS (
    SELECT
      1
    FROM
      transfers AS t
    WHERE
      t.recurring_transfer_id = recurring_months.recurring_transfer_id
      AND date(t.transaction_date, 'start of month') = recurring_months.month
  );
