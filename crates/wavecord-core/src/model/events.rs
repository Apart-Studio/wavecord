// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! Server message and event models.

use serde::{Deserialize, Serialize};

use super::load::Exception;
use super::player::PlayerState;
use super::stats::Stats;
use super::track::TrackInfo;

/// A message pushed by the node over the WebSocket, normalized across v3/v4.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum ServerMessage {
    /// First message after connecting; carries the session id.
    Ready(Ready),
    /// Periodic per-guild player state.
    PlayerUpdate(PlayerUpdate),
    /// Periodic node stats.
    Stats(Stats),
    /// A player lifecycle event.
    Event(Event),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ready {
    pub resumed: bool,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerUpdate {
    pub guild_id: String,
    pub state: PlayerState,
}

/// The track carried by an event. On v4 the node sends a full object (so `info`
/// is present); on v3 it sends only the base64 string (so `info` is `None`).
/// Normalizing to this shape gives Python one consistent representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTrack {
    pub encoded: String,
    #[serde(default)]
    pub info: Option<TrackInfo>,
}

impl EventTrack {
    pub fn from_encoded(encoded: impl Into<String>) -> Self {
        Self {
            encoded: encoded.into(),
            info: None,
        }
    }
}

/// The `event` op, discriminated by its `type` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    TrackStartEvent {
        #[serde(rename = "guildId")]
        guild_id: String,
        track: EventTrack,
    },
    TrackEndEvent {
        #[serde(rename = "guildId")]
        guild_id: String,
        track: EventTrack,
        reason: TrackEndReason,
    },
    TrackExceptionEvent {
        #[serde(rename = "guildId")]
        guild_id: String,
        track: EventTrack,
        exception: Exception,
    },
    TrackStuckEvent {
        #[serde(rename = "guildId")]
        guild_id: String,
        track: EventTrack,
        #[serde(rename = "thresholdMs")]
        threshold_ms: u64,
    },
    WebSocketClosedEvent {
        #[serde(rename = "guildId")]
        guild_id: String,
        code: i32,
        reason: String,
        #[serde(rename = "byRemote")]
        by_remote: bool,
    },
}

/// Why a track stopped. v4 sends camelCase (`finished`); v3 sends SCREAMING_CASE
/// (`FINISHED`) - accepted via aliases so both normalize to the same value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrackEndReason {
    #[serde(alias = "FINISHED")]
    Finished,
    #[serde(alias = "LOAD_FAILED")]
    LoadFailed,
    #[serde(alias = "STOPPED")]
    Stopped,
    #[serde(alias = "REPLACED")]
    Replaced,
    #[serde(alias = "CLEANUP")]
    Cleanup,
}
