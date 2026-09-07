# Offline Help articles

**Canon:** [`desktop-ux.md`](../desktop-ux.md) §7 · Phase O `#263` shell / `#264` seed.

## Layout

```text
docs/help/{en,uk}/<help_id>.md
```

`<help_id>` matches [`HelpId`](../../crates/aira-desktop/src/lexicon.rs) (`start`, `work.submit`, …).

## Article format

EN headings:

1. `# Title`
2. `## What is this?`
3. `## What to do?`
4. `## What should happen?`
5. `## If it failed?`
6. `## Related`

UK headings: `Що це?` / `Що зробити?` / `Що має відбутися?` / `Що робити, якщо не вдалося?` / `Пов'язані теми`.

## Cross-links

Related bullets use stable tokens `help:<id>` (no remote URLs). Link check: `crates/aira-desktop/src/help/links.rs`.

## Embed

`aira-desktop` embeds sources with `include_str!` — rebuild Desktop after editing articles.
