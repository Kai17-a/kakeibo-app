# kakeibo-app

毎日の収支をシンプルに管理する家計簿アプリ。フロントエンドとAPI、SQLiteデータベースを1つのDockerイメージで提供する。

## 公開Dockerイメージを使う

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

```bash
docker compose up -d
```

### データの永続化

SQLiteデータベースはコンテナ内の `/data/kakeibo.db` に保存されます。上記のコマンドと設定では、Dockerボリューム `kakeibo-data` にデータを永続化します。

## 開発

開発環境のセットアップ、開発サーバーの起動、テストやタスクの実行方法は [DEVELOPMENTS.md](./DEVELOPMENTS.md) を参照してください。
