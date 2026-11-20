# 認証

環境変数で有効にするオプションの認証。既定は無効で、設定しなければ従来どおりすべてのリクエストを通す。

家計データはユーザーごとに分けない。認証は「アクセスできる人を限る門」として働き、登録したユーザーは全員が同じ家計データを扱う。

## モード

| `KAKEIBO_AUTH_MODE` | 内容 |
|---|---|
| `none`（既定） | 認証しない |
| `local` | アプリが持つユーザー名とパスワードでログインする |

リバースプロキシのヘッダー認証（`proxy`）とアプリ内 OIDC（`oidc`）は、必要になったときに追加する。`local` ではパスワードに加えてパスキーでもログインできる。

## 環境変数

| 変数 | 必須 | 既定 | 内容 |
|---|---|---|---|
| `KAKEIBO_AUTH_MODE` | | `none` | `none` または `local` |
| `KAKEIBO_PUBLIC_URL` | `local` のとき | | ブラウザで開く URL。`Origin` の照合と、Cookie に `Secure` を付けるか（https のとき付ける）の判定に使う |
| `KAKEIBO_AUTH_SESSION_TTL` | | `30d` | セッションの有効期間。秒数、または `m`・`h`・`d` を付けた値 |
| `KAKEIBO_AUTH_DATABASE` | | `auth.db` | ユーザーとセッションを保存する SQLite ファイル |
| `KAKEIBO_AUTH_MIN_PASSWORD_LENGTH` | | `8` | パスワードの最小文字数（1〜256）。ユーザー登録とパスワード変更のときに確かめる |

設定が不正なときは、原因の変数名をログに出して起動を止める。

## データの置き場所

ユーザーとセッションは、家計データの `kakeibo.db` とは別の `auth.db` に保存する（マイグレーションは `apps/api/migrations_auth/`）。フルバックアップの取得と復元は `kakeibo.db` だけを対象にするため、バックアップを復元してもパスワードやセッションは巻き戻らない。

## パスワード

- Argon2id（OWASP 推奨の既定値：19 MiB、2 回、並列度 1）でハッシュ化する。salt はパスワードごとに生成し、PHC 形式のハッシュ文字列に含める。
- 長さは `KAKEIBO_AUTH_MIN_PASSWORD_LENGTH`（既定 8）以上、256 文字以下。文字種の強制はしない。CLI も同じ環境変数を読む。
- 存在しないユーザー名でもダミーのハッシュと照合し、応答時間からユーザー名の有無を推測させない。
- ユーザー名は大文字と小文字を区別しない。

## セッション

- ログインに成功すると 32 バイトの乱数トークンを発行し、Cookie `kakeibo_session` で返す。DB にはトークンの SHA-256 だけを保存する。
- Cookie の属性は `HttpOnly; SameSite=Lax; Path=/`。`KAKEIBO_PUBLIC_URL` が https なら `Secure` も付ける。
- 使われているセッションは、1 時間に 1 回まで有効期限を延ばし、Cookie を送り直す。期限切れのセッションは参照時と 1 時間ごとの掃除で削除する。
- ログイン時に既存のセッション Cookie があれば、そのセッションは削除する。
- パスワードを変えるか、ユーザーを削除すると、そのユーザーのセッションはすべて消える。

## 保護する範囲

`local` のときだけミドルウェアを入れる。

- セッションが必要：`/api/*`（`/api/auth/*` を除く）、`/docs`
- 公開：`/api/auth/*`、`/health`、SPA の静的ファイル（ログイン画面を表示するため）
- 状態を変えるリクエスト（GET・HEAD・OPTIONS 以外）で `Origin` が `KAKEIBO_PUBLIC_URL` と一致しないものは 403 にする。`Origin` がなく `Sec-Fetch-Site: cross-site` のものも拒否する。

## ログインの回数制限

同じユーザー名で 15 分以内に 5 回失敗すると、15 分間ロックして 429 を返す。リバースプロキシの後ろでは全員の IP が同じになりうるため、IP ではなくユーザー名で数える。カウンタはメモリ上にあり、再起動でリセットされる。

- 照合を始める前に試行の枠を予約し、照合中の試行も回数に含める。同時に送られたリクエストでも、15 分あたりの照合は 5 回までになる
- 覚えておくユーザー名は 1024 件まで。満杯のときは、ロック中と照合中のものを残して、いちばん古い記録から消す。消せるものがなければ、新しいユーザー名のログインは 429 で断る（記録しないまま照合を許すと、回数制限をすり抜けられるため）
- 65 文字以上のユーザー名は登録できないので、照合も記録もせずに 401 を返す
- Argon2 の照合は 1 回あたり約 19 MiB を使うため、同時に動かすのは 4 件までにする

## API

| メソッドとパス | 内容 |
|---|---|
| `GET /api/auth/session` | `{ mode, authenticated, user }`。認証が無効なら `authenticated` は常に true |
| `POST /api/auth/login` | `{ username, password }`。成功で 204 と Set-Cookie、失敗で 401、ロック中は 429。認証が無効なら 404 |
| `POST /api/auth/logout` | セッションを削除して Cookie を消す。認証が無効なら 404 |

## パスキー

`local` のログイン方法として WebAuthn のパスキーを使える（webauthn-rs）。ログインしたあとに設定画面の「ログイン > パスキー」から登録し、以後はログイン画面の「パスキーでログイン」で入れる。

- Relying Party は `KAKEIBO_PUBLIC_URL` から決める。オリジンはそのURL、RP ID はそのホスト名。ホストが IP アドレスなどで RP ID にできないときは、警告をログに出してパスキーだけを無効にする（`/api/auth/session` の `passkey` が false になる）。ブラウザは HTTPS か `localhost` でしかパスキーを使えない。
- ログイン画面ではユーザー名を聞かないため、登録時に discoverable credential（`residentKey: required`）を求め、ログインは discoverable authentication で行う。credential の user handle にはユーザー ID（UUID の 32 桁表記）を入れる。
- ユーザー確認（生体認証や画面ロック）を必須にする。パスキー単独で多要素とみなせるため、パスワードとの併用は求めない。
- 登録・ログインの途中状態はメモリ上に 5 分だけ持ち、ID で 1 回だけ取り出せる。同時に 1024 件まで持ち、超えたらいちばん古いものから捨てる（ログインの challenge は誰でも要求できるため、満杯で断ると、だれでもパスキーでのログインを止められてしまう）。
- パスキーは `auth.db` の `passkeys` テーブルに保存する（credential ID、名前、webauthn-rs の Passkey を JSON にしたもの、登録日時、最終使用日時）。ユーザーを削除すると、そのユーザーのパスキーも消える。
- 自分のパスキーだけを一覧・削除できる。パスワードを変えてもパスキーは残る。

| メソッドとパス | 内容 |
|---|---|
| `POST /api/auth/passkey/challenge` | ログイン用の options と `challenge_id` を返す。認証不要 |
| `POST /api/auth/passkey/login` | `{ challenge_id, credential }`。成功で 204 と Set-Cookie、失敗で 401、期限切れや使用済みの challenge は 400 |
| `GET /api/passkeys` | 自分のパスキーの一覧 |
| `POST /api/passkeys/registration` | 登録用の options と `challenge_id` を返す |
| `POST /api/passkeys` | `{ challenge_id, name, credential }`。名前は 1〜64 文字 |
| `DELETE /api/passkeys/{id}` | 自分のパスキーを削除する |

`/api/passkeys` は通常の API と同じくセッションが必要。

## ユーザー管理（CLI）

```bash
kakeibo-app auth add-user <username>       # パスワードを 2 回入力
kakeibo-app auth set-password <username>   # パスワードを変え、全セッションを無効にする
kakeibo-app auth list-users
kakeibo-app auth remove-user <username>
```

`add-user` と `set-password` は `--password-stdin` を付けると標準入力の 1 行目をパスワードとして読む。`auth.db` の場所は `KAKEIBO_AUTH_DATABASE` に従う。

`local` で起動したときにユーザーが 0 人なら、警告をログに出す（API はすべて 401 になる）。初回にセットアップ画面は出さない。公開した直後に他人に先に登録されるのを防ぐため。
