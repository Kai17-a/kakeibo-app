-- 指定月（?1: YYYY-MM）の定期支出を支出明細として計上する。
-- payment_day が月末日を超える場合は月末日に丸める。
-- 同じ定期支出・同じ月の明細が既にある場合は二重計上しない。
INSERT INTO expenses (
  transaction_date
  , amount
  , category_id
  , payment_method_id
  , recurring_expense_id
  , description
)
SELECT
  date(
    ?1 || '-01'
    , '+' || (
      min(
        r.payment_day
        , cast(
          strftime('%d', date(?1 || '-01', '+1 month', '-1 day')) AS INTEGER
        )
      ) - 1
    ) || ' days'
  ) AS transaction_date
  , r.amount
  , r.category_id
  , r.payment_method_id
  , r.id AS recurring_expense_id
  , r.description
FROM
  recurring_expenses AS r
WHERE
  r.is_active = 1
  AND r.is_variable = 0
  AND r.start_date <= date(?1 || '-01', '+1 month', '-1 day')
  AND (r.end_date IS NULL OR r.end_date >= ?1 || '-01')
  AND NOT EXISTS (
    SELECT
      1
    FROM
      expenses AS e
    WHERE
      e.recurring_expense_id = r.id
      AND strftime('%Y-%m', e.transaction_date) = ?1
  );
