# apps/web-nuxt

Nuxt 4 + Nuxt UI 4への移行用フロントエンド。詳しい経緯は`docs/design/nuxt-ui-migration-evaluation.md`を参照。

既存の`apps/web`(SvelteKit)は動かしたまま、画面を1つずつこちらへ移植していく。全画面の移植が完了するまで、両方のディレクトリが並行して存在する。

## セットアップ

```bash
npm install
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
npm run typecheck
npm run lint
```
