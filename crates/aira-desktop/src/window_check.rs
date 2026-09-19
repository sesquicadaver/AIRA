//! P4 window contract. Not a screenshot and not a contrast pass.
//!
//! The default window is 560×720. A docked 320 px F1 panel on that width
//! leaves about 240 px for Work, which clips the status strip and forms.
//! F1 docks beside the screen only when the central column stays at least
//! the default Work width.

/// Default inner width (`main` viewport).
pub const DEFAULT_WINDOW_WIDTH: f32 = 560.0;
/// Docked F1 width when the window is wide enough.
pub const HELP_DOCK_WIDTH: f32 = 320.0;
/// Smallest window that still leaves [`DEFAULT_WINDOW_WIDTH`] beside F1.
pub const HELP_DOCK_MIN_WINDOW: f32 = DEFAULT_WINDOW_WIDTH + HELP_DOCK_WIDTH;

/// `true` when F1 should be a side panel. Narrower windows float Help.
pub fn help_docks_beside(window_width: f32) -> bool {
    window_width >= HELP_DOCK_MIN_WINDOW
}

/// Width left for Work / System / Settings while F1 is open.
pub fn central_width_with_f1(window_width: f32) -> f32 {
    if help_docks_beside(window_width) {
        (window_width - HELP_DOCK_WIDTH).max(0.0)
    } else {
        window_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_check_f1_keeps_central_column() {
        assert!(!help_docks_beside(560.0));
        assert_eq!(central_width_with_f1(560.0), 560.0);

        assert!(help_docks_beside(900.0));
        assert!(central_width_with_f1(900.0) >= DEFAULT_WINDOW_WIDTH);

        assert!(help_docks_beside(1600.0));
        assert!(central_width_with_f1(1600.0) >= DEFAULT_WINDOW_WIDTH);
        assert_eq!(central_width_with_f1(1600.0), 1600.0 - HELP_DOCK_WIDTH);
    }
}
