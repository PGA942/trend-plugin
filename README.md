# trend-plugin — ローカル規則のトレンドサジェスト(`trend.suggest`)

検索ホームのトレンド節に **サジェスト行**を出すサンプル。サーバーには一切問い合わせず、
ローカルの規則だけで候補を作る:

1. **共起辞書** — `recent`(検索ホームが今持っているトレンド語)に紐づく語
   (例: 打上会 → 写真 / 反省会、宇宙 → ロケット発射 / 天文、料理 → レシピ / カレー、音楽 → ライブ / 新曲、ゲーム → 配信 / 攻略)
2. **前置一致** — `query` が空でなければ組み込みタグ一覧から前方一致(大文字小文字と先頭の `#` は無視)
3. **固定タグ** — 雑談 / RASQ / 写真(候補が空にならないように)

重複と `recent` に既にある語は除き、`limit`(既定 6、最大 50)で切る。各候補には `reason`(理由の短文)を添える。

## 契約(CONTRACT.md §2.7)

| fn | args | 戻り |
|---|---|---|
| `plugin.manifest` | `{}` | `{id: "sample.trend", kind: "trend", slots: ["trend.suggest"], permissions: []}` |
| `trend.suggest` | `{recent: [string], query?: string, limit?: number}` | `{suggestions: [{text, reason}]}` |
| その他 | | `C01` |

## ビルド / テスト

```sh
cargo test -p trend-plugin
cargo build --release --target wasm32-unknown-unknown -p trend-plugin
# → target/wasm32-unknown-unknown/release/trend_plugin.wasm(./build.sh なら dist/)
```

## 端末での確認(rasq-client)

1. サイドバー → **WASMプラグイン試験 (dev)** → `.wasm のパスからインポート` に `trend_plugin.wasm` の絶対パス → **読み込み**。
2. **スロットに登録**(権限は無し)。
3. **検索ホームのトレンド節**を開く。トレンド語が無くても「雑談 / RASQ / 写真」が出れば OK。
   トレンドに「打上会」等の辞書語があれば「写真 / 反省会」が先頭に出る。検索欄に「ロ」と打つと「ロケット発射」が出る。

---

## このリポジトリの形

- クレートはルート(`src/`)、共通部品 [`rasq-plugin-kit`](kit) は `kit/` に同梱(開発用モノレポから写している)。
- 契約は [`CONTRACT.md`](CONTRACT.md)(スロットプラグイン契約 v0.1)。
- ビルド済み `.wasm` は [`dist/`](dist)。`cargo build --release --target wasm32-unknown-unknown` で更新。
