//! Minimal offline Markdown → plain text for egui Labels (no HTML / no scripts).

/// Strip common Markdown chrome into readable plain text.
pub fn render_markdown_plain(md: &str) -> String {
    let mut out = String::with_capacity(md.len());
    for line in md.lines() {
        let mut t = line.trim_end().to_string();
        if let Some(rest) = t.strip_prefix("#### ") {
            t = rest.to_string();
        } else if let Some(rest) = t.strip_prefix("### ") {
            t = rest.to_string();
        } else if let Some(rest) = t.strip_prefix("## ") {
            t = rest.to_string();
        } else if let Some(rest) = t.strip_prefix("# ") {
            t = rest.to_string();
        }
        t = t.replace("**", "");
        t = t.replace("__", "");
        t = t.replace('`', "");
        if let Some(rest) = t.strip_prefix("- ") {
            t = format!("• {rest}");
        } else if let Some(rest) = t.strip_prefix("* ") {
            t = format!("• {rest}");
        }
        out.push_str(&t);
        out.push('\n');
    }
    out
}
