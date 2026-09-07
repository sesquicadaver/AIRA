//! Stable Desktop action / error / help IDs (`#258`).
//!
//! Contract: `code → localized message → help_id` shared with future F1 topics.
//! Keys are stable English identifiers; only human text is localized via [`UiLang`].

use aira_desktop_runtime::UiLang;

/// Catalog help topic IDs (Phase O §7). Articles land in `#263`/`#264`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HelpId {
    Start,
    WorkSubmit,
    WorkResult,
    WorkWaiting,
    ModelSelect,
    ModelUnavailable,
    NetworkConnect,
    NetworkReachability,
    NetworkTrust,
    SettingsApply,
    NodeLifecycle,
}

impl HelpId {
    /// Stable wire / F1 key (not localized).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::WorkSubmit => "work.submit",
            Self::WorkResult => "work.result",
            Self::WorkWaiting => "work.waiting",
            Self::ModelSelect => "model.select",
            Self::ModelUnavailable => "model.unavailable",
            Self::NetworkConnect => "network.connect",
            Self::NetworkReachability => "network.reachability",
            Self::NetworkTrust => "network.trust",
            Self::SettingsApply => "settings.apply",
            Self::NodeLifecycle => "node.lifecycle",
        }
    }

    /// Full Phase O seed catalog.
    pub fn catalog() -> &'static [HelpId] {
        &HELP_CATALOG
    }

    /// Parse a stable key; unknown keys fail closed.
    pub fn parse(s: &str) -> Option<Self> {
        HELP_CATALOG.iter().copied().find(|h| h.as_str() == s)
    }
}

const HELP_CATALOG: [HelpId; 11] = [
    HelpId::Start,
    HelpId::WorkSubmit,
    HelpId::WorkResult,
    HelpId::WorkWaiting,
    HelpId::ModelSelect,
    HelpId::ModelUnavailable,
    HelpId::NetworkConnect,
    HelpId::NetworkReachability,
    HelpId::NetworkTrust,
    HelpId::SettingsApply,
    HelpId::NodeLifecycle,
];

/// User-level Desktop actions with stable IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionId {
    WorkSubmit,
    NodeStart,
    NodeStop,
    StatusRefresh,
    SettingsPersist,
    Quit,
}

impl ActionId {
    /// Stable action id (not localized).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WorkSubmit => "work.submit",
            Self::NodeStart => "node.start",
            Self::NodeStop => "node.stop",
            Self::StatusRefresh => "status.refresh",
            Self::SettingsPersist => "settings.persist",
            Self::Quit => "app.quit",
        }
    }

    /// Default F1 topic for this action.
    pub fn help_id(self) -> HelpId {
        match self {
            Self::WorkSubmit => HelpId::WorkSubmit,
            Self::NodeStart | Self::NodeStop | Self::Quit => HelpId::NodeLifecycle,
            Self::StatusRefresh => HelpId::NetworkReachability,
            Self::SettingsPersist => HelpId::SettingsApply,
        }
    }

    /// All known Desktop actions (F1 / gates).
    pub fn catalog() -> &'static [ActionId] {
        &[
            Self::WorkSubmit,
            Self::NodeStart,
            Self::NodeStop,
            Self::StatusRefresh,
            Self::SettingsPersist,
            Self::Quit,
        ]
    }
}

/// Availability of an action for the current UI state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionGate {
    pub action: ActionId,
    pub available: bool,
    /// Stable reason when unavailable (for message / F1).
    pub reason: Option<ErrorCode>,
}

impl ActionGate {
    pub fn open(action: ActionId) -> Self {
        Self {
            action,
            available: true,
            reason: None,
        }
    }

    pub fn blocked(action: ActionId, reason: ErrorCode) -> Self {
        Self {
            action,
            available: false,
            reason: Some(reason),
        }
    }
}

/// Gate for Work submit: at most one in-flight job (`#257`).
pub fn work_submit_gate(inflight: bool) -> ActionGate {
    if inflight {
        ActionGate::blocked(ActionId::WorkSubmit, ErrorCode::WorkSubmitInFlight)
    } else {
        ActionGate::open(ActionId::WorkSubmit)
    }
}

/// Stable problem codes shown across UI surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    WorkEmptyText,
    WorkSubmitInFlight,
    WorkSubmitFailed,
    StatusRefreshFailed,
    NodeStartFailed,
    NodeStopFailed,
    SettingsPersistFailed,
    MeshSnapshotFailed,
    AutostartSyncFailed,
    Generic,
}

impl ErrorCode {
    /// Stable error code (not localized).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WorkEmptyText => "work.empty_text",
            Self::WorkSubmitInFlight => "work.submit_in_flight",
            Self::WorkSubmitFailed => "work.submit_failed",
            Self::StatusRefreshFailed => "status.refresh_failed",
            Self::NodeStartFailed => "node.start_failed",
            Self::NodeStopFailed => "node.stop_failed",
            Self::SettingsPersistFailed => "settings.persist_failed",
            Self::MeshSnapshotFailed => "mesh.snapshot_failed",
            Self::AutostartSyncFailed => "desktop.autostart_sync_failed",
            Self::Generic => "desktop.generic",
        }
    }

    pub fn help_id(self) -> HelpId {
        match self {
            Self::WorkEmptyText | Self::WorkSubmitInFlight | Self::WorkSubmitFailed => {
                HelpId::WorkSubmit
            }
            Self::StatusRefreshFailed | Self::MeshSnapshotFailed => HelpId::NetworkReachability,
            Self::NodeStartFailed | Self::NodeStopFailed | Self::AutostartSyncFailed => {
                HelpId::NodeLifecycle
            }
            Self::SettingsPersistFailed => HelpId::SettingsApply,
            Self::Generic => HelpId::Start,
        }
    }

    pub fn corrective_action(self) -> Option<ActionId> {
        match self {
            Self::WorkEmptyText => Some(ActionId::WorkSubmit),
            Self::WorkSubmitFailed => Some(ActionId::WorkSubmit),
            Self::StatusRefreshFailed | Self::MeshSnapshotFailed => Some(ActionId::StatusRefresh),
            Self::NodeStartFailed => Some(ActionId::NodeStart),
            Self::NodeStopFailed => Some(ActionId::NodeStop),
            Self::SettingsPersistFailed => Some(ActionId::SettingsPersist),
            Self::WorkSubmitInFlight | Self::AutostartSyncFailed | Self::Generic => None,
        }
    }

    /// Localized short message (end-user language).
    pub fn message(self, lang: UiLang) -> &'static str {
        match (lang, self) {
            (UiLang::En, Self::WorkEmptyText) => {
                "Enter a task before submitting. Empty text cannot be sent."
            }
            (UiLang::Uk, Self::WorkEmptyText) => {
                "Введіть завдання перед надсиланням. Порожній текст не приймається."
            }
            (UiLang::En, Self::WorkSubmitInFlight) => {
                "A task is already being submitted. Wait for it to finish."
            }
            (UiLang::Uk, Self::WorkSubmitInFlight) => {
                "Завдання вже надсилається. Дочекайтеся завершення."
            }
            (UiLang::En, Self::WorkSubmitFailed) => {
                "Could not submit the task. Check that AIRA is running, then try again."
            }
            (UiLang::Uk, Self::WorkSubmitFailed) => {
                "Не вдалося надіслати завдання. Перевірте, що AIRA запущена, і спробуйте знову."
            }
            (UiLang::En, Self::StatusRefreshFailed) => {
                "Could not refresh system status. Try Refresh again."
            }
            (UiLang::Uk, Self::StatusRefreshFailed) => {
                "Не вдалося оновити стан системи. Натисніть «Оновити» ще раз."
            }
            (UiLang::En, Self::NodeStartFailed) => {
                "Could not start AIRA. See technical details, then try Start again."
            }
            (UiLang::Uk, Self::NodeStartFailed) => {
                "Не вдалося запустити AIRA. Перегляньте технічні подробиці й спробуйте «Старт»."
            }
            (UiLang::En, Self::NodeStopFailed) => {
                "Could not stop AIRA. See technical details, then try Stop again."
            }
            (UiLang::Uk, Self::NodeStopFailed) => {
                "Не вдалося зупинити AIRA. Перегляньте технічні подробиці й спробуйте «Стоп»."
            }
            (UiLang::En, Self::SettingsPersistFailed) => {
                "Could not save settings. Check the marked values and apply again."
            }
            (UiLang::Uk, Self::SettingsPersistFailed) => {
                "Не вдалося зберегти параметри. Перевірте позначені поля й застосуйте знову."
            }
            (UiLang::En, Self::MeshSnapshotFailed) => {
                "Could not read network status. External reachability may be unknown."
            }
            (UiLang::Uk, Self::MeshSnapshotFailed) => {
                "Не вдалося прочитати стан мережі. Зовнішня доступність може бути невідомою."
            }
            (UiLang::En, Self::AutostartSyncFailed) => {
                "Could not update login autostart. Settings may be out of sync with the OS."
            }
            (UiLang::Uk, Self::AutostartSyncFailed) => {
                "Не вдалося оновити автозапуск. Параметри можуть розійтися з ОС."
            }
            (UiLang::En, Self::Generic) => {
                "Something went wrong. Open Help for the related topic, or try again."
            }
            (UiLang::Uk, Self::Generic) => {
                "Сталася помилка. Відкрийте Довідку за пов’язаною темою або спробуйте знову."
            }
        }
    }
}

/// Localized problem surface: stable code + message + help binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiProblem {
    pub code: ErrorCode,
    pub message: String,
    pub help_id: HelpId,
    pub detail: Option<String>,
    pub corrective: Option<ActionId>,
}

impl UiProblem {
    pub fn new(code: ErrorCode, lang: UiLang, detail: Option<String>) -> Self {
        Self {
            code,
            message: code.message(lang).to_string(),
            help_id: code.help_id(),
            detail,
            corrective: code.corrective_action(),
        }
    }

    /// Map a submit worker error string to a stable code.
    pub fn from_submit_err(err: &str, lang: UiLang) -> Self {
        let lower = err.to_ascii_lowercase();
        let code = if lower.contains("non-empty") {
            ErrorCode::WorkEmptyText
        } else {
            ErrorCode::WorkSubmitFailed
        };
        Self::new(code, lang, Some(err.to_string()))
    }

    pub fn from_code_err(code: ErrorCode, lang: UiLang, err: impl ToString) -> Self {
        Self::new(code, lang, Some(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn help_catalog_matches_phase_o_seed_ids() {
        let expected = [
            "start",
            "work.submit",
            "work.result",
            "work.waiting",
            "model.select",
            "model.unavailable",
            "network.connect",
            "network.reachability",
            "network.trust",
            "settings.apply",
            "node.lifecycle",
        ];
        let got: Vec<_> = HelpId::catalog().iter().map(|h| h.as_str()).collect();
        assert_eq!(got, expected);
        for id in expected {
            assert_eq!(HelpId::parse(id).unwrap().as_str(), id);
        }
        assert!(HelpId::parse("nope").is_none());
    }

    #[test]
    fn action_and_error_ids_are_unique() {
        let actions = [
            ActionId::WorkSubmit,
            ActionId::NodeStart,
            ActionId::NodeStop,
            ActionId::StatusRefresh,
            ActionId::SettingsPersist,
            ActionId::Quit,
        ];
        let mut set = HashSet::new();
        for a in actions {
            assert!(set.insert(a.as_str()), "duplicate {}", a.as_str());
            assert!(!a.help_id().as_str().is_empty());
        }
        let codes = [
            ErrorCode::WorkEmptyText,
            ErrorCode::WorkSubmitInFlight,
            ErrorCode::WorkSubmitFailed,
            ErrorCode::StatusRefreshFailed,
            ErrorCode::NodeStartFailed,
            ErrorCode::NodeStopFailed,
            ErrorCode::SettingsPersistFailed,
            ErrorCode::MeshSnapshotFailed,
            ErrorCode::AutostartSyncFailed,
            ErrorCode::Generic,
        ];
        set.clear();
        for c in codes {
            assert!(set.insert(c.as_str()), "duplicate {}", c.as_str());
            assert!(!c.message(UiLang::En).is_empty());
            assert!(!c.message(UiLang::Uk).is_empty());
            assert_ne!(c.message(UiLang::En), c.message(UiLang::Uk));
        }
    }

    #[test]
    fn empty_submit_classifies_to_work_empty_text() {
        let p = UiProblem::from_submit_err("problem text must be non-empty", UiLang::En);
        assert_eq!(p.code, ErrorCode::WorkEmptyText);
        assert_eq!(p.help_id, HelpId::WorkSubmit);
        assert_eq!(p.code.as_str(), "work.empty_text");
    }

    #[test]
    fn work_submit_gate_blocks_when_inflight() {
        assert!(work_submit_gate(false).available);
        let g = work_submit_gate(true);
        assert!(!g.available);
        assert_eq!(g.reason, Some(ErrorCode::WorkSubmitInFlight));
        assert_eq!(g.action.as_str(), "work.submit");
    }

    #[test]
    fn problem_binds_code_message_help() {
        let p = UiProblem::new(ErrorCode::NodeStartFailed, UiLang::Uk, Some("boom".into()));
        assert_eq!(p.code.as_str(), "node.start_failed");
        assert_eq!(p.help_id.as_str(), "node.lifecycle");
        assert!(p.message.contains("запустити") || p.message.contains("AIRA"));
        assert_eq!(p.detail.as_deref(), Some("boom"));
        assert_eq!(p.corrective, Some(ActionId::NodeStart));
    }
}
