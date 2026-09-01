# 追加機能 実装TODO

`docs/development/feature-rollout-progress.md` の仕様棚卸しから出た8機能のうち、未実装分の設計と実装手順をまとめる。担当者・使用ツールを問わず着手できるよう、汎用的な開発タスクとして記述する。

対象外（実装しない）: レシート画像添付、定期支出の支払日リマインド、認証・マルチユーザー対応、DB復元のアプリ内実装（理由は「TODO 6」参照）、貯蓄目標金額を登録する専用機能（残高推移グラフのみ実装、詳細は「TODO 5」参照）。

## 進め方

1件ずつ、以下の流れで進める。

1. 対象範囲を実装する
2. 実装した差分をレビューする（設計意図との整合性、エッジケース漏れ、既存パターンからの逸脱がないか）
3. 指摘事項を解消する
4. `cd apps/api && cargo fmt --all && cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings` と、リポジトリルートで `mise run web-ci` をグリーンにする
5. `mise run run` / `mise run web-dev` で実際に画面を操作し、golden pathとエッジケースを手動確認する
6. 無関係な既存の未コミット差分を巻き込まないよう、対象ファイルのみをステージして1機能1コミットで記録する
7. 本ファイルの該当セクションと `feature-rollout-progress.md` のステータス表を更新する

## 依存関係と実装順序

後続のTODOは前のTODOが追加するカラム/データを使うため、番号順に進めることを推奨する。

1. **資産・口座残高管理** — 土台。`payment_methods.initial_balance`、`incomes.payment_method_id` を追加（実装中/実装済み。詳細は `feature-rollout-progress.md` を参照）
2. **収入明細一覧UI・フィルタ** — TODO1の `incomes.payment_method_id` を使う
3. **予算管理** — `budget.exceeded` イベントを新設
4. **Webhookイベント種別フィルタ** — TODO3で増えるイベント種別を含める
5. **グラフ分析の強化 + 資産推移グラフ** — TODO1の残高、TODO3の予算実績を使う
6. **DBバックアップ（ダウンロードのみ）** — 他と独立、最後でよい

---

## TODO 2: 収入明細一覧UI・収入明細フィルタ

**現状**: `apps/web/src/routes/+page.svelte` には支出の編集・削除（`editingExpense`、`deleteTarget: Expense | null`）はあるが、収入の編集・削除UIが配線されていない（`lib/api.ts` に `updateIncome`/`deleteIncome` は存在するが未使用）。`lib/features/ledger/LedgerSheet.svelte` の「支出明細」タブと `lib/domain/expense-filters.ts::filterExpenses` が対応するテンプレートになる。

**方針**: フロントエンドのみ、バックエンド変更なし。このアプリは自己ホスト・単一世帯向けで想定データ量が少なく、`routes/+page.svelte` の `onMount` で収入・支出を既に全件クライアントに読み込み済みのため、既存データに対するクライアントサイドの絞り込みのみで実装する（先行実装した支出側のフィルタ機能と同じ設計判断）。

**実装内容**:
- `lib/domain/income-filters.ts`（新規）: `filterIncomes(incomes, {keyword, categoryId, paymentMethodId})`。`filterExpenses` と同じ形の純粋関数。`paymentMethodId` はTODO1で追加した任意フィールド向けで、未設定の収入は `paymentMethodId` 条件を指定すると除外される仕様でよい
- `lib/features/ledger/LedgerSheet.svelte`: `Tabs.Trigger value="income-details"`（表示名「収入明細」）を追加し、`{#if tab === 'income-details'}` ブロックを新設。「支出明細」タブのマークアップを踏襲（列は 日付・カテゴリ・支払方法（空なら「—」）・金額・備考・操作）。フィルタバーも同パターンで複製
- `routes/+page.svelte`: `editingIncome` 状態を追加し、`deleteTarget` を `Expense | Income` のユニオン型に拡張。収入版の `onedit`/`ondelete` ハンドラを追加して `LedgerSheet` に配線する。`lib/features/forms/TransactionForm.svelte` は `initialIncome` prop経由で編集をサポートしている前提だが、実装時に再確認すること
- `lib/domain/income-filters.test.ts`（新規）: `expense-filters.test.ts` と同構成のテスト

---

## TODO 3: 予算管理

**DB移行**: `apps/api/migrations/0007__budgets.up.sql`（`.down.sql`と対で作成する）
```sql
CREATE TABLE budgets (
  id TEXT PRIMARY KEY DEFAULT (...),  -- 0001__initial.up.sql の他テーブルと同じUUID default句を複製
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  category_id TEXT NOT NULL UNIQUE REFERENCES expense_categories(id),
  amount TEXT NOT NULL
);
```

これは**カテゴリごとに毎月繰り返し適用される恒久的な月額予算**であり、年月ごとの個別予算や履歴は持たない。カテゴリ階層（`parent_category_id`）は考慮せず、各カテゴリ個別に判定する（子カテゴリの支出を親カテゴリの予算にロールアップしない）。

**バックエンド**: `docs/development/api-implementation-order.md` の手順に厳密に従い、新規リソースとして実装する。テンプレートは `webhook_urls`（Query構造体・ページングなしの単純CRUD）を使う（`payment_methods`はページング機構を持つため不適）。

1. `queries/budgets/{find_all,find_by_id,insert,update,delete_by_id}.sql`
2. `src/database/models/budgets.rs`: `BudgetRow`
3. `src/model/budgets.rs`: `Budget`, `BudgetUpsertRequest { category_id, amount }`
4. `src/repository/budgets.rs`
5. `src/service/budgets.rs`: `validate()` で `category_id` の存在チェック（存在しないカテゴリは400）と `amount` の数値チェック。`amount` は `service/incomes.rs`/`service/expenses.rs`と同じ `.parse::<u64>()` 規約に合わせ、`0`も許可する（「このカテゴリでの支出は即座に超過扱い」という意味で有効）。加えてunit1の`initial_balance`検証と同様に`i64`範囲に収まることも確認する。UNIQUE制約違反（同一カテゴリに2つ目の予算）は、`sqlx::Error::Database`の種別を確認して専用の400エラーへマップする（既存コードに前例がないため`repository/budgets.rs`のinsert/updateで個別に実装する）
6. `src/handler/budgets.rs` / `src/router/budgets.rs`
7. `src/router/redoc.rs` へ登録: `paths`・`components(schemas(...))`・`tags`（タグ「予算」）・`tag_for_path()`の4箇所すべてに追加する
8. `tests/budgets_api.rs`
9. カテゴリ削除保護の拡張: `expense_categories`の削除は明細で使用中の場合に既にブロックされているが、`budgets`から参照されている場合もブロックするよう`service/expense_categories.rs`（または該当箇所）を拡張する（unit1で支払方法の削除保護を支出・収入の両方に拡張したのと同じ考え方）

**`budget.exceeded` イベント**: 支出の**作成時・更新時**（削除・定期支出の自動計上は対象外）に、対象カテゴリ・対象月の支出合計が予算以下から超過へ「跨いだ瞬間」だけイベントを発火する。

- **原子性が必須**: INSERT/UPDATEと当月カテゴリ合計の算出を同一トランザクションで行い、コミット後に`notify()`を呼ぶこと。ハンドラ層で「INSERT→（別接続扱いになりうる）SELECT」という現状の`expense.created`通知の呼び出し方をそのまま踏襲すると、SQLiteの単一コネクションプール上でも複数リクエストの文が挟まり得るため、`ExpenseRepository`に「挿入（または更新）とその時点の当月カテゴリ合計取得を1トランザクションで行うメソッド」を新設し、`ExpenseService`がそれを呼んでから`notify`する
- **作成時**: `before_total = after_total - 今回のamount`（同一カテゴリ・同一月内で完結するため1回のトランザクションで足りる）
- **更新時**: 更新前レコードの`category_id`・`transaction_date`・`amount`を取得し、カテゴリまたは月が変わらない場合は`before_total`=更新前合計、`after_total = before_total - 旧amount + 新amount`。カテゴリまたは月が変わる場合は、旧バケットからは判定を行わず（超過から予算内に戻る側は通知しない）、新バケットに対してのみ「跨いだ瞬間」を判定する
- **削除・定期支出自動計上（`insert_recurring_for_month`経由の計上）は対象外**。将来的な課題として残す
- **`budget.recovered`（超過から回復した通知）は対象外**。予算金額自体の変更でもイベントは発火しない（支出の作成・更新のみがトリガー）
- 当月カテゴリ合計を取るための新規クエリが必要（`SELECT COALESCE(SUM(CAST(amount AS NUMERIC)),0) FROM expenses WHERE category_id = ?1 AND strftime('%Y-%m', transaction_date) = ?2`、更新の場合は対象expenseを除外する必要がある点に注意）
- 予算・合計の取得自体が失敗した場合は、支出の作成・更新は成功のまま扱い、警告ログのみ出して通知をスキップする（既存の`notify()`が失敗時にログのみ記録する方針と揃える）
- Webhook payloadには少なくとも `category_id`・カテゴリ名・予算金額・実績金額（`after_total`）・対象年月（`YYYY-MM`）を含める
- 既存の通知の仕組みは `src/service/webhook_urls.rs::notify(event, data)` を呼び出すだけでよい

**関連する既知の未検証ギャップ**（このunitでは対応しない、将来の課題として記録）:
- `service/expenses.rs::validate()`の`transaction_date`は非空チェックのみで、`%Y-%m-%d`形式であることまでは検証していない（`service/incomes.rs::parse_date`や`service/import.rs`と同じ`chrono::NaiveDate::parse_from_str`検証を追加すれば防げるが、budget機能のスコープを超えるため本unitでは対応しない。不正な日付形式のexpenseは月次集計から静かに除外され得る）

**フロントエンド**:
- `routes/settings/+page.svelte`: 新しい `SettingsTab` メンバー `'budget'`、`Tabs.Trigger`/`Tabs.Content`、`budgetPanel` スニペット（カテゴリセレクト＋金額入力のダイアログ式フォーム＋テーブル、既存の`PaymentMethodForm`等と同じCRUD+テーブル+ConfirmDialogの形）。カテゴリセレクトは新規作成時、既に予算が設定済みのカテゴリを除外する（編集時は自分自身の現在のカテゴリは選択肢に残す）
- `lib/features/monthly/MonthlySummary.svelte`: 「支出の内訳」カードの近くに「予算実績」カードを追加。`budgets` propを受け取り、`categoryTotals(expenses, expenseCategories)` の結果と突き合わせて予算に対する進捗バーを表示。進捗バーは100%でクランプし、パーセント表示自体はクランプしない（150%なら「150%」と表示）。予算はあるが当月実績が0円のカテゴリも表示する（0%のバー）。予算がないカテゴリはこのカードに表示しない
- `lib/api.ts`: `budgets()`, `createBudget`, `updateBudget`, `deleteBudget` を追加
- `routes/+page.svelte`: `budgets` 状態を追加し `loadAll()` に組み込む

---

## TODO 4: Webhookイベント種別フィルタ

**DB移行**: `apps/api/migrations/0008__webhook_urls_events.up.sql`
```sql
ALTER TABLE webhook_urls ADD COLUMN events TEXT;
```
（NULL = 全イベント購読、既存行は自動的にNULLで後方互換）

**バックエンド**:
- `src/database/models/webhook_urls.rs` / `src/model/webhook_urls.rs`: `events: Option<String>`（カンマ区切り）を `WebhookUrl`/`WebhookUrlRow`/`WebhookUrlUpsertRequest` に追加
- `queries/webhook_urls/{find_all,find_active,find_by_id,insert,update}.sql`: カラム追加
- `src/service/webhook_urls.rs::notify()`: `find_active()` で取得した行を、`row.events` が `None` または空文字なら常に通知、`Some(csv)` なら `csv.split(',').map(str::trim).any(|e| e == event)` で通知対象を絞り込む
- 既知のイベント名は `"expense.created"`, `"income.created"`, TODO3で追加する `"budget.exceeded"` の3種。`service/webhook_urls.rs::validate()` でこの3種以外を拒否するかは、将来のイベント追加のしやすさとのトレードオフで判断する

**フロントエンド**:
- `routes/settings/+page.svelte` のwebhookパネル: フォームに3つのチェックボックス（支出登録・収入登録・予算超過）を追加。全選択時は `events: null` として送信（将来イベントも自動購読される設計を保つ）、一部のみ選択時はカンマ区切りで送信、1つも選択されていない場合はクライアント側バリデーションで送信不可にする

---

## TODO 5: グラフ分析の強化 + 資産推移グラフ

新規ライブラリは導入しない。`lib/features/annual/AnnualSummary.svelte` の既存の手書きCSSバー方式（`chartMax` を分母にした `style:height` のパーセンテージ指定）を踏襲して以下を追加する。

**`lib/domain/summaries.ts` に純粋関数を追加**（`annualMonthlyTotals` と同じ形）:
- `categoryMonthlyTotals(expenses, categories, year)`: 月×カテゴリのマトリクスを返す。カテゴリ別月次推移の積み上げ棒グラフに使う
- `budgetActuals(expenses, categories, budgets, month)`: カテゴリごとの予算・実績・達成率を返す（TODO3の `budgets` を使用）
- `paymentMethodBalanceTrend(incomes, expenses, paymentMethods, year)`: 各支払方法の月末時点残高推移を返す。`initial_balance` が設定されている支払方法のみ対象（`balance = initial_balance + Σ(その月末までの収入) − Σ(その月末までの支出)`）。バックエンドの残高計算（TODO1）とロジックを重複させず、フロントエンドは生の `incomes`/`expenses`/`paymentMethods.initial_balance` から独立に計算する（月次スナップショットAPIは新設しない）。これが「貯蓄目標可視化」の実装範囲であり、目標金額を登録する機能は作らない

**`AnnualSummary.svelte` に3つのセクションを追加**（既存の月別収支バーグラフの下に配置）:
- カテゴリ別月次推移（積み上げ棒、既存のカラートークンを使い回す）
- 予算実績比較（予算のあるカテゴリのみ、達成率バー + 数値）
- 資産（支払方法別）残高推移グラフ（`paymentMethodBalanceTrend` の結果を棒グラフで表現）

**テスト**: `summaries.test.ts` に3関数分のケースを追加

---

## TODO 6: DBバックアップ（ダウンロードのみ）

**スコープ制約**: 復元（リストア）はアプリ内実装しない。理由: `apps/api/src/database/migration.rs` の `SqlitePool` は `max_connections(1)` で `main.rs` の全ルーターに `.clone()` で直接配線されており、稼働中のプールを安全に入れ替える仕組みが存在しない。復元は「コンテナ停止→ボリューム内の `.db` ファイルを差し替え→起動」という手順（README参照）で運用する。

**バックエンド**:
- `src/router/backup.rs` / `src/handler/backup.rs`: `GET /api/backup`
- `src/service/backup.rs`（新規）: `sqlx::query("VACUUM INTO ?1").bind(&tmp_path).execute(&pool)` で一時ファイルへ整合性のあるスナップショットを書き出し、ファイルバイト列を読み込んでレスポンスボディにし、一時ファイルを削除する。`tmp_path` は `std::env::temp_dir()` 配下にUUID等でユニーク化する
- レスポンスヘッダ: `Content-Type: application/octet-stream`、`Content-Disposition: attachment; filename="kakeibo-backup-{YYYYMMDD}.db"`（`src/handler/export.rs` のヘッダ設定パターンを参考にする。中身はバイナリ）
- `src/router/redoc.rs` へ登録

**フロントエンド**:
- `routes/settings/data/+page.svelte`: 既存の「データのエクスポート」Cardの並びに「フルバックアップ」Cardを追加。`<Button variant="outline" href="/api/backup">` のプレーンリンク（CSVエクスポートと同じ、JS fetchなし）

**README.md**: 「データの永続化」節の後に「バックアップとリストア」節を追加。ダウンロード方法（上記UI）と、復元手順（`docker compose down` → ダウンロード済み `.db` を `docker volume` 内の `kakeibo.db` にコピー → `docker compose up -d`）を明記する。

---

## 検証方法（各TODO共通）

1. `cd apps/api && cargo fmt --all && cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings`
2. `mise run web-ci`（フロントエンドの型チェック・lint・テスト）
3. `mise run run` でAPIサーバーを起動し、`mise run web-dev` でフロントエンドを起動して `http://localhost:5173` で該当機能を手動確認する（新規UIは実際にブラウザで操作し、golden pathとエッジケース — 予算未設定カテゴリ、初期残高未設定の支払方法、events未選択のWebhook等 — を確認する）
4. `/api/docs`（Redoc）で新規/変更エンドポイントのスキーマが正しく表示されることを確認する
5. 完了後、本ファイルの該当セクションを削除し、`feature-rollout-progress.md` のステータス表と履歴セクションを更新してからコミットする
