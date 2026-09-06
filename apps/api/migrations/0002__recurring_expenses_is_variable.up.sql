-- 定期支出設定に金額変動フラグ（準固定費）を追加
-- is_variable = 1 の場合は金額が月ごとに変動する「準固定費」として扱い、
-- 支出明細への自動計上は行わない
ALTER TABLE recurring_expenses
ADD COLUMN is_variable INTEGER NOT NULL DEFAULT 0;
