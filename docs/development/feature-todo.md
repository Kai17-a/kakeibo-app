# 追加機能 実装TODO

`docs/development/feature-rollout-progress.md` の仕様棚卸しから出た8機能（資産・口座残高管理／収入明細一覧UI・フィルタ／予算管理／Webhookイベント種別フィルタ／グラフ分析の強化・資産推移グラフ／DBバックアップ）は、6つの実装単位としてすべて完了した。各単位の設計判断・実装内容・レビュー指摘の詳細は `docs/development/feature-rollout-progress.md` の「単位1」〜「単位6」セクションを参照。

対象外（実装しない）: レシート画像添付、定期支出の支払日リマインド、認証・マルチユーザー対応、DB復元のアプリ内実装（運用手順はREADME参照）、貯蓄目標金額を登録する専用機能（残高推移グラフのみ実装）。

## 今後、同種の機能追加TODOを書く際の進め方（参考）

1件ずつ、以下の流れで進める。

1. 対象範囲を実装する
2. 実装した差分をレビューする（設計意図との整合性、エッジケース漏れ、既存パターンからの逸脱がないか）
3. 指摘事項を解消する
4. `cd apps/api && cargo fmt --all && cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings` と、リポジトリルートで `mise run web-ci` をグリーンにする
5. `mise run run` / `mise run web-dev` で実際に画面を操作し、golden pathとエッジケースを手動確認する
6. 無関係な既存の未コミット差分を巻き込まないよう、対象ファイルのみをステージして1機能1コミットで記録する
7. 本ファイルへ新規TODOセクションを追記し、完了後は削除して `feature-rollout-progress.md` のステータス表と履歴セクションを更新する
