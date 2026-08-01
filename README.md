# kakeibo-app

毎日の収支をシンプルに管理する家計簿アプリ。フロントエンドとAPI、SQLiteデータベースを1つのDockerイメージで提供する。

## 起動方法（Docker）

### イメージのビルド

```bash
docker build -t kakeibo-app .
```

### 起動

```bash
docker run --rm -p 8000:8000 -v kakeibo-data:/data kakeibo-app
```

起動したら `http://localhost:8000` を開いてください。

- フロントエンドとAPIは同じコンテナで配信されます
- APIドキュメント（Redoc）は `http://localhost:8000/docs` で参照できます
- ヘルスチェックは `http://localhost:8000/health` です

### データの永続化

SQLiteデータベースはコンテナ内の `/data/kakeibo.db` に保存されます。`-v kakeibo-data:/data` でDockerボリュームをマウントしているため、コンテナを削除してもデータは残ります。

データを完全に削除する場合は、コンテナ停止後にボリュームを削除します。

```bash
docker volume rm kakeibo-data
```

### ポートを変更する

コンテナ側のポートを変更する場合は、イメージに `--port` オプションを渡し、あわせてポートマッピングを変更します。

```bash
docker run --rm -p 3000:3000 -v kakeibo-data:/data kakeibo-app --port 3000
```

`http://localhost:3000` でアクセスできます。

### ログレベルを変更する

環境変数 `RUST_LOG` でログレベルを変更できます（デフォルト: `info`）。

```bash
docker run --rm -p 8000:8000 -v kakeibo-data:/data -e RUST_LOG=debug kakeibo-app
```

### バックグラウンドで起動する

`--rm` の代わりに `-d` を付けてデタッチモードで起動します。

```bash
docker run -d --name kakeibo -p 8000:8000 -v kakeibo-data:/data kakeibo-app
```

停止する場合:

```bash
docker stop kakeibo
```

## 開発

開発環境のセットアップ、開発サーバーの起動、テストやタスクの実行方法は [DEVELOPMENTS.md](./DEVELOPMENTS.md) を参照してください。

