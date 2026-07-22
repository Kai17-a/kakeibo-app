#!/usr/bin/env bash

set -euo pipefail

readonly REPOSITORY_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly DEFAULT_DATABASE="$REPOSITORY_ROOT/apps/api/kakeibo.db"

usage() {
  cat <<'EOF'
Usage:
  scripts/test-data.sh <add|clear> [--database <path>]

Options:
  -d, --database <path>  SQLite database path (default: apps/api/kakeibo.db)
  -h, --help             Show this help
EOF
}

action=""
database="$DEFAULT_DATABASE"

while (($# > 0)); do
  case "$1" in
    add | clear)
      if [[ -n "$action" ]]; then
        echo "Error: specify only one action." >&2
        usage >&2
        exit 2
      fi
      action="$1"
      shift
      ;;
    -d | --database)
      if (($# < 2)); then
        echo "Error: $1 requires a path." >&2
        exit 2
      fi
      database="$2"
      shift 2
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "Error: unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "$action" ]]; then
  echo "Error: add or clear is required." >&2
  usage >&2
  exit 2
fi

if [[ ! -f "$database" ]]; then
  echo "Error: database does not exist: $database" >&2
  exit 1
fi

add_test_data() {
  sqlite3 -bail "$database" <<'SQL'
PRAGMA foreign_keys = ON;
BEGIN IMMEDIATE;

INSERT INTO income_categories (id, name, description)
VALUES
  ('00000000-0000-4000-8000-000000000101', '給与', 'テスト用の給与カテゴリ'),
  ('00000000-0000-4000-8000-000000000102', '臨時収入', 'テスト用の臨時収入カテゴリ')
ON CONFLICT (id) DO UPDATE SET
  name = excluded.name,
  description = excluded.description,
  updated_at = current_timestamp;

INSERT INTO expense_categories (id, name, description)
VALUES
  ('00000000-0000-4000-8000-000000000201', '食費', 'テスト用の食費カテゴリ'),
  ('00000000-0000-4000-8000-000000000202', '住居費', 'テスト用の住居費カテゴリ'),
  ('00000000-0000-4000-8000-000000000203', '光熱費', 'テスト用の光熱費カテゴリ'),
  ('00000000-0000-4000-8000-000000000204', '交通費', 'テスト用の交通費カテゴリ'),
  ('00000000-0000-4000-8000-000000000205', '娯楽費', 'テスト用の娯楽費カテゴリ'),
  ('00000000-0000-4000-8000-000000000206', '医療費', 'テスト用の医療費カテゴリ'),
  ('00000000-0000-4000-8000-000000000207', '教育費', 'テスト用の教育費カテゴリ'),
  ('00000000-0000-4000-8000-000000000208', '通信費', 'テスト用の通信費カテゴリ'),
  ('00000000-0000-4000-8000-000000000209', '日用品', 'テスト用の日用品カテゴリ'),
  ('00000000-0000-4000-8000-000000000210', 'その他', 'テスト用のその他カテゴリ')
ON CONFLICT (id) DO UPDATE SET
  name = excluded.name,
  description = excluded.description,
  updated_at = current_timestamp;

INSERT INTO payment_methods (id, name, description)
VALUES
  ('00000000-0000-4000-8000-000000000301', '現金', 'テスト用の現金支払い'),
  ('00000000-0000-4000-8000-000000000302', 'クレジットカード', 'テスト用のカード支払い')
ON CONFLICT (id) DO UPDATE SET
  name = excluded.name,
  description = excluded.description,
  updated_at = current_timestamp;

INSERT INTO recurring_expenses (
  id, name, amount, payment_day, start_date, end_date,
  category_id, payment_method_id, is_active, description
)
VALUES (
  '00000000-0000-4000-8000-000000000401', '家賃', '85000', 27, '2026-01-01', NULL,
  '00000000-0000-4000-8000-000000000202',
  '00000000-0000-4000-8000-000000000302', 1, 'テスト用の定期支出'
)
ON CONFLICT (id) DO UPDATE SET
  name = excluded.name,
  amount = excluded.amount,
  payment_day = excluded.payment_day,
  start_date = excluded.start_date,
  end_date = excluded.end_date,
  category_id = excluded.category_id,
  payment_method_id = excluded.payment_method_id,
  is_active = excluded.is_active,
  description = excluded.description,
  updated_at = current_timestamp;

INSERT INTO incomes (id, category_id, transaction_date, amount, description)
VALUES
  (
    '00000000-0000-4000-8000-000000000501',
    '00000000-0000-4000-8000-000000000101', '2026-07-25', '300000', '7月分給与'
  ),
  (
    '00000000-0000-4000-8000-000000000502',
    '00000000-0000-4000-8000-000000000102', '2026-07-10', '20000', '臨時収入'
  )
ON CONFLICT (id) DO UPDATE SET
  category_id = excluded.category_id,
  transaction_date = excluded.transaction_date,
  amount = excluded.amount,
  description = excluded.description,
  updated_at = current_timestamp;

INSERT INTO expenses (
  id, transaction_date, amount, category_id,
  payment_method_id, recurring_expense_id, description
)
VALUES
  (
    '00000000-0000-4000-8000-000000000601', '2026-07-19', '4200',
    '00000000-0000-4000-8000-000000000201',
    '00000000-0000-4000-8000-000000000301', NULL, '食料品'
  ),
  (
    '00000000-0000-4000-8000-000000000602', '2026-07-27', '85000',
    '00000000-0000-4000-8000-000000000202',
    '00000000-0000-4000-8000-000000000302',
    '00000000-0000-4000-8000-000000000401', '7月分家賃'
  ),
  (
    '00000000-0000-4000-8000-000000000603', '2026-07-15', '12000',
    '00000000-0000-4000-8000-000000000203',
    '00000000-0000-4000-8000-000000000302', NULL, '電気・ガス料金'
  ),
  (
    '00000000-0000-4000-8000-000000000604', '2026-07-18', '880',
    '00000000-0000-4000-8000-000000000204',
    '00000000-0000-4000-8000-000000000301', NULL, '電車運賃'
  ),
  (
    '00000000-0000-4000-8000-000000000605', '2026-07-20', '2000',
    '00000000-0000-4000-8000-000000000205',
    '00000000-0000-4000-8000-000000000302', NULL, '映画鑑賞'
  ),
  (
    '00000000-0000-4000-8000-000000000606', '2026-07-21', '3500',
    '00000000-0000-4000-8000-000000000206',
    '00000000-0000-4000-8000-000000000301', NULL, '診察代'
  ),
  (
    '00000000-0000-4000-8000-000000000607', '2026-07-22', '4200',
    '00000000-0000-4000-8000-000000000207',
    '00000000-0000-4000-8000-000000000302', NULL, '参考書'
  ),
  (
    '00000000-0000-4000-8000-000000000608', '2026-07-23', '6500',
    '00000000-0000-4000-8000-000000000208',
    '00000000-0000-4000-8000-000000000302', NULL, '携帯電話料金'
  ),
  (
    '00000000-0000-4000-8000-000000000609', '2026-07-24', '1600',
    '00000000-0000-4000-8000-000000000209',
    '00000000-0000-4000-8000-000000000301', NULL, '洗剤・消耗品'
  ),
  (
    '00000000-0000-4000-8000-000000000610', '2026-07-26', '3000',
    '00000000-0000-4000-8000-000000000210',
    '00000000-0000-4000-8000-000000000301', NULL, '贈り物'
  )
ON CONFLICT (id) DO UPDATE SET
  transaction_date = excluded.transaction_date,
  amount = excluded.amount,
  category_id = excluded.category_id,
  payment_method_id = excluded.payment_method_id,
  recurring_expense_id = excluded.recurring_expense_id,
  description = excluded.description,
  updated_at = current_timestamp;

COMMIT;
SQL
}

clear_test_data() {
  sqlite3 -bail "$database" <<'SQL'
PRAGMA foreign_keys = ON;
BEGIN IMMEDIATE;

DELETE FROM expenses
WHERE id IN (
  '00000000-0000-4000-8000-000000000601',
  '00000000-0000-4000-8000-000000000602',
  '00000000-0000-4000-8000-000000000603',
  '00000000-0000-4000-8000-000000000604',
  '00000000-0000-4000-8000-000000000605',
  '00000000-0000-4000-8000-000000000606',
  '00000000-0000-4000-8000-000000000607',
  '00000000-0000-4000-8000-000000000608',
  '00000000-0000-4000-8000-000000000609',
  '00000000-0000-4000-8000-000000000610'
);

DELETE FROM incomes
WHERE id IN (
  '00000000-0000-4000-8000-000000000501',
  '00000000-0000-4000-8000-000000000502'
);

DELETE FROM recurring_expenses
WHERE id = '00000000-0000-4000-8000-000000000401';

DELETE FROM payment_methods
WHERE id IN (
  '00000000-0000-4000-8000-000000000301',
  '00000000-0000-4000-8000-000000000302'
);

DELETE FROM expense_categories
WHERE id IN (
  '00000000-0000-4000-8000-000000000201',
  '00000000-0000-4000-8000-000000000202',
  '00000000-0000-4000-8000-000000000203',
  '00000000-0000-4000-8000-000000000204',
  '00000000-0000-4000-8000-000000000205',
  '00000000-0000-4000-8000-000000000206',
  '00000000-0000-4000-8000-000000000207',
  '00000000-0000-4000-8000-000000000208',
  '00000000-0000-4000-8000-000000000209',
  '00000000-0000-4000-8000-000000000210'
);

DELETE FROM income_categories
WHERE id IN (
  '00000000-0000-4000-8000-000000000101',
  '00000000-0000-4000-8000-000000000102'
);

COMMIT;
SQL
}

case "$action" in
  add)
    add_test_data
    echo "Test data added to $database"
    ;;
  clear)
    clear_test_data
    echo "Test data cleared from $database"
    ;;
esac
