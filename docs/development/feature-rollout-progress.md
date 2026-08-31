# 機能追加ロードマップ 進捗メモ

現状の仕様を棚卸しし、不足/追加候補として挙がった7機能を実装するロードマップの進捗を記録する。

対象外（実装しない）: レシート画像添付、定期支出の支払日リマインド、認証・マルチユーザー対応。

各機能は「Codexへ独立分析を依頼 → 実装を依頼 → 実装後レビューを依頼 → 指摘対応 → `cargo fmt/test/clippy` と `mise run web-ci` をグリーンにする → 1機能1コミット」の流れで進める。無関係な既存の未コミット差分を巻き込まないよう、コミット前に対象ファイルのみをステージする。

## ステータス

| # | 機能 | 状態 | コミット |
| --- | --- | --- | --- |
| 1 | 定期収入の自動計上 | 完了 | `feat(recurring-incomes): 定期収入の登録と自動計上機能の追加` |
| 2 | 明細の検索・絞り込み | 完了 | `feat(ledger): 支出明細のキーワード・カテゴリ・支払方法による絞り込み機能の追加` |
| 3 | カテゴリの階層化 | 完了 | `feat(categories): 支出・収入カテゴリの親子階層対応` |
| 4 | 資産・口座残高の管理（`payment_methods`拡張） | 未着手 | - |
| 5 | 予算管理（カテゴリ毎の常設予算） | 未着手 | - |
| 6 | Webhookのイベント種別フィルタ | 未着手 | - |
| 7 | グラフ分析の強化 | 未着手 | - |

## 機能1: 定期収入の自動計上（完了）

`recurring_expenses`／定期支出の自動計上ロジック（`docs/design/recurring-expense-posting.md`）を収入側に対称実装した。`recurring_incomes` テーブル、`incomes.recurring_income_id`、収入一覧取得時の遅延生成ロジック、CRUD一式（backend + frontend）、準固定収入の手動確定フローを追加。

レビューで指摘された以下3件は、既存の定期支出/支出側と完全に対称な既知の挙動と判断し、意図的に未対応とした（片方だけ直すとexpense/income間で非対称になるため）。

- 有効期間が対象月の途中にある場合、`transaction_date` が期間外になり得る（月の重なりだけで判定しているため）
- 更新APIで `recurring_income_id` を省略すると `None` に上書きされ、次回一覧取得時に再計上され得る（アプリのUI経由では発生しない）
- 月次画面の定期収入カードが `is_active`/`is_variable` のみで絞り込み、選択月の有効期間を考慮しない

## 機能2: 明細の検索・絞り込み（完了）

分析段階ではバックエンドAPIのQuery/Pagination新設（`Vec<Expense>` → `{items, pagination}` の破壊的変更）を前提としていたが、実装前に既存コードを再確認した結果、方針を転換した。

**実装方針の転換理由**: `+page.svelte` は `onMount` で `api.expenses()`/`api.incomes()` により既に全件を無条件取得し、`inPeriod()`（`domain/summaries.ts`）でクライアント側に選択月分をフィルタしていた。個々の取引明細を一覧表示するUIは `LedgerSheet.svelte` の「支出明細」タブのみ（収入の明細一覧UIはそもそも存在しない）。この規模のアプリ（自己ホスト・単一世帯、想定データ量は多くない）で、既にクライアント側に全件ロード済みのデータに対してバックエンドのページング・破壊的API変更・OpenAPI改修・E2Eモック改修まで行うのは過剰設計と判断し、**バックエンド変更なし・クライアントサイドの絞り込みのみ**で実装した。

**実装内容**:
- `apps/web/src/lib/domain/expense-filters.ts`（新規）: `filterExpenses(expenses, {keyword, categoryId, paymentMethodId})` 純粋関数。keywordは trim・小文字化して `description` に部分一致、category/paymentMethodは完全一致、3条件はAND結合。
- `apps/web/src/lib/features/ledger/LedgerSheet.svelte`: 「支出明細」タブにフィルタバー（キーワード入力・カテゴリ選択・支払方法選択・条件クリアボタン）を追加。既存の月次スコープ済み `expenses` props に対して `$derived` でフィルタ適用のみ（新規API呼び出し・新規state分離は不要）。フィルタ結果0件時は「条件に一致する明細がありません。」、元データ0件時は既存の「この月の支出明細はありません。」を出し分け。
- 他タブ（収支・明細サマリー、月ごとのカテゴリ別支出）は引き続き未フィルターの `expenses` を使用しており、集計値への影響なし。
- `apps/web/src/lib/domain/expense-filters.test.ts`（新規）: 空白キーワード、部分一致、カテゴリ単体、支払方法単体、カテゴリ+支払方法、3条件ANDで0件、の6テスト。

Codexレビューで軽微な指摘2件（`.toSorted()`→既存パターンの`.sort()`へ統一、カテゴリ/支払方法の個別条件テスト追加）があり対応済み。Critical/Majorな指摘はなし。`mise run web-ci` グリーン確認済み。コミット `c7c6fef`。

## 機能3: カテゴリの階層化（完了）

`expense_categories`/`income_categories` に自己参照FK `parent_category_id` を追加（2階層まで）。サービス層で自己参照禁止・親候補が既に子である場合の拒否・子を持つカテゴリを子にする操作の拒否の3検証を実装（不存在の親は400）。複数の子カテゴリを同一親に持たせることは許可。フロントは`CategoryForm.svelte`に親カテゴリセレクト（自分自身と既に子のカテゴリを除外、子を持つカテゴリ編集時は親セレクト無効化）、`settings/+page.svelte`は親の直後に子を並べる表示に変更。`CategoryInput`は支払方法用の入力型と分離。削除失敗時のエラー文言は「明細で使用中」「子カテゴリが存在」の両方をカバー。

Codexレビューで「マージ前に修正が必要」の指摘（income_categories側で既存バリデーションテストが変更時に脱落、明細使用中カテゴリの削除阻止テスト欠如、フロントの非null parent_category_idテスト欠如）があり、全て対応済み。`cargo test`(41 passed)/`clippy`/`mise run web-ci`(30 passed)グリーン。コミット `d0940fa`。

## 未着手機能

もともとの7機能ロードマップのうち残り4件（資産・口座残高管理／予算管理／Webhookイベントフィルタ／グラフ分析強化）に加え、その後の仕様レビューで見つかった4件（収入明細一覧UI／収入明細フィルタ／DBバックアップ／貯蓄目標可視化）を含めた計8件を、依存関係に基づき6つの実装単位に再編した。詳細な設計・実装手順は `docs/development/feature-todo.md` を参照。

| 単位 | 内容 | 状態 |
| --- | --- | --- |
| 1 | 資産・口座残高管理（`payment_methods.initial_balance`、`incomes.payment_method_id`） | 実装・検証済み（未コミット） |
| 2 | 収入明細一覧UI・収入明細フィルタ | 未着手 |
| 3 | 予算管理（`budgets`テーブル、`budget.exceeded` Webhook） | 未着手 |
| 4 | Webhookイベント種別フィルタ（`webhook_urls.events`） | 未着手 |
| 5 | グラフ分析の強化 + 資産推移グラフ | 未着手 |
| 6 | DBバックアップ（ダウンロードのみ） | 未着手 |

対象外（実装しない、単位2〜6の範囲でも変わらず）: DB復元のアプリ内実装（`SqlitePool`が`max_connections(1)`で全ルーターに直接配線されており、稼働中の安全な差し替え手段がないため）、目標金額を登録する貯蓄目標機能（口座残高の推移グラフのみ実装）。

## 単位1: 資産・口座残高管理（実装・検証済み、未コミット）

`payment_methods.initial_balance`（任意、非負整数文字列）と `incomes.payment_method_id`（任意、`payment_methods` FK）をmigration `0006__account_balances` で追加。`payment_methods` の一覧・単体取得に `balance`（`initial_balance` 設定時のみ算出、未設定時は `null`）を追加。`initial_balance` 未設定は「残高追跡対象外」として `0円` と区別する。支払方法の削除は、支出・収入いずれかから参照されている場合にブロックするよう拡張（従来は支出のみ判定）。

レビューで指摘され対応済みの事項:
- 収入・支出の `amount` が非負整数であることを検証していなかった（残高計算が `CAST(amount AS INTEGER)` に依存するため、小数や非数値が計上されると残高が静かに不正確になる問題）→ `service/incomes.rs`・`service/expenses.rs` に検証を追加
- フロントエンドの型定義（`initial_balance`/`balance`/`payment_method_id`）が実際のAPI契約（常にキーは存在しnull許容）とズレていた → レスポンス型を修正（入力型は引き続きoptional）
- 収入の `payment_method_id` に空文字列を送るとFK制約エラーになる問題 → 空文字列は400を返すよう検証を追加

`cargo test`/`clippy`/`mise run web-ci` グリーン確認済み。コミットは未実施（対象ファイルのみをステージして次回コミットする）。
