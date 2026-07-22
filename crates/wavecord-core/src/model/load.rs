// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! Track-loading result models.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::track::Track;

/// Result of `GET /v4/loadtracks?identifier=...`.
///
/// The wire format is `{"loadType": "...", "data": ...}`; we model it as an
/// adjacently-tagged enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "loadType", content = "data", rename_all = "camelCase")]
pub enum LoadResult {
    Track(Track),
    Playlist(Playlist),
    Search(Vec<Track>),
    Empty,
    Error(Exception),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub info: PlaylistInfo,
    #[serde(default)]
    pub plugin_info: Value,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistInfo {
    pub name: String,
    /// Index of the selected track, or -1 if none.
    pub selected_track: i32,
}

/// A Lavalink exception (used by `loadType: error` and track exception events).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exception {
    pub message: Option<String>,
    pub severity: Severity,
    pub cause: Option<String>,
    #[serde(default)]
    pub cause_stack_trace: Option<String>,
}

/// v4 sends lowercase (`common`); v3 sends uppercase (`COMMON`) - both accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    #[serde(alias = "COMMON")]
    Common,
    #[serde(alias = "SUSPICIOUS")]
    Suspicious,
    #[serde(alias = "FAULT")]
    Fault,
}
