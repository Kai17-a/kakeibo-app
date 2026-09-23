# Nuxt.js + Nuxt UIへの移行検討（評価結果）

[Issue #16](https://github.com/Kai17-a/kakeibo-app/issues/16) で提起された、フロントエンド(`apps/web`)のNuxt.js + Nuxt UIへの移行について、着手可否を判断するための評価をまとめる。

## 結論

**現時点では移行しない。** 移行を正当化する具体的な技術的・プロダクト的な動機が見つからず、一方で移行コスト（既存130コンポーネント・約7,300行のSvelteコードの全面書き換え）が大きい。移行の引き金となる具体的な課題（後述）が発生した場合に再検討する。

## 現状のスタック（調査結果）

- フレームワーク: SvelteKit 5（Svelte 5 runes: `$state`/`$derived`/`$effect`/`$props`）
- UIライブラリ: shadcn-svelte（Tailwind CSSベース、Tailwind Variants APIでテーマ管理）— 導入済みコンポーネント24種（alert, alert-dialog, badge, button, card, checkbox, command, dialog, empty, field, input, input-group, label, native-select, popover, progress, separator, skeleton, sonner, spinner, table, tabs, textarea, toggle, toggle-group）
- レンダリング方式: `adapter-static`によるSSG/CSR。SSRは使用していない（`apps/api`のRustサーバーが静的ファイルを配信する構成のため、SSRを必要とする要件がそもそも存在しない）
- 規模: `.svelte`ファイル130個、合計約7,300行
- テスト: Playwright E2Eテスト8本、Vitestユニットテスト50件

## Nuxt UIの概要（Issue #16作成時に調査済み）

- Vue/Nuxt向けのUIコンポーネントライブラリ。Tailwind CSSベースで、Tailwind Variants APIによるテーマカスタマイズが可能（shadcn-svelteと概念的に近い設計）
- SSRへの一次対応、Nuxt Content・i18n・Fonts・Icon・Color Modeなど周辺モジュールとの統合が充実
- プレーンなVue(Vite)構成でも利用可能（Nuxt本体は必須ではない）

## 比較

| 観点 | 現状（SvelteKit + shadcn-svelte） | Nuxt + Nuxt UI |
| --- | --- | --- |
| SSR | 不使用（`adapter-static`、要件なし） | 一次対応（本アプリでは活かせない） |
| コンポーネント設計思想 | Tailwind Variants、コピー＆オウンパターン | Tailwind Variants、ライブラリとして配布 |
| 言語・リアクティビティ | Svelte 5 runes | Vue Composition API |
| 周辺エコシステム統合 | 個別に実装済み（i18n等は本アプリでは未使用） | i18n/Content/Fonts等が標準統合 |
| 本アプリでの利用実績 | 130コンポーネントが稼働中、直近もモバイルUI改善（タブの横スクロール、ダイアログの高さ制御等）を実施済み | ゼロから再構築が必要 |

## 移行コストの試算

- `.svelte`ファイル130個（約7,300行）をVue Composition APIへ全面書き換え
- shadcn-svelteのコンポーネントカスタマイズ（24種）をNuxt UIの対応コンポーネントへ再実装
- 直近のセッションで修正したモバイルUI関連の不具合（設定画面タブの横スクロール、ダイアログの高さ超過、テーブルの折り返し等）を含め、レイアウト・スタイリングを再度作り直し、再検証する必要がある
- Playwright E2Eテスト8本・Vitestユニットテスト50件の書き直し、または移行方針次第では並行稼働期間の維持コスト

## 移行の動機となりうる具体的な課題（現時点では未発生）

Issue #16作成時点で「移行のトリガーとなる具体的な課題が明確になっていない」ことが未確認事項として挙げられていた。調査の結果、以下のような動機は本アプリには該当しないことを確認した。

- **SSR/SEOの必要性**: 本アプリは認証不要の個人向け家計簿アプリで、`adapter-static`によるCSR構成。検索エンジン最適化やSSRを必要とする要件は存在しない
- **多言語対応(i18n)**: 現状、日本語のみのUIであり、Nuxt i18nの統合を活かす場面がない
- **開発者体験・エコシステムの問題**: SvelteKit 5 + shadcn-svelteで直近まで機能追加（固定費CSVインポート、モバイルUI改善等）が円滑に進んでおり、開発体験上の具体的な障害は確認されていない

以下のような状況が生じた場合は、再度移行を検討する価値がある。

- SSRやSEOが必要な機能（公開ページ、検索流入の獲得等）を追加することになった場合
- 多言語対応が必要になった場合
- Svelte/SvelteKitエコシステムの停滞やメンテナンス上の重大な問題が生じた場合

## 却下した代替案

| 代替案 | 却下理由 |
| --- | --- |
| 影響範囲の小さい画面（設定画面の1タブ等）で試験移植し、DXを比較する | 移行を正当化する動機自体が現時点でないため、試験移植のコストに見合う判断材料が得られない。動機が明確になった時点で改めて実施を検討する |
| 段階的に全画面を移行する | 全面移行前提の代替案のため、動機が明確でない現時点では時期尚早 |
