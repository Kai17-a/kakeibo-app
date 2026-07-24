# API機能の実装順

リソース単位で、APIからデータベースまで縦方向に完成させる。

Repositoryを全リソース分まとめて実装するのではなく、1つのリソースをテストまで完成させてから次へ進む。

## 1. API仕様を決める

以下の項目を実装前に決める。

- URL
- HTTPメソッド
- リクエスト
- レスポンス
- クエリパラメータ
- HTTPステータス
- エラー条件

## 2. SQLを作成する

```text
queries/<resource>/
```

必要な操作に応じてSQLファイルを追加する。

```text
find_all.sql
find_by_id.sql
insert.sql
update.sql
delete_by_id.sql
```

## 3. DBモデルを作成する

```text
src/database/models/<resource>.rs
```

SQLの取得結果を受け取るためのモデルを定義する。

## 4. APIモデルを作成する

```text
src/model/<resource>.rs
```

以下のモデルを必要に応じて定義する。

- レスポンスモデル
- 登録・更新用の入力モデル
- クエリパラメータ
- ページング情報

## 5. Repositoryを作成する

```text
src/repository/<resource>.rs
```

Repositoryはデータベース操作を担当する。

- 一覧取得
- ID指定取得
- 登録
- 更新
- 削除

## 6. Serviceを作成する

```text
src/service/<resource>.rs
```

Serviceはアプリケーションのルールを担当する。

- 入力値の検証
- Not Found判定
- ページング処理
- Repositoryの呼び出し
- DBモデルからAPIモデルへの変換

## 7. Handlerを作成する

```text
src/handler/<resource>.rs
```

HandlerはHTTPの入出力を担当する。

- Pathパラメータの取得
- Queryパラメータの取得
- JSONリクエストの取得
- HTTPステータスの決定
- JSONレスポンスの返却

## 8. Routerを作成する

```text
src/router/<resource>.rs
```

URLとHandlerを関連付ける。

親の `mod.rs` にはモジュール宣言だけを追加する。

```rust
pub mod <resource>;
```

## 9. ReDocへ登録する

```text
src/router/redoc.rs
```

追加したHandlerとスキーマをOpenAPI定義へ登録する。

## 10. API統合テストを作成する

```text
tests/<resource>_api.rs
```

以下の動作を確認する。

- 正常系
- 入力値エラー
- Not Found
- 絞り込み
- 並び順
- ページング
- 未知のパラメータ

## 11. チェックを実行する

```bash
cargo fmt --all
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

## 12. コミットする

1コミット1目的を守り、原則として1リソースのAPI実装を1コミットにまとめる。

```text
feat(<resource>): <リソース名>CRUD APIの追加
```
