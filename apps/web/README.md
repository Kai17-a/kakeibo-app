# Web frontend

Nuxt 4 + Kav-UI のフロントエンド。Kav-UI の CSS は `app/assets/vendor/kav-ui/css/` に同梱し、Vue の自前部品が入力・モーダル・通知などの状態を管理する。出典と更新方法は`app/assets/vendor/kav-ui/README.md` を参照。外部ディレクトリへの相対パス依存はない。既存レイアウトの Tailwind ユーティリティは `@tailwindcss/vite` で処理する。

## セットアップ

```bash
npm ci
```

## 開発サーバー

`apps/api`(ポート8000)を起動した状態で:

```bash
npm run dev
```

`http://localhost:3000`で起動する。`/api`へのリクエストは開発時、`nuxt.config.ts`の`nitro.devProxy`設定により`http://localhost:8000`(Rust APIサーバー)へプロキシされる。

## ビルド・チェック

```bash
npm run build
npm run generate
npm run typecheck
npm run lint
npm run lint:fix
npm run format
npm run format:check
npm test
npm run test:e2e
```

本番用のDockerイメージでは`npm run generate`が生成する`.output/public`をRust APIから静的配信する。
