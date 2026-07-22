// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! Track models.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A playable Lavalink track (identical shape on v3 and v4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Opaque base64 track string - what you send back to play it.
    pub encoded: String,
    pub info: TrackInfo,
    #[serde(default, rename = "pluginInfo")]
    pub plugin_info: Value,
    #[serde(default, rename = "userData")]
    pub user_data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    pub identifier: String,
    pub is_seekable: bool,
    pub author: String,
    /// Track length in milliseconds.
    pub length: u64,
    pub is_stream: bool,
    /// Current position in milliseconds.
    pub position: u64,
    pub title: String,
    pub uri: Option<String>,
    pub artwork_url: Option<String>,
    pub isrc: Option<String>,
    pub source_name: String,
}
