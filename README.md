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

### バックアップとリストア

設定画面の「データ管理」にある「フルバックアップ」から、SQLiteデータベースをダウンロードできます。

#### 画面から復元する

同じ画面の「フルバックアップの復元」でバックアップファイル（256 MiB以下）を選ぶと、警告を確認したうえで、現在の全データをその内容で置き換えられます。追加・マージではありません。古いバージョンで取得したバックアップは現在の形式に更新して取り込みます。現在のアプリより新しいバージョンのバックアップは復元できません。

復元の直前に、現在のデータの退避コピーが自動で作成されます。

- 保存先: データベースと同じディレクトリの `.restore-backups/`（Dockerでは `/data/.restore-backups/`、開発時は `apps/api/.restore-backups/`）
- ファイル名: `kakeibo-before-restore-<時刻>.db`
- 新しい5世代を残し、古いものから自動で削除されます

復元前の状態に戻すには、退避コピーを画面から復元するか、次の手動リストアの手順で `kakeibo.db` を退避コピーに差し替えます。

#### 手動でリストアする

上記のDocker Compose named volume構成へリストアする場合は、次の手順でコンテナを停止してからデータベースを差し替えます。

1. `docker compose down` を実行する
2. Dockerボリューム `kakeibo-data` 内の `kakeibo.db` を、ダウンロードしたバックアップファイルで置き換える
3. `docker compose up -d` を実行する

差し替え時は、`kakeibo.db-wal` や `kakeibo.db-shm` などのsidecarファイルが残っていないことも確認してください。

## 開発

開発環境のセットアップ、開発サーバーの起動、テストやタスクの実行方法は [DEVELOPMENTS.md](./DEVELOPMENTS.md) を参照してください。
