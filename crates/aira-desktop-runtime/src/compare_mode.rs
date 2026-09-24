//! Work Compare enter/exit session (`#374`).
//!
//! Entering Compare takes A from the current solo selection and leaves B empty.
//! Exiting restores the saved solo pick unchanged. Distinct full CLI names stay
//! distinct rows (alias collision is already handled by `host_ollama_model_ref`).

/// Solo Work selector state before / after Compare (`#374`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoloWorkPick {
    /// Settings tip / default — no explicit Required pick.
    Auto,
    /// Explicit single-model Required pick.
    Specific { model_ref: String },
}

/// Compare legs after enter (`#374`). `b` is always empty on entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareLegs {
    pub a: String,
    pub b: String,
}

/// Enter Compare: A from current solo selection; B empty; return saved solo (`#374`).
///
/// - [`SoloWorkPick::Specific`] → A = that `model_ref` (when non-empty).
/// - [`SoloWorkPick::Auto`] (or empty Specific) → A = tip when present, else empty.
/// - B is always cleared so the second combo shows the empty prompt.
pub fn enter_compare(
    solo: &SoloWorkPick,
    tip_model_ref: Option<&str>,
) -> (CompareLegs, SoloWorkPick) {
    let a = match solo {
        SoloWorkPick::Specific { model_ref } => {
            let trimmed = model_ref.trim();
            if !trimmed.is_empty() {
                trimmed.to_string()
            } else {
                tip_model_ref
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("")
                    .to_string()
            }
        }
        SoloWorkPick::Auto => tip_model_ref
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("")
            .to_string(),
    };
    (
        CompareLegs {
            a,
            b: String::new(),
        },
        solo.clone(),
    )
}

/// Exit Compare: restore the solo pick saved at enter (`#374`).
pub fn exit_compare(saved: SoloWorkPick) -> SoloWorkPick {
    saved
}

/// True when each listed CLI name maps to its own catalog id (`#374`).
///
/// `org/model:latest` and `org_model:latest` must not collapse into one row.
pub fn distinct_host_cli_names_stay_distinct(cli_names: &[&str]) -> bool {
    let mut refs = std::collections::HashSet::new();
    for name in cli_names {
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        let model_ref = aira_flow::host_ollama_model_ref(name);
        if !refs.insert(model_ref) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `#374`: B starts empty; A comes from Specific solo pick.
    #[test]
    fn enter_compare_takes_a_from_solo_and_clears_b() {
        let solo = SoloWorkPick::Specific {
            model_ref: "aira:model:ollama-a-aaaaaaaaaaaa".into(),
        };
        let (legs, saved) = enter_compare(&solo, Some("aira:model:ollama-tip-bbbbbbbbbbbb"));
        assert_eq!(legs.a, "aira:model:ollama-a-aaaaaaaaaaaa");
        assert!(legs.b.is_empty());
        assert_eq!(saved, solo);
    }

    /// `#374`: Auto enter uses tip for A; B still empty.
    #[test]
    fn enter_compare_auto_uses_tip_for_a() {
        let solo = SoloWorkPick::Auto;
        let (legs, saved) = enter_compare(&solo, Some("aira:model:ollama-tip-bbbbbbbbbbbb"));
        assert_eq!(legs.a, "aira:model:ollama-tip-bbbbbbbbbbbb");
        assert!(legs.b.is_empty());
        assert_eq!(saved, SoloWorkPick::Auto);
    }

    /// `#374`: exit restores Specific; Compare legs do not leak into solo.
    #[test]
    fn exit_compare_restores_previous_solo_pick() {
        let solo = SoloWorkPick::Specific {
            model_ref: "aira:model:ollama-x-cccccccccccc".into(),
        };
        let (_legs, saved) = enter_compare(&solo, None);
        // Operator may change A/B while comparing; exit ignores those legs.
        let restored = exit_compare(saved);
        assert_eq!(
            restored,
            SoloWorkPick::Specific {
                model_ref: "aira:model:ollama-x-cccccccccccc".into(),
            }
        );
        let auto_restored = exit_compare(SoloWorkPick::Auto);
        assert_eq!(auto_restored, SoloWorkPick::Auto);
    }

    /// `#374`: re-enter always clears B even if a prior Compare left B set.
    #[test]
    fn reenter_clears_stale_b() {
        let solo = SoloWorkPick::Specific {
            model_ref: "aira:model:ollama-a-aaaaaaaaaaaa".into(),
        };
        let (first, _) = enter_compare(&solo, None);
        assert!(first.b.is_empty());
        // Simulating UI that had set B previously is irrelevant: enter always returns empty B.
        let (second, _) = enter_compare(&solo, None);
        assert!(second.b.is_empty());
        assert_eq!(second.a, first.a);
    }

    /// `#374`: slash vs underscore full names stay two executor identities.
    #[test]
    fn distinct_full_cli_names_do_not_merge() {
        assert!(distinct_host_cli_names_stay_distinct(&[
            "org/model:latest",
            "org_model:latest",
        ]));
        assert_ne!(
            aira_flow::host_ollama_model_ref("org/model:latest"),
            aira_flow::host_ollama_model_ref("org_model:latest")
        );
    }
}
