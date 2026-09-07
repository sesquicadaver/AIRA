//! Offline Help search over embedded article title + body.

use aira_desktop_runtime::UiLang;

use crate::lexicon::HelpId;

use super::load_article;

/// Return catalog topics whose id/title/body contain `query` (case-insensitive).
/// Empty query → full catalog order.
pub fn search_help_ids(lang: UiLang, query: &str) -> Vec<HelpId> {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return HelpId::catalog().to_vec();
    }
    HelpId::catalog()
        .iter()
        .copied()
        .filter(|id| {
            let art = load_article(lang, *id);
            art.id.as_str().to_ascii_lowercase().contains(&q)
                || art.title.to_ascii_lowercase().contains(&q)
                || art.markdown.to_ascii_lowercase().contains(&q)
        })
        .collect()
}
