//! Offline contextual Help (`#263`): embedded Markdown, search, context routing.
//!
//! Articles live under `docs/help/{en,uk}/` and are compiled into the binary.
//! No network, no LLM, no dependency on a running node.

mod render;
mod search;

use aira_desktop_runtime::UiLang;

use crate::lexicon::HelpId;

pub use render::render_markdown_plain;
pub use search::search_help_ids;

/// One embedded Help article.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpArticle {
    pub id: HelpId,
    pub title: String,
    pub markdown: &'static str,
}

/// Load the embedded Markdown source for a topic (fail-closed to Start).
pub fn article_markdown(lang: UiLang, id: HelpId) -> &'static str {
    match lang {
        UiLang::En => article_en(id),
        UiLang::Uk => article_uk(id),
    }
}

/// Parsed article with a display title (first `#` heading or the help id).
pub fn load_article(lang: UiLang, id: HelpId) -> HelpArticle {
    let markdown = article_markdown(lang, id);
    let title = first_heading(markdown).unwrap_or_else(|| id.as_str().to_string());
    HelpArticle {
        id,
        title,
        markdown,
    }
}

fn first_heading(md: &str) -> Option<String> {
    for line in md.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("# ") {
            let title = rest.trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    None
}

fn article_en(id: HelpId) -> &'static str {
    match id {
        HelpId::Start => include_str!("../../../../docs/help/en/start.md"),
        HelpId::WorkSubmit => include_str!("../../../../docs/help/en/work.submit.md"),
        HelpId::WorkResult => include_str!("../../../../docs/help/en/work.result.md"),
        HelpId::WorkWaiting => include_str!("../../../../docs/help/en/work.waiting.md"),
        HelpId::ModelSelect => include_str!("../../../../docs/help/en/model.select.md"),
        HelpId::ModelUnavailable => include_str!("../../../../docs/help/en/model.unavailable.md"),
        HelpId::NetworkConnect => include_str!("../../../../docs/help/en/network.connect.md"),
        HelpId::NetworkReachability => {
            include_str!("../../../../docs/help/en/network.reachability.md")
        }
        HelpId::NetworkTrust => include_str!("../../../../docs/help/en/network.trust.md"),
        HelpId::SettingsApply => include_str!("../../../../docs/help/en/settings.apply.md"),
        HelpId::NodeLifecycle => include_str!("../../../../docs/help/en/node.lifecycle.md"),
    }
}

fn article_uk(id: HelpId) -> &'static str {
    match id {
        HelpId::Start => include_str!("../../../../docs/help/uk/start.md"),
        HelpId::WorkSubmit => include_str!("../../../../docs/help/uk/work.submit.md"),
        HelpId::WorkResult => include_str!("../../../../docs/help/uk/work.result.md"),
        HelpId::WorkWaiting => include_str!("../../../../docs/help/uk/work.waiting.md"),
        HelpId::ModelSelect => include_str!("../../../../docs/help/uk/model.select.md"),
        HelpId::ModelUnavailable => include_str!("../../../../docs/help/uk/model.unavailable.md"),
        HelpId::NetworkConnect => include_str!("../../../../docs/help/uk/network.connect.md"),
        HelpId::NetworkReachability => {
            include_str!("../../../../docs/help/uk/network.reachability.md")
        }
        HelpId::NetworkTrust => include_str!("../../../../docs/help/uk/network.trust.md"),
        HelpId::SettingsApply => include_str!("../../../../docs/help/uk/settings.apply.md"),
        HelpId::NodeLifecycle => include_str!("../../../../docs/help/uk/node.lifecycle.md"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalog_id_has_embedded_en_and_uk() {
        for id in HelpId::catalog() {
            let en = load_article(UiLang::En, *id);
            let uk = load_article(UiLang::Uk, *id);
            assert!(!en.markdown.trim().is_empty(), "empty en {}", id.as_str());
            assert!(!uk.markdown.trim().is_empty(), "empty uk {}", id.as_str());
            assert!(en.markdown.contains("## "), "en sections {}", id.as_str());
            assert!(uk.markdown.contains("## "), "uk sections {}", id.as_str());
            assert!(!en.title.is_empty());
            assert!(!uk.title.is_empty());
        }
    }

    #[test]
    fn search_finds_work_submit() {
        let hits = search_help_ids(UiLang::En, "Ctrl+Enter");
        assert!(
            hits.contains(&HelpId::WorkSubmit),
            "expected work.submit in {hits:?}"
        );
    }

    #[test]
    fn render_strips_heading_markers() {
        let plain = render_markdown_plain("# Title\n\nHello **world**.\n");
        assert!(plain.contains("Title"));
        assert!(plain.contains("Hello world"));
        assert!(!plain.contains("**"));
    }
}
