# Analyze-299 — Living Spec Matrix

| ТЗ / канон | Модуль | Тести |
|------------|--------|-------|
| Seed 4+Related sections | `docs/help/{en,uk}/**` | `every_seed_article_valid_en_uk` |
| `help:<id>` resolve | `help::links::extract_help_links` | `extract_skips_unknown_and_keeps_order` |
| No orphan topics | link graph | `every_catalog_id_is_linked_from_somewhere` |
| EN/UK related parity | seed articles | `related_ids_match_across_langs` |
| Related chips UI | `ui_help_panel` | compile + i18n `help_related` |
| RFC-D | `AIRA-RFC-0155` | `phase_o_doc` |
| QUEUE | `#264` DONE → `#265` | `phase_o_queue_264_done_265_open` |
