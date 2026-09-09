//! Actionable problem / strip next-step copy (`#290` / `phase-r-plan` R4).
//!
//! User-facing next action is primary. Stable wire ids (`help:`, `try:`,
//! error codes) stay secondary/tech — never the only copy.

use aira_desktop_runtime::UiLang;

use crate::lexicon::{ActionId, UiProblem};

/// Localized next-step phrase for a Desktop action.
pub fn action_next_step(action: ActionId, lang: UiLang) -> &'static str {
    match (lang, action) {
        (UiLang::En, ActionId::WorkSubmit) => "Next: enter a task and press Run",
        (UiLang::Uk, ActionId::WorkSubmit) => "Далі: введіть завдання і натисніть «Виконати»",
        (UiLang::En, ActionId::NodeStart) => "Next: press Start",
        (UiLang::Uk, ActionId::NodeStart) => "Далі: натисніть «Старт»",
        (UiLang::En, ActionId::NodeStop) => "Next: press Stop",
        (UiLang::Uk, ActionId::NodeStop) => "Далі: натисніть «Стоп»",
        (UiLang::En, ActionId::StatusRefresh) => "Next: press Refresh",
        (UiLang::Uk, ActionId::StatusRefresh) => "Далі: натисніть «Оновити»",
        (UiLang::En, ActionId::SettingsPersist) => "Next: fix the marked fields and save again",
        (UiLang::Uk, ActionId::SettingsPersist) => {
            "Далі: виправте позначені поля й збережіть знову"
        }
        (UiLang::En, ActionId::Quit) => "Next: quit AIRA if you need a clean exit",
        (UiLang::Uk, ActionId::Quit) => "Далі: вийдіть з AIRA для чистого завершення",
    }
}

/// Short strip Work-cell hint when a corrective action is known.
pub fn action_strip_hint(action: ActionId, lang: UiLang) -> &'static str {
    match (lang, action) {
        (UiLang::En, ActionId::WorkSubmit) => "needs Run",
        (UiLang::Uk, ActionId::WorkSubmit) => "потрібно «Виконати»",
        (UiLang::En, ActionId::NodeStart) => "needs Start",
        (UiLang::Uk, ActionId::NodeStart) => "потрібен Старт",
        (UiLang::En, ActionId::NodeStop) => "needs Stop",
        (UiLang::Uk, ActionId::NodeStop) => "потрібен Стоп",
        (UiLang::En, ActionId::StatusRefresh) => "needs Refresh",
        (UiLang::Uk, ActionId::StatusRefresh) => "потрібно Оновити",
        (UiLang::En, ActionId::SettingsPersist) => "needs Save",
        (UiLang::Uk, ActionId::SettingsPersist) => "потрібно Зберегти",
        (UiLang::En, ActionId::Quit) => "needs Quit",
        (UiLang::Uk, ActionId::Quit) => "потрібен Вихід",
    }
}

/// Projection for problem footer / events: human next step + optional wire ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemActionView {
    pub next_step: Option<&'static str>,
    pub code_wire: &'static str,
    pub help_wire: &'static str,
    pub try_wire: Option<&'static str>,
}

impl ProblemActionView {
    pub fn from_problem(problem: &UiProblem, lang: UiLang) -> Self {
        let (next_step, try_wire) = match problem.corrective {
            Some(action) => (Some(action_next_step(action, lang)), Some(action.as_str())),
            None => (None, None),
        };
        Self {
            next_step,
            code_wire: problem.code.as_str(),
            help_wire: problem.help_id.as_str(),
            try_wire,
        }
    }

    /// True when the surface exposes a human next step (not wire-only).
    pub fn has_human_next_step(&self) -> bool {
        self.next_step.is_some()
    }
}

/// Strip Work cell when a problem is present.
pub fn strip_work_from_problem(
    problem: &UiProblem,
    lang: UiLang,
    generic_needs_attention: &'static str,
) -> &'static str {
    match problem.corrective {
        Some(action) => action_strip_hint(action, lang),
        None => generic_needs_attention,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::ErrorCode;

    #[test]
    fn next_step_is_human_not_wire_id() {
        let step = action_next_step(ActionId::NodeStart, UiLang::En);
        assert!(step.contains("Start"));
        assert!(!step.contains("node.start"));
        assert!(!step.starts_with("try:"));
        let uk = action_next_step(ActionId::NodeStart, UiLang::Uk);
        assert!(uk.contains("Старт"));
        assert!(!uk.contains("node.start"));
    }

    #[test]
    fn problem_view_exposes_human_before_wires() {
        let p = UiProblem::new(ErrorCode::NodeStartFailed, UiLang::En, None);
        let v = ProblemActionView::from_problem(&p, UiLang::En);
        assert!(v.has_human_next_step());
        assert_eq!(
            v.next_step,
            Some(action_next_step(ActionId::NodeStart, UiLang::En))
        );
        assert_eq!(v.code_wire, "node.start_failed");
        assert_eq!(v.help_wire, "node.lifecycle");
        assert_eq!(v.try_wire, Some("node.start"));
        assert!(!v.next_step.unwrap().contains(v.try_wire.unwrap()));
    }

    #[test]
    fn problem_without_corrective_has_no_fake_next_step() {
        let p = UiProblem::new(ErrorCode::WorkSubmitInFlight, UiLang::Uk, None);
        let v = ProblemActionView::from_problem(&p, UiLang::Uk);
        assert!(!v.has_human_next_step());
        assert!(v.try_wire.is_none());
        assert_eq!(v.code_wire, "work.submit_in_flight");
    }

    #[test]
    fn strip_uses_action_hint_not_only_generic() {
        let p = UiProblem::new(ErrorCode::StatusRefreshFailed, UiLang::En, None);
        let hint = strip_work_from_problem(&p, UiLang::En, "needs attention");
        assert_eq!(hint, "needs Refresh");
        assert_ne!(hint, "needs attention");
        assert!(!hint.contains("status.refresh"));
    }

    #[test]
    fn all_actions_have_en_and_uk_next_steps() {
        for action in ActionId::catalog() {
            let en = action_next_step(*action, UiLang::En);
            let uk = action_next_step(*action, UiLang::Uk);
            assert!(!en.is_empty(), "{action:?}");
            assert!(!uk.is_empty(), "{action:?}");
            assert_ne!(en, uk, "{action:?}");
            assert!(!en.contains(action.as_str()), "{action:?} en leaks wire");
            assert!(!uk.contains(action.as_str()), "{action:?} uk leaks wire");
        }
    }
}
