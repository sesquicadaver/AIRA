//! Work model selector projection (`#373`).
//!
//! Search + Compatible / Show-all filter + «shown N of M» counter. Selection is
//! by stable id and **survives** being hidden by filter/search. Filter hides
//! rows on screen; it does not delete them from the snapshot (`M` is always
//! the full snapshot length).

use crate::model_catalog::{CatalogProjectionRow, CatalogSource};
use crate::ollama_fitness::ModelFitnessState;

/// Map a shared catalog row to a provisional fitness for the Compatible filter (`#373`).
///
/// Full tags/show evaluation remains `#372` inputs; until metadata is attached to every
/// row, host Ollama with an exact CLI name stays actionable (Prepare/Run), local files
/// are incompatible with Process Work, and missing CLI is NeedsAction.
pub fn fitness_from_projection_row(row: &CatalogProjectionRow) -> ModelFitnessState {
    match row.source {
        CatalogSource::LocalFile => ModelFitnessState::Incompatible,
        CatalogSource::HostOllama if row.cli_name.is_none() => ModelFitnessState::NeedsAction,
        CatalogSource::HostOllama if row.available => ModelFitnessState::PrepareAndRun,
        CatalogSource::HostOllama => ModelFitnessState::NeedsAction,
    }
}

/// Default Work filter vs operator override (`#373`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModelSelectorFilterMode {
    /// «Compatible with this task» — [`ModelFitnessState::passes_compatible_filter`].
    #[default]
    Compatible,
    /// «Show all» — every snapshot row (search still applies).
    ShowAll,
}

/// One snapshot row fed into the selector (`#373`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSelectorRow {
    /// Stable id (`model_ref`); selection key.
    pub id: String,
    /// Text matched by search (CLI name / display). Case-insensitive.
    pub search_text: String,
    pub fitness: ModelFitnessState,
}

/// Projected view for one paint of the Work selector (`#373`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSelectorView {
    /// Ids visible after search + filter (order preserved from snapshot).
    pub visible_ids: Vec<String>,
    /// `N` — visible count.
    pub shown: usize,
    /// `M` — full snapshot length (never shrinks when filtering).
    pub total: usize,
    /// Current selection id (may be hidden).
    pub selected_id: Option<String>,
    /// Whether [`Self::selected_id`] appears in [`Self::visible_ids`].
    pub selected_visible: bool,
    /// True while metadata is loading and the compatible list is empty.
    pub checking_compatibility: bool,
}

/// Format «shown N of M» counter digits (UI localizes the words in `#375`).
pub fn model_selector_counter_label(shown: usize, total: usize) -> String {
    format!("{shown} / {total}")
}

/// Project search + filter without mutating the catalog snapshot (`#373`).
///
/// - `selected_id` is preserved even when the row is filtered out.
/// - Search is case-insensitive and does not merge distinct ids.
/// - `metadata_loading`: when Compatible yields no rows, set
///   [`ModelSelectorView::checking_compatibility`] instead of implying an empty catalog.
pub fn project_model_selector(
    rows: &[ModelSelectorRow],
    query: &str,
    mode: ModelSelectorFilterMode,
    selected_id: Option<&str>,
    metadata_loading: bool,
) -> ModelSelectorView {
    let total = rows.len();
    let q = query.trim().to_ascii_lowercase();
    let mut visible_ids = Vec::new();
    for row in rows {
        if !q.is_empty() && !row.search_text.to_ascii_lowercase().contains(&q) {
            continue;
        }
        let pass = match mode {
            ModelSelectorFilterMode::ShowAll => true,
            ModelSelectorFilterMode::Compatible => row.fitness.passes_compatible_filter(),
        };
        if pass {
            visible_ids.push(row.id.clone());
        }
    }
    let shown = visible_ids.len();
    let selected_id = selected_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let selected_visible = selected_id
        .as_ref()
        .is_some_and(|id| visible_ids.iter().any(|v| v == id));
    let checking_compatibility = metadata_loading
        && matches!(mode, ModelSelectorFilterMode::Compatible)
        && shown == 0
        && total > 0;
    ModelSelectorView {
        visible_ids,
        shown,
        total,
        selected_id,
        selected_visible,
        checking_compatibility,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, search: &str, fitness: ModelFitnessState) -> ModelSelectorRow {
        ModelSelectorRow {
            id: id.into(),
            search_text: search.into(),
            fitness,
        }
    }

    /// `#373`: counter N/M uses full snapshot M after Compatible filter.
    #[test]
    fn counter_shown_of_total_uses_snapshot_m() {
        let rows = vec![
            row("a", "alpha:latest", ModelFitnessState::Runnable),
            row("b", "beta:latest", ModelFitnessState::Incompatible),
            row("c", "gamma:latest", ModelFitnessState::PrepareAndRun),
            row("d", "delta:latest", ModelFitnessState::NeedsAction),
        ];
        let view =
            project_model_selector(&rows, "", ModelSelectorFilterMode::Compatible, None, false);
        assert_eq!(view.total, 4);
        assert_eq!(view.shown, 2); // runnable + prepare (+ checking would count)
        assert_eq!(
            model_selector_counter_label(view.shown, view.total),
            "2 / 4"
        );
        assert_eq!(view.visible_ids, vec!["a".to_string(), "c".to_string()]);

        let all = project_model_selector(&rows, "", ModelSelectorFilterMode::ShowAll, None, false);
        assert_eq!(all.total, 4);
        assert_eq!(all.shown, 4);
    }

    /// `#373`: selection by id survives Compatible filter that hides the row.
    #[test]
    fn selection_survives_filter_hide() {
        let rows = vec![
            row("keep", "keep:latest", ModelFitnessState::Runnable),
            row("hidden", "hidden:latest", ModelFitnessState::Incompatible),
        ];
        let view = project_model_selector(
            &rows,
            "",
            ModelSelectorFilterMode::Compatible,
            Some("hidden"),
            false,
        );
        assert_eq!(view.selected_id.as_deref(), Some("hidden"));
        assert!(!view.selected_visible);
        assert!(!view.visible_ids.iter().any(|id| id == "hidden"));
        // Clearing filter reveals it without changing selection.
        let shown = project_model_selector(
            &rows,
            "",
            ModelSelectorFilterMode::ShowAll,
            Some("hidden"),
            false,
        );
        assert_eq!(shown.selected_id.as_deref(), Some("hidden"));
        assert!(shown.selected_visible);
    }

    /// `#373`: search is case-insensitive and does not merge distinct names.
    #[test]
    fn search_case_insensitive_keeps_distinct_ids() {
        let rows = vec![
            row(
                "aira:model:ollama-org-model-latest-aaaa",
                "org/model:latest",
                ModelFitnessState::Runnable,
            ),
            row(
                "aira:model:ollama-org_model-latest-bbbb",
                "org_model:latest",
                ModelFitnessState::Runnable,
            ),
        ];
        let view = project_model_selector(
            &rows,
            "ORG/MODEL",
            ModelSelectorFilterMode::ShowAll,
            None,
            false,
        );
        assert_eq!(view.shown, 1);
        assert_eq!(
            view.visible_ids,
            vec!["aira:model:ollama-org-model-latest-aaaa".to_string()]
        );
        assert_eq!(view.total, 2);
    }

    /// `#373`: metadata loading + empty Compatible → checking, not empty catalog.
    #[test]
    fn metadata_loading_marks_checking_when_compatible_empty() {
        let rows = vec![row("cloud", "kimi:cloud", ModelFitnessState::NeedsAction)];
        let view =
            project_model_selector(&rows, "", ModelSelectorFilterMode::Compatible, None, true);
        assert!(view.checking_compatibility);
        assert_eq!(view.shown, 0);
        assert_eq!(view.total, 1);
    }
}
