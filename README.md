# kakeibo-app

毎日の収支をシンプルに管理する家計簿アプリ。フロントエンドとAPI、SQLiteデータベースを1つのDockerイメージで提供する。

## 公開Dockerイメージを使う

GitHub Container Registry（GHCR）で公開しているイメージを取得します。

```bash
docker pull ghcr.io/kai17-a/kakeibo-app:latest
```

### Dockerで起動する

```bash
docker run --rm -p 8000:8000 -v kakeibo-data:/data ghcr.io/kai17-a/kakeibo-app:latest
```

起動したら `http://localhost:8000` を開いてください。

- フロントエンドとAPIは同じコンテナで配信されます
- APIドキュメント（Redoc）は `http://localhost:8000/docs` で参照できます
- ヘルスチェックは `http://localhost:8000/health` です

### Docker Composeで起動する

次の内容で `compose.yaml` を作成します。

```yaml
services:
  app:
    image: ghcr.io/kai17-a/kakeibo-app:latest
    ports:
      - "8000:8000"
    volumes:
      - kakeibo-data:/data
    environment:
      RUST_LOG: info
    restart: unless-stopped

volumes:
  kakeibo-data:
```

イメージを取得してバックグラウンドで起動します。

```bash
docker compose pull
docker compose up -d
```

停止する場合:

```bash
docker compose down
```

新しいイメージへ更新する場合:

```bash
docker compose pull
docker compose up -d
```

### データの永続化

SQLiteデータベースはコンテナ内の `/data/kakeibo.db` に保存されます。`-v kakeibo-data:/data` でDockerボリュームをマウントしているため、コンテナを削除してもデータは残ります。

データを完全に削除する場合は、コンテナ停止後にボリュームを削除します。

```bash
docker volume rm kakeibo-data
```

Docker Composeの場合は、ボリュームごと削除できます。

```bash
docker compose down -v
```

### ポートを変更する

コンテナ側のポートを変更する場合は、イメージに `--port` オプションを渡し、あわせてポートマッピングを変更します。

```bash
docker run --rm -p 3000:3000 -v kakeibo-data:/data ghcr.io/kai17-a/kakeibo-app:latest --port 3000
```

`http://localhost:3000` でアクセスできます。

### ログレベルを変更する

環境変数 `RUST_LOG` でログレベルを変更できます（デフォルト: `info`）。

```bash
docker run --rm -p 8000:8000 -v kakeibo-data:/data -e RUST_LOG=debug ghcr.io/kai17-a/kakeibo-app:latest
```

### バックグラウンドで起動する

`--rm` の代わりに `-d` を付けてデタッチモードで起動します。

```bash
docker run -d --name kakeibo -p 8000:8000 -v kakeibo-data:/data ghcr.io/kai17-a/kakeibo-app:latest
```

停止する場合:

```bash
docker stop kakeibo
```

## ローカルでDockerイメージをビルドする

リポジトリをクローン済みの場合は、Dockerイメージをローカルでビルドできます。

```bash
docker build -t kakeibo-app .
docker run --rm -p 8000:8000 -v kakeibo-data:/data kakeibo-app
```

## 開発

開発環境のセットアップ、開発サーバーの起動、テストやタスクの実行方法は [DEVELOPMENTS.md](./DEVELOPMENTS.md) を参照してください。
