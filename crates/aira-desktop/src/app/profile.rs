use aira_desktop_runtime::{NetworkProfile, DEFAULT_PEER_LISTEN, DEFAULT_RELAY_TTL_DAYS};

use crate::actions;

use super::AiraDesktopApp;

impl AiraDesktopApp {
    pub(super) fn apply_profile(&mut self, profile: NetworkProfile) {
        let relay_ttl = if profile.is_relay_profile() {
            Some(
                self.relay_ttl_edit
                    .trim()
                    .parse()
                    .unwrap_or(DEFAULT_RELAY_TTL_DAYS),
            )
        } else {
            None
        };
        match actions::apply_network_profile(
            &mut self.settings,
            profile,
            &self.peer_listen_edit,
            relay_ttl,
        ) {
            Ok(()) => {
                if profile.requires_peer_listen() {
                    self.peer_listen_edit = self
                        .settings
                        .peer_listen
                        .clone()
                        .unwrap_or_else(|| DEFAULT_PEER_LISTEN.to_string());
                }
                if profile.is_relay_profile() {
                    self.relay_ttl_edit = self
                        .settings
                        .relay_ttl_days
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| DEFAULT_RELAY_TTL_DAYS.to_string());
                }
                self.restart_hint = true;
                if let Err(e) = self.persist_settings() {
                    self.last_error = Some(format!("{e:#}"));
                }
            }
            Err(e) => self.last_error = Some(format!("{e:#}")),
        }
    }

    pub(super) fn save_peer_listen(&mut self) {
        let profile = self.settings.network_profile;
        if !profile.requires_peer_listen() {
            return;
        }
        let relay_ttl = if profile.is_relay_profile() {
            Some(
                self.relay_ttl_edit
                    .trim()
                    .parse()
                    .unwrap_or(DEFAULT_RELAY_TTL_DAYS),
            )
        } else {
            None
        };
        match actions::apply_network_profile(
            &mut self.settings,
            profile,
            &self.peer_listen_edit,
            relay_ttl,
        ) {
            Ok(()) => {
                self.restart_hint = true;
                if let Err(e) = self.persist_settings() {
                    self.last_error = Some(format!("{e:#}"));
                }
            }
            Err(e) => self.last_error = Some(format!("{e:#}")),
        }
    }

    pub(super) fn save_relay_ttl(&mut self) {
        if !self.settings.network_profile.is_relay_profile() {
            return;
        }
        let days = self
            .relay_ttl_edit
            .trim()
            .parse()
            .unwrap_or(DEFAULT_RELAY_TTL_DAYS);
        match actions::apply_network_profile(
            &mut self.settings,
            NetworkProfile::P3,
            &self.peer_listen_edit,
            Some(days),
        ) {
            Ok(()) => {
                self.restart_hint = true;
                if let Err(e) = self.persist_settings() {
                    self.last_error = Some(format!("{e:#}"));
                }
            }
            Err(e) => self.last_error = Some(format!("{e:#}")),
        }
    }

    pub(super) fn toggle_relay_profile(&mut self, enable: bool) {
        if enable {
            self.apply_profile(NetworkProfile::P3);
        } else if self.settings.network_profile == NetworkProfile::P3 {
            self.apply_profile(NetworkProfile::P2);
        }
    }

    pub(super) fn toggle_gossip_profile(&mut self, enable: bool) {
        if enable {
            self.apply_profile(NetworkProfile::P4);
        } else if self.settings.network_profile == NetworkProfile::P4 {
            self.apply_profile(NetworkProfile::P2);
        }
    }
}
