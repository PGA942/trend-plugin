//! 利用者に見せる文の表(PROTOCOL.md §4.6)。ホストの `args.locale`(無ければ `ja`)で引く。
//!
//! 基準は `ja`。`en` / `zh-Hans` / `zh-Hant` / `ko` は同じキーを全部持つ(テストで確かめる)。

rasq_plugin_kit::i18n_catalog! {
    pub(crate) static TEXT = {
        "ja": {
            "reason.cooccur" => "「{tag}」と一緒によく使われます",
            "reason.prefix" => "「{query}」で始まるタグ",
            "reason.fallback" => "定番のタグ",
            "root.desc" => "トレンドサジェストサンプル: 共起辞書 / 前置一致 / 固定タグ(サーバー無し)",
        },
        "en": {
            "reason.cooccur" => "Often used with “{tag}”",
            "reason.prefix" => "Tag starting with “{query}”",
            "reason.fallback" => "Popular tag",
            "root.desc" => "Trending suggestions sample: co-occurrence dictionary / prefix match / fixed tags (no server)",
        },
        "zh-Hans": {
            "reason.cooccur" => "经常与「{tag}」一起使用",
            "reason.prefix" => "以「{query}」开头的标签",
            "reason.fallback" => "常用标签",
            "root.desc" => "热门建议示例：共现词典 / 前缀匹配 / 固定标签（无需服务器）",
        },
        "zh-Hant": {
            "reason.cooccur" => "經常與「{tag}」一起使用",
            "reason.prefix" => "以「{query}」開頭的標籤",
            "reason.fallback" => "常用標籤",
            "root.desc" => "熱門建議範例：共現詞典 / 前綴比對 / 固定標籤（無需伺服器）",
        },
        "ko": {
            "reason.cooccur" => "‘{tag}’와(과) 함께 자주 쓰입니다",
            "reason.prefix" => "‘{query}’(으)로 시작하는 태그",
            "reason.fallback" => "자주 쓰는 태그",
            "root.desc" => "트렌드 추천 샘플: 동시 출현 사전 / 접두사 일치 / 고정 태그(서버 없음)",
        },
    };
}

#[cfg(test)]
mod tests {
    use super::TEXT;
    use std::collections::BTreeSet;

    /// この表の言語ごとのキー(`Catalog::check` は「基準に無いキー」と差し込みの食い違いしか見ないので、訳の抜けはここで見る)。
    fn table_keys() -> Vec<(String, BTreeSet<String>)> {
        let mut out: Vec<(String, BTreeSet<String>)> = Vec::new();
        for line in include_str!("text.rs").lines().map(str::trim) {
            if let Some(tag) = line.strip_prefix('"').and_then(|l| l.strip_suffix("\": {")) {
                out.push((tag.to_string(), BTreeSet::new()));
            } else if let Some((key, _)) = line.strip_prefix('"').and_then(|l| l.split_once("\" =>")) {
                out.last_mut().expect("表の外のキー").1.insert(key.to_string());
            }
        }
        out
    }

    #[test]
    fn every_locale_has_every_key() {
        assert!(TEXT.check().is_empty(), "{:?}", TEXT.check());
        let tables = table_keys();
        let tags: Vec<&str> = tables.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(tags, TEXT.locales());
        assert_eq!(tags, ["ja", "en", "zh-Hans", "zh-Hant", "ko"]);
        for (tag, keys) in &tables[1..] {
            let missing: Vec<_> = tables[0].1.difference(keys).collect();
            assert!(missing.is_empty(), "{tag} に訳が無いキー: {missing:?}");
        }
    }
}
