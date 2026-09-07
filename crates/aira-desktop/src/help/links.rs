//! Offline Help cross-link check (`#264`).
//!
//! Articles may reference other catalog topics as `help:<id>` (plain text).
//! Tokens resolve by longest [`HelpId`] prefix so trailing punctuation (e.g.
//! `help:node.lifecycle.`) does not invent unknown ids.

use std::collections::BTreeSet;

use crate::lexicon::HelpId;

/// Resolve the longest catalog id prefix at the start of `rest`.
fn longest_catalog_prefix(rest: &str) -> Option<HelpId> {
    let mut best: Option<HelpId> = None;
    let mut best_len = 0usize;
    for id in HelpId::catalog() {
        let key = id.as_str();
        if rest.starts_with(key) && key.len() > best_len {
            best_len = key.len();
            best = Some(*id);
        }
    }
    best
}

/// Extract resolved `help:<id>` links (order preserved, unique).
pub fn extract_help_links(markdown: &str) -> Vec<HelpId> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut search_from = 0usize;
    while let Some(rel) = markdown[search_from..].find("help:") {
        let abs = search_from + rel;
        let rest = &markdown[abs + 5..];
        if let Some(id) = longest_catalog_prefix(rest) {
            if seen.insert(id.as_str()) {
                out.push(id);
            }
            search_from = abs + 5 + id.as_str().len();
        } else {
            search_from = abs + 5;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_desktop_runtime::UiLang;

    use crate::help::load_article;

    /// Raw `help:` payloads that do **not** resolve to a catalog id (fail-closed).
    fn extract_unknown_help_link_keys(markdown: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut search_from = 0usize;
        while let Some(rel) = markdown[search_from..].find("help:") {
            let abs = search_from + rel;
            let rest = &markdown[abs + 5..];
            if let Some(id) = longest_catalog_prefix(rest) {
                search_from = abs + 5 + id.as_str().len();
                continue;
            }
            let mut end = 0usize;
            for (i, c) in rest.char_indices() {
                if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
                    end = i + c.len_utf8();
                } else {
                    break;
                }
            }
            if end > 0 {
                out.push(rest[..end].trim_end_matches('.').to_string());
            }
            search_from = abs + 5 + end.max(1);
        }
        out
    }

    fn required_en_sections() -> &'static [&'static str] {
        &[
            "## What is this?",
            "## What to do?",
            "## What should happen?",
            "## If it failed?",
            "## Related",
        ]
    }

    fn required_uk_sections() -> &'static [&'static str] {
        &[
            "## Що це?",
            "## Що зробити?",
            "## Що має відбутися?",
            "## Що робити, якщо не вдалося?",
            "## Пов'язані теми",
        ]
    }

    fn validate_seed_article(lang: UiLang, id: HelpId) -> Result<(), String> {
        let art = load_article(lang, id);
        let md = art.markdown;
        let sections = match lang {
            UiLang::En => required_en_sections(),
            UiLang::Uk => required_uk_sections(),
        };
        for s in sections {
            if !md.contains(s) {
                return Err(format!(
                    "{} {}: missing section `{s}`",
                    lang_tag(lang),
                    id.as_str()
                ));
            }
        }
        if md.trim().len() < 500 {
            return Err(format!(
                "{} {}: seed body too short ({})",
                lang_tag(lang),
                id.as_str(),
                md.trim().len()
            ));
        }
        let links = extract_help_links(md);
        if links.is_empty() {
            return Err(format!(
                "{} {}: no help: related links",
                lang_tag(lang),
                id.as_str()
            ));
        }
        let unknown = extract_unknown_help_link_keys(md);
        if !unknown.is_empty() {
            return Err(format!(
                "{} {}: unknown help link(s) {unknown:?}",
                lang_tag(lang),
                id.as_str()
            ));
        }
        for link in &links {
            if *link == id {
                return Err(format!(
                    "{} {}: self-link help:{} is not allowed",
                    lang_tag(lang),
                    id.as_str(),
                    link.as_str()
                ));
            }
        }
        Ok(())
    }

    fn lang_tag(lang: UiLang) -> &'static str {
        match lang {
            UiLang::En => "en",
            UiLang::Uk => "uk",
        }
    }

    fn related_ids_match_across_langs(id: HelpId) -> Result<(), String> {
        let en: BTreeSet<_> = extract_help_links(load_article(UiLang::En, id).markdown)
            .into_iter()
            .map(HelpId::as_str)
            .collect();
        let uk: BTreeSet<_> = extract_help_links(load_article(UiLang::Uk, id).markdown)
            .into_iter()
            .map(HelpId::as_str)
            .collect();
        if en != uk {
            return Err(format!(
                "{}: related mismatch en={:?} uk={:?}",
                id.as_str(),
                en,
                uk
            ));
        }
        Ok(())
    }

    #[test]
    fn extract_skips_unknown_and_keeps_order() {
        let md =
            "see help:work.submit and help:nope and help:node.lifecycle. again help:work.submit";
        let ids = extract_help_links(md);
        assert_eq!(
            ids,
            vec![HelpId::WorkSubmit, HelpId::NodeLifecycle],
            "{ids:?}"
        );
        let unknown = extract_unknown_help_link_keys(md);
        assert_eq!(unknown, vec!["nope".to_string()]);
    }

    #[test]
    fn every_seed_article_valid_en_uk() {
        for id in HelpId::catalog() {
            validate_seed_article(UiLang::En, *id).unwrap();
            validate_seed_article(UiLang::Uk, *id).unwrap();
            related_ids_match_across_langs(*id).unwrap();
        }
    }

    #[test]
    fn every_catalog_id_is_linked_from_somewhere() {
        let mut targets = BTreeSet::new();
        for id in HelpId::catalog() {
            for lang in [UiLang::En, UiLang::Uk] {
                for link in extract_help_links(load_article(lang, *id).markdown) {
                    targets.insert(link.as_str());
                }
            }
        }
        for id in HelpId::catalog() {
            assert!(
                targets.contains(id.as_str()),
                "orphan help topic never linked: {}",
                id.as_str()
            );
        }
    }
}
