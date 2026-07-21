# kakeibo-app

## Setup

### Prerequisites

- [mise](https://mise.jdx.dev/)
- SQLite 3（テストデータ操作時に使用）

### Initialize

```bash
mise run setup
cp .env.example .env
```

### Run

```bash
mise run run
```

デフォルトでは `http://localhost:8000` で起動します。ポートを変更する場合は、次のように指定します。

```bash
cargo run -- --port 3000
```

初回起動時に `kakeibo.db` が作成され、マイグレーションが自動で適用されます。

### Check

```bash
mise run ci
```

個別のタスクは `mise tasks` で確認できます。

### Test Data

```bash
# 追加
mise run test-data-add

# 削除
mise run test-data-clear
```

別のSQLiteデータベースを指定する場合は、スクリプトを直接実行します。

```bash
./scripts/test-data.sh add --database path/to/database.db
./scripts/test-data.sh clear --database path/to/database.db
```

## Directory Structure

```text
.
├── migrations/       # SQLxマイグレーション
├── queries/          # リソースごとのSQLクエリ
├── scripts/          # 開発・運用補助スクリプト
├── src/
│   ├── database/     # DB接続、マイグレーション、DBモデル
│   ├── handler/      # HTTPリクエスト・レスポンス処理
│   ├── model/        # APIのリクエスト・レスポンスモデル
│   ├── repository/   # データアクセス
│   ├── router/       # ルーティングとAPIドキュメント
│   ├── service/      # ユースケースとバリデーション
│   └── utils/        # エラー処理、ロギングなどの共通処理
├── tests/            # API統合テスト
├── Cargo.toml        # Rustパッケージ設定
└── mise.toml         # ツールとタスクの定義
```
