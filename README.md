# trend-plugin — ローカル規則のトレンドサジェスト(スロット `trend.suggest`)

検索ホームのトレンド節に **サジェスト行**を出すサンプル。サーバーには一切問い合わせず、
ローカルの規則だけで候補を作る:

1. **共起辞書** — `recent`(検索ホームが今持っているトレンド語)に紐づく語
   (例: 打上会 → 写真 / 反省会、宇宙 → ロケット発射 / 天文、料理 → レシピ / カレー、音楽 → ライブ / 新曲、ゲーム → 配信 / 攻略)
2. **前置一致** — `query` が空でなければ組み込みタグ一覧から前方一致(大文字小文字と先頭の `#` は無視)
3. **固定タグ** — 雑談 / RASQ / 写真(候補が空にならないように)

重複と `recent` に既にある語は除き、`limit`(既定 6、最大 50)で切る。各候補には `reason`(理由の短文)を添える。

## このリポジトリの形(`rasq` ワークスペース)

`rasq-sdk-sample` の `docs/cli.md` §2 のワークスペース。`rasq init` / `rasq new trend.suggest` が作る形に合わせてある。

```
rasq.toml                                # schema = "rasq.workspace/1"、[[plugins]] に name/slot/id
Cargo.toml                               # [workspace](rasq CLI が rasq.toml から作り直す)
kit/                                     # rasq-plugin-kit の写し(正典は SDK の rust/rasq-plugin-kit)
plugins/trend-plugin/
  manifest.json                          # plugin.manifest の答えそのもの
  src/lib.rs                             # manifest.json を include_str! で返し、trend.suggest を実装
dist/                                    # rasq build --raw の出力(.rasqpkg と生の .wasm)
```

- 契約の正典は [`CONTRACT.md`](CONTRACT.md) が指す `rasq-sdk-sample/docs/slots.md` §2.7。
- マニフェストは `plugins/trend-plugin/manifest.json` の 1 か所だけ。`src/lib.rs` は `include_str!` で
  そのまま返すので、`rasq check` の往復検査(wasm の `plugin.manifest` == `manifest.json`)が常に一致する。
  名前 / 版 / 説明 / 権限を変えるときは `manifest.json` を直す。

## 契約(`docs/slots.md` §2.7)

| fn | args | 戻り |
|---|---|---|
| `plugin.manifest` | `{}` | `manifest.json`(`{id: "sample.trend", kind: "trend", slots: ["trend.suggest"], permissions: []}`) |
| `lifecycle` | `{kind, surface?}` | `{}` |
| `trend.suggest` | `{recent: [string], query?: string, limit?: number}` | `{suggestions: [{text, reason}]}` |
| その他 | | `C01` |

ホスト関数は使わない(`permissions: []`)。

### 候補の作り方

| 順 | 規則 | `reason` |
|---|---|---|
| 1 | 共起辞書(`recent` の順に引く) | 「〈recent の語〉と一緒によく使われます」 |
| 2 | `query` の前置一致(先頭の `#` と大文字小文字は無視) | 「〈query〉で始まるタグ」 |
| 3 | 固定タグ(雑談 / RASQ / 写真) | 「定番のタグ」 |

`recent` に既にある語と、既に選んだ語は入れない(どちらも大文字小文字を区別しない)。
`limit` は既定 6・最大 50。`0` や数でない値は既定に落とす。`recent` が配列でない / 要素が文字列でない
といった壊れた入力でもパニックしない(空として扱う)。

## ビルドと確認

`rasq` CLI は `rasq-sdk-sample` から組む(`cd rust && cargo build --release -p rasq-cli`)。

```sh
rasq check                 # wasm32 ビルド → export 検査 → manifest 往復 → スキーマ / 登録簿 → cargo test
rasq test                  # cargo test だけ
rasq build --raw           # dist/sample.trend-0.1.0.rasqpkg と dist/trend_plugin.wasm
rasq inspect dist/sample.trend-0.1.0.rasqpkg
```

`cargo test` は native で走るので、wasm でだけ起きる問題(時刻 / 乱数 / スレッド、`docs/cli.md` §5 の落とし穴)は
**ビルド済みの wasm をそのまま叩く** `rasq call` で確かめる:

```sh
rasq call trend-plugin plugin.manifest
rasq call trend-plugin trend.suggest --args '{"recent":["打上会","宇宙"],"query":"ロ","limit":4}'
# → {"suggestions": [{"text": "写真", "reason": "「打上会」と一緒によく使われます"},
#                    {"text": "反省会", "reason": "「打上会」と一緒によく使われます"},
#                    {"text": "ロケット発射", "reason": "「宇宙」と一緒によく使われます"},
#                    {"text": "天文", "reason": "「宇宙」と一緒によく使われます"}]}

rasq call trend-plugin trend.suggest --args '{}'
# → {"suggestions": [{"text": "雑談", …}, {"text": "RASQ", …}, {"text": "写真", …}]}

rasq call trend-plugin trend.other --args '{}'   # → C01(exit 1)
```

## 端末での試し方(rasq-client)

1. `rasq dev`(または `rasq build --raw`)が出す `dist/` の絶対パスを控える。
2. サイドバー → **WASMプラグイン試験 (dev)** → `.wasm のパスからインポート` に `trend_plugin.wasm` の絶対パスを貼って **読み込み**。
3. **スロットに登録**(権限は無し)。
4. **検索ホームのトレンド節**を開く。トレンド語が無くても「雑談 / RASQ / 写真」が出れば OK。
   トレンドに「打上会」等の辞書語があれば「写真 / 反省会」が先頭に出る。検索欄に「ロ」と打つと「ロケット発射」が出る。

## 掲載(審査)に出す

`rasq build` が出す `dist/sample.trend-0.1.0.rasqpkg` がそのまま掲載できる形
(zstd tar + `bundle.json` `rasq.bundle/1` + `SHA512SUMS`)。`rasq publish --central <url> --dry-run` で
Central への 3 要求を書き出せる。開発者鍵の署名(`*.sig.json`)はサイドロード用で、リポジトリには入れない。
