//! trend-plugin — トレンドサジェスト(`trend.suggest`)のサンプル。
//!
//! サーバー無しで動くローカル規則だけで候補を返す:
//! 1. 共起辞書 — `recent` のタグに紐づく語
//! 2. `query` の前置一致 — 組み込みタグ一覧から
//! 3. フォールバック — 固定タグ(雑談 / RASQ / 写真)
//!
//! 重複と `recent` に既にある語は除き、`limit`(既定 6)で切る。
//!
//! 契約は v0.13(`rasq-sdk-sample/docs/slots.md` §2.7、[`../../CONTRACT.md`](../../CONTRACT.md))。
//! `plugin.manifest` の答えは同じディレクトリの `manifest.json` そのもの(`include_str!`。
//! `rasq check` の往復検査が常に一致する)。
use rasq_plugin_kit::{i18n, rasq_plugin, rrl, serde_json::{json, Value}, unsupported};

mod text;
use text::TEXT;

const DEFAULT_LIMIT: usize = 6;
const MAX_LIMIT: usize = 50;

/// 共起辞書: `recent` にあるタグ → 一緒に出やすいタグ。
const COOCCUR: &[(&str, &[&str])] = &[
    ("打上会", &["写真", "反省会"]),
    ("宇宙", &["ロケット発射", "天文"]),
    ("料理", &["レシピ", "カレー"]),
    ("音楽", &["ライブ", "新曲"]),
    ("ゲーム", &["配信", "攻略"]),
    ("写真", &["カメラ", "散歩"]),
    ("天文", &["流星群", "宇宙"]),
    ("RASQ", &["プラグイン", "AsTell"]),
];

/// `query` の前置一致に使う組み込みタグ。
const TAGS: &[&str] = &[
    "雑談", "RASQ", "写真", "打上会", "反省会", "宇宙", "ロケット発射", "天文", "流星群",
    "料理", "レシピ", "カレー", "音楽", "ライブ", "新曲", "ゲーム", "配信", "攻略",
    "カメラ", "散歩", "プラグイン", "AsTell", "Misskey", "Mastodon", "rust", "wasm",
];

/// 何も無いときの固定タグ。
const FALLBACK: &[&str] = &["雑談", "RASQ", "写真"];

fn root() -> Value {
    rrl::column("root", vec![rrl::text("desc", TEXT.t(&i18n::current(), "root.desc"))])
}

/// `plugin.manifest` の答え = `manifest.json`(名前 / 版 / 説明 / 権限はあちらを直す)。
fn manifest() -> Value {
    rasq_plugin_kit::serde_json::from_str(include_str!("../manifest.json")).expect("manifest.json")
}

fn handle(fn_name: &str, args: Value) -> Result<Value, Value> {
    match fn_name {
        "plugin.manifest" => Ok(manifest()),
        "lifecycle" => Ok(json!({})),
        "trend.suggest" => Ok(suggest(&args)),
        _ => Err(unsupported(fn_name)),
    }
}

rasq_plugin! { scope: "sample.trend:main", root: root, call: handle }

fn suggest(args: &Value) -> Value {
    let loc = i18n::locale_of(args);
    let recent: Vec<&str> = args
        .get("recent")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    let query = args.get("query").and_then(Value::as_str).map(str::trim).unwrap_or("");
    let limit = match args.get("limit") {
        Some(v) => v.as_u64().map(|n| n as usize).filter(|n| *n > 0).unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT),
        None => DEFAULT_LIMIT,
    };

    let mut picked: Vec<Value> = Vec::new();
    let mut push = |text: &str, reason: String| {
        if picked.len() >= limit {
            return;
        }
        if recent.iter().any(|r| r.eq_ignore_ascii_case(text)) {
            return;
        }
        if picked.iter().any(|p| p["text"].as_str().is_some_and(|t| t.eq_ignore_ascii_case(text))) {
            return;
        }
        picked.push(json!({"text": text, "reason": reason}));
    };

    // 1. 共起辞書(recent の順に)。
    for r in &recent {
        for (key, related) in COOCCUR {
            if key.eq_ignore_ascii_case(r) {
                for t in *related {
                    push(t, TEXT.f(loc, "reason.cooccur", &[("tag", *r)]));
                }
            }
        }
    }
    // 2. query の前置一致(大文字小文字は区別しない)。
    if !query.is_empty() {
        let q = query.trim_start_matches('#').to_lowercase();
        if !q.is_empty() {
            for t in TAGS {
                if t.to_lowercase().starts_with(&q) {
                    push(t, TEXT.f(loc, "reason.prefix", &[("query", query)]));
                }
            }
        }
    }
    // 3. フォールバック(常に空にならないように)。
    for t in FALLBACK {
        push(t, TEXT.t(loc, "reason.fallback").to_string());
    }

    json!({ "suggestions": picked })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(args: Value) -> Vec<String> {
        call_native("trend.suggest", args).unwrap()["suggestions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["text"].as_str().unwrap().to_string())
            .collect()
    }

    /// `plugin.manifest` は `manifest.json` そのもの(`rasq check` の往復検査と同じ突き合わせ)。
    #[test]
    fn manifest_matches_manifest_json() {
        let m = call_native("plugin.manifest", json!({})).unwrap();
        let file: Value = rasq_plugin_kit::serde_json::from_str(include_str!("../manifest.json")).unwrap();
        assert_eq!(m, file);
    }

    #[test]
    fn manifest_declares_slot() {
        let m = call_native("plugin.manifest", json!({})).unwrap();
        assert_eq!(m["id"], json!("sample.trend"));
        assert_eq!(m["kind"], json!("trend"));
        assert_eq!(m["slots"], json!(["trend.suggest"]));
        assert_eq!(m["permissions"], json!([]));
        assert_eq!(call_native("trend.other", json!({})).unwrap_err()["code"], json!("C01"));
    }

    #[test]
    fn cooccurrence_from_recent() {
        let t = texts(json!({"recent": ["打上会", "宇宙"]}));
        assert_eq!(&t[..4], ["写真", "反省会", "ロケット発射", "天文"]);
        let r = call_native("trend.suggest", json!({"recent": ["打上会"]})).unwrap();
        assert!(r["suggestions"][0]["reason"].as_str().unwrap().contains("打上会"));
    }

    #[test]
    fn query_prefix() {
        let t = texts(json!({"recent": [], "query": "ロ"}));
        assert_eq!(t[0], "ロケット発射");
        // 大文字小文字を無視、先頭の # も無視
        let t = texts(json!({"recent": [], "query": "#ra"}));
        assert_eq!(t[0], "RASQ");
        // 一致なしでもフォールバックで空にならない
        let t = texts(json!({"recent": [], "query": "zzz"}));
        assert_eq!(t, ["雑談", "RASQ", "写真"]);
    }

    #[test]
    fn excludes_recent_and_dedupes() {
        // 「写真」は recent にあるので出ない。「天文」→「宇宙」も recent なので出ない
        let t = texts(json!({"recent": ["打上会", "写真", "天文", "宇宙"]}));
        assert!(!t.contains(&"写真".to_string()));
        assert!(!t.contains(&"宇宙".to_string()));
        assert!(t.contains(&"反省会".to_string()));
        assert!(t.contains(&"流星群".to_string()));
        // 重複なし
        let t = texts(json!({"recent": ["料理", "料理"], "query": "カ"}));
        let mut sorted = t.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), t.len());
        assert_eq!(t.iter().filter(|s| *s == "カレー").count(), 1);
    }

    #[test]
    fn limit() {
        assert_eq!(texts(json!({"recent": ["打上会", "宇宙"], "limit": 2})), ["写真", "反省会"]);
        // 既定は 6
        assert_eq!(texts(json!({"recent": ["打上会", "宇宙", "料理", "音楽", "ゲーム"]})).len(), 6);
        // 不正な limit は既定
        assert_eq!(texts(json!({"recent": ["打上会", "宇宙", "料理", "音楽", "ゲーム"], "limit": 0})).len(), 6);
        assert_eq!(texts(json!({"recent": ["打上会", "宇宙", "料理", "音楽", "ゲーム"], "limit": "x"})).len(), 6);
        assert_eq!(texts(json!({"recent": [], "limit": 1})), ["雑談"]);
    }

    #[test]
    fn empty_input_falls_back() {
        assert_eq!(texts(json!({})), ["雑談", "RASQ", "写真"]);
        assert_eq!(texts(json!({"recent": [], "query": ""})), ["雑談", "RASQ", "写真"]);
        assert_eq!(texts(Value::Null), ["雑談", "RASQ", "写真"]);
        // フォールバック語も recent にあるなら除外(それでも他の候補は出る)
        assert_eq!(texts(json!({"recent": ["雑談", "RASQ", "写真"]})), ["プラグイン", "AsTell", "カメラ", "散歩"]);
    }

    /// `reason` は `args.locale` の言語(タグそのものは辞書のまま)。
    #[test]
    fn reasons_follow_locale() {
        let _ = i18n::take_misses();
        let r = call_native("trend.suggest", json!({"locale": "en", "recent": ["打上会"]})).unwrap();
        assert_eq!(r["suggestions"][0], json!({"text": "写真", "reason": "Often used with “打上会”"}));
        let r = call_native("trend.suggest", json!({"locale": "zh-Hant-TW", "query": "ロ"})).unwrap();
        assert_eq!(r["suggestions"][0], json!({"text": "ロケット発射", "reason": "以「ロ」開頭的標籤"}));
        let r = call_native("trend.suggest", json!({"locale": "ko", "limit": 1})).unwrap();
        assert_eq!(r["suggestions"][0], json!({"text": "雑談", "reason": "자주 쓰는 태그"}));
        let r = call_native("trend.suggest", json!({"locale": "zh-Hans", "limit": 1})).unwrap();
        assert_eq!(r["suggestions"][0]["reason"], "常用标签");
        // locale が無ければ日本語
        let r = call_native("trend.suggest", json!({"limit": 1})).unwrap();
        assert_eq!(r["suggestions"][0]["reason"], "定番のタグ");
        assert!(i18n::take_misses().is_empty());
    }

    #[test]
    fn manifest_has_every_locale() {
        let m = call_native("plugin.manifest", json!({})).unwrap();
        for tag in ["en", "zh-Hans", "zh-Hant", "ko"] {
            assert!(m["locales"][tag]["name"].is_string(), "{tag}");
            assert!(m["locales"][tag]["description"].is_string(), "{tag}");
        }
    }

    #[test]
    fn bad_input_does_not_panic() {
        let r = call_native("trend.suggest", json!({"recent": "x", "query": 1, "limit": -5})).unwrap();
        assert!(!r["suggestions"].as_array().unwrap().is_empty());
        let r = call_native("trend.suggest", json!({"recent": [1, null, "", "  "]})).unwrap();
        assert_eq!(r["suggestions"].as_array().unwrap().len(), 3);
    }
}
