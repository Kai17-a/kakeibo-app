# AGENTS.md

このファイルは、このリポジトリでエージェントが作業および Git コミットを行うときの運用ルールをまとめる。

## 作業完了時のコミット

依頼された作業が完了したら、変更内容を確認したうえで必ずコミットする。

作業完了とは、次の状態を指す。

- 依頼された変更が実装されている
- 必要なフォーマット、静的解析、テストが完了している
- 発見した不具合や未完了箇所が残っていない
- コミット対象の差分が整理されている

作業途中の状態や、テストに失敗している状態ではコミットしない。

複数の目的を含む場合は、作業内容を適切な単位に分割してコミットする。

ユーザーからコミットしないよう明示的な指示がある場合は、その指示を優先する。

## コミット前の確認

コミット前に必ず差分とステージ状態を確認する。

```bash
git status
git diff
git diff --cached
```

確認すること:

- 1 コミット 1 目的になっているか
- 不要なファイル、生成物、ログ、秘密情報が含まれていないか
- フォーマット変更と内容変更が混ざっていないか
- 既存のユーザー変更を巻き込んでいないか

## ステージング

意図した変更だけをステージする。

```bash
git add path/to/file
git add -p
```

`git add .` は、対象範囲が明確な場合だけ使う。

## コミット単位

コミット単位は `git/COMMIT_UNIT_GUIDE.md` に従う。

- 1 コミット 1 目的を守る
- レビュー、revert、cherry-pick しやすい単位にする
- `wip` や後追い修正の連続をそのまま残さない
- 必要に応じて `git/cook-book/rebase/` の手順で履歴を整理する

## コミットメッセージ

コミットメッセージは `git/COMMIT_MESSAGE_GUIDE.md` に従い、Conventional Commits 形式にする。

基本形式:

```text
<type>(<scope>): <description>
```

日本語の `description` は体言止めで簡潔に書く。

例:

```text
docs(readme): 開発ガイドラインREADMEの整備
docs(git): rebase手順Cook Bookの追加
fix(config): 設定読み込み失敗時の例外処理
```

## コミット実行

ステージ済み差分を確認してからコミットする。

```bash
git diff --cached
git commit -m "docs(readme): 開発ガイドラインREADMEの整備"
```

コミット後に状態と履歴を確認する。

```bash
git status
git log --oneline --decorate -1
```

作業対象の変更がコミットされ、意図しない未コミット差分が残っていないことを確認する。

## 注意

- 共有済み履歴を書き換える操作は、明示的な依頼がない限り行わない
- `git reset --hard` や `git clean -fd` のような破壊的操作は、明示的な依頼なしに実行しない
- コミット対象外の変更がある場合は、勝手に戻さず対象から外す
- コミットできない理由がある場合は、理由と未コミットの変更内容をユーザーに報告する

---

You are able to use the Svelte MCP server, where you have access to comprehensive Svelte 5 and SvelteKit documentation. Here's how to use the available tools effectively:

## Available Svelte MCP Tools:

### 1. list-sections

Use this FIRST to discover all available documentation sections. Returns a structured list with titles, use_cases, and paths.
When asked about Svelte or SvelteKit topics, ALWAYS use this tool at the start of the chat to find relevant sections.

### 2. get-documentation

Retrieves full documentation content for specific sections. Accepts single or multiple sections.
After calling the list-sections tool, you MUST analyze the returned documentation sections (especially the use_cases field) and then use the get-documentation tool to fetch ALL documentation sections that are relevant for the user's task.

### 3. svelte-autofixer

Analyzes Svelte code and returns issues and suggestions.
You MUST use this tool whenever writing Svelte code before sending it to the user. Keep calling it until no issues or suggestions are returned.

### 4. playground-link

Generates a Svelte Playground link with the provided code.
After completing the code, ask the user if they want a playground link. Only call this tool after user confirmation and NEVER if code was written to files in their project.

---

## shadcn-svelte

- UIコンポーネントにはshadcn-svelteを使用する。
- コンポーネントを手動で新規実装する前に、既存コンポーネントを確認する。
- コンポーネントの追加にはshadcn-svelte CLIを使用する。
- React版shadcn/uiのコードを使用しない。
- Svelte 5のrunesと、現在のBits UI APIを使用する。
- components.jsonのエイリアスと設定に従う。
