use ratatui::style::Color;

use crate::update;

use super::super::{CYAN, GREEN, RED, YELLOW};

pub(super) fn version_label(version: Option<&str>) -> String {
    version
        .and_then(known_version)
        .map_or_else(|| "unknown".into(), |version| format!("v{version}"))
}

pub(super) fn known_version(version: &str) -> Option<&str> {
    (!version.is_empty() && version != "legacy").then_some(version)
}

pub(super) fn peer_target_version<'a>(
    installed: Option<&str>,
    peer_desired: Option<&'a str>,
    local_desired: Option<&'a str>,
) -> Option<&'a str> {
    installed.and_then(known_version)?;
    match (
        peer_desired.and_then(known_version),
        local_desired.and_then(known_version),
    ) {
        (Some(peer), Some(local)) if update::newer_version(peer, local) => Some(local),
        (Some(peer), _) => Some(peer),
        (None, local) => local,
    }
}

pub(super) fn peer_update_state(
    connected: bool,
    installed: Option<&str>,
    peer_desired: Option<&str>,
    local_desired: Option<&str>,
) -> (&'static str, Color) {
    if !connected {
        return ("offline", RED);
    }
    let Some(installed) = installed.and_then(known_version) else {
        return ("version unknown · setup required", YELLOW);
    };
    let peer_desired = peer_desired.and_then(known_version);
    let local_desired = local_desired.and_then(known_version);
    if peer_desired.is_some_and(|desired| update::newer_version(installed, desired)) {
        return ("updating", YELLOW);
    }
    if local_desired.is_some_and(|desired| update::newer_version(installed, desired)) {
        return ("outdated · press u", YELLOW);
    }
    if local_desired.is_some_and(|desired| update::newer_version(desired, installed)) {
        return ("ahead", CYAN);
    }
    ("current", GREEN)
}
