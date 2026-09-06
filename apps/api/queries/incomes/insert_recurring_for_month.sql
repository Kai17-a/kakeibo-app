-- 指定月（?1: YYYY-MM）の定期収入を収入明細として計上する。
-- payment_day が月末日を超える場合は月末日に丸める。
-- 同じ定期収入・同じ月の明細が既にある場合は二重計上しない。
INSERT INTO incomes (
  transaction_date, amount, category_id, recurring_income_id, description
)
SELECT
  date(
    ?1 || '-01'
    , '+' || (
      min(
        r.payment_day
        , cast(strftime('%d', date(?1 || '-01', '+1 month', '-1 day')) AS INTEGER)
      ) - 1
    ) || ' days'
  ) AS transaction_date
  , r.amount
  , r.category_id
  , r.id AS recurring_income_id
  , r.description
FROM
  recurring_incomes AS r
WHERE
  r.is_active = 1
  AND r.is_variable = 0
  AND r.start_date <= date(?1 || '-01', '+1 month', '-1 day')
  AND (r.end_date IS NULL OR r.end_date >= ?1 || '-01')
  AND NOT EXISTS (
    SELECT
      1
    FROM
      incomes AS i
    WHERE
      i.recurring_income_id = r.id
      AND strftime('%Y-%m', i.transaction_date) = ?1
  );
