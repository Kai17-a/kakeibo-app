# Web frontend

Nuxt 4 + Nuxt UI 4で実装したフロントエンド。移行の経緯は`docs/design/nuxt-ui-migration-evaluation.md`を参照。

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
