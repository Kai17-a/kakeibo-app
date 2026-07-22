# kakeibo-app

## Setup

### Prerequisites

- [mise](https://mise.jdx.dev/)
- SQLite 3（テストデータ操作時に使用）

### Initialize

```bash
mise run setup
cp apps/api/.env.example apps/api/.env
```

### Run

```bash
mise run run
```

デフォルトでは `http://localhost:8000` で起動します。ポートを変更する場合は、次のように指定します。

```bash
mise run run -- --port 3000
```

初回起動時に `apps/api/kakeibo.db` が作成され、マイグレーションが自動で適用されます。

別のターミナルでフロントエンドを起動します。

```bash
mise run web-install
mise run web-dev
```

`http://localhost:5173` を開いてください。開発サーバーは `/api` を
`http://localhost:8000` にプロキシします。

### Run with Docker

```bash
docker build -t kakeibo-app .
docker run --rm -p 8000:8000 -v kakeibo-data:/data kakeibo-app
```

`http://localhost:8000` を開いてください。フロントエンドとAPIを同じコンテナで配信します。
SQLiteデータベースは `kakeibo-data` ボリュームに保存されます。

### Check

```bash
mise run ci
```

フロントエンドだけを確認する場合:

```bash
mise run web-ci
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
├── apps/
│   ├── api/
│   │   ├── migrations/  # SQLxマイグレーション
│   │   ├── queries/     # リソースごとのSQLクエリ
│   │   ├── src/
│   │   │   ├── database/    # DB接続、マイグレーション、DBモデル
│   │   │   ├── handler/     # HTTPリクエスト・レスポンス処理
│   │   │   ├── model/       # APIのリクエスト・レスポンスモデル
│   │   │   ├── repository/  # データアクセス
│   │   │   ├── router/      # ルーティングとAPIドキュメント
│   │   │   ├── service/     # ユースケースとバリデーション
│   │   │   └── utils/       # エラー処理、ロギングなどの共通処理
│   │   ├── tests/       # API統合テスト
│   │   └── Cargo.toml   # Rustパッケージ設定
│   └── web/             # フロントエンド配置先
├── scripts/          # 開発・運用補助スクリプト
└── mise.toml         # ツールとタスクの定義
```
