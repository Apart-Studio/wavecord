// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! Player and voice-state models.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::track::Track;

/// A player as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub guild_id: String,
    pub track: Option<Track>,
    pub volume: i32,
    pub paused: bool,
    pub state: PlayerState,
    pub voice: VoiceState,
    #[serde(default)]
    pub filters: Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerState {
    /// Unix timestamp (ms) the state was generated.
    pub time: i64,
    /// Position of the track in ms.
    pub position: i64,
    /// Present on v4 and v3.4+; defaults to `false` on older nodes.
    #[serde(default)]
    pub connected: bool,
    /// Voice-gateway ping in ms, or -1 if not connected. Not sent by older v3
    /// nodes, so it defaults to -1.
    #[serde(default = "ping_default")]
    pub ping: i64,
}

fn ping_default() -> i64 {
    -1
}

/// Voice connection info handed to Lavalink (from Discord's VOICE_SERVER_UPDATE
/// + VOICE_STATE_UPDATE). Also returned inside `Player`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceState {
    pub token: String,
    pub endpoint: String,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
}

/// Request body for `PATCH /v4/sessions/{sid}/players/{guild}`.
/// Every field is optional; only what you set is sent.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<UpdatePlayerTrack>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<Option<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceState>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayerTrack {
    /// `Some(Some(s))` plays `s`; `Some(None)` sends `null` to stop; `None` omits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoded: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data: Option<Value>,
}

impl UpdatePlayer {
    /// Play a track by its encoded string.
    pub fn play(encoded: impl Into<String>) -> Self {
        Self {
            track: Some(UpdatePlayerTrack {
                encoded: Some(Some(encoded.into())),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Stop playback (sends `{"track":{"encoded":null}}`).
    pub fn stop() -> Self {
        Self {
            track: Some(UpdatePlayerTrack {
                encoded: Some(None),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
