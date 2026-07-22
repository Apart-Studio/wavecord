// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! Lavalink v3 wire specifics: client-to-server WebSocket ops and normalization
//! of v3 server messages and loadtracks responses into the canonical model.

use serde_json::{json, Value};

use crate::error::{Error, Result};
use crate::model::{
    Event, EventTrack, LoadResult, Playlist, PlaylistInfo, Ready, ServerMessage, Stats, Track,
    TrackEndReason, TrackInfo,
};

/// `voiceUpdate` - hands Lavalink the Discord voice connection.
pub fn voice_update_op(guild_id: &str, token: &str, endpoint: &str, session_id: &str) -> Value {
    json!({
        "op": "voiceUpdate",
        "guildId": guild_id,
        "sessionId": session_id,
        "event": { "token": token, "endpoint": endpoint, "guildId": guild_id },
    })
}

/// `play` - start an encoded track.
pub fn play_op(
    guild_id: &str,
    encoded: &str,
    start_ms: Option<i64>,
    end_ms: Option<i64>,
    volume: Option<i32>,
    paused: Option<bool>,
    no_replace: bool,
) -> Value {
    let mut op = json!({
        "op": "play",
        "guildId": guild_id,
        "track": encoded,
        "noReplace": no_replace,
    });
    let map = op.as_object_mut().expect("json object");
    if let Some(v) = start_ms {
        map.insert("startTime".into(), json!(v));
    }
    if let Some(v) = end_ms {
        map.insert("endTime".into(), json!(v));
    }
    if let Some(v) = volume {
        map.insert("volume".into(), json!(v));
    }
    if let Some(v) = paused {
        map.insert("pause".into(), json!(v));
    }
    op
}

pub fn stop_op(guild_id: &str) -> Value {
    json!({ "op": "stop", "guildId": guild_id })
}

pub fn pause_op(guild_id: &str, paused: bool) -> Value {
    json!({ "op": "pause", "guildId": guild_id, "pause": paused })
}

pub fn seek_op(guild_id: &str, position_ms: i64) -> Value {
    json!({ "op": "seek", "guildId": guild_id, "position": position_ms })
}

pub fn volume_op(guild_id: &str, volume: i32) -> Value {
    json!({ "op": "volume", "guildId": guild_id, "volume": volume })
}

pub fn destroy_op(guild_id: &str) -> Value {
    json!({ "op": "destroy", "guildId": guild_id })
}

/// `configureResuming` - keep this node's players alive for `timeout` seconds
/// after a disconnect; reconnect with the same `key` in the `Resume-Key` header.
pub fn configure_resuming_op(key: &str, timeout: u64) -> Value {
    json!({ "op": "configureResuming", "key": key, "timeout": timeout })
}

/// `filters` - in v3 the filter fields sit at the top level of the op (unlike
/// v4, where they are nested under a `filters` key in the REST body).
pub fn filters_op(guild_id: &str, filters: &Value) -> Value {
    let mut op = json!({ "op": "filters", "guildId": guild_id });
    if let (Some(map), Some(obj)) = (op.as_object_mut(), filters.as_object()) {
        for (key, value) in obj {
            map.insert(key.clone(), value.clone());
        }
    }
    op
}

/// Convert a raw v3 WebSocket message into the canonical [`ServerMessage`].
/// Returns `Ok(None)` for messages we intentionally ignore.
pub fn normalize_message(raw: &str) -> Result<Option<ServerMessage>> {
    let value: Value = serde_json::from_str(raw)?;
    let op = value.get("op").and_then(Value::as_str).unwrap_or_default();

    match op {
        // ready and stats share v4's shape.
        "ready" => Ok(Some(ServerMessage::Ready(serde_json::from_value::<Ready>(
            value,
        )?))),
        "stats" => Ok(Some(ServerMessage::Stats(serde_json::from_value::<Stats>(
            value,
        )?))),
        "playerUpdate" => {
            let guild_id = str_field(&value, "guildId")?;
            let state = serde_json::from_value(value.get("state").cloned().unwrap_or(Value::Null))?;
            Ok(Some(ServerMessage::PlayerUpdate(
                crate::model::PlayerUpdate { guild_id, state },
            )))
        }
        "event" => normalize_event(&value).map(|e| e.map(ServerMessage::Event)),
        _ => Ok(None),
    }
}

fn normalize_event(value: &Value) -> Result<Option<Event>> {
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let guild_id = str_field(value, "guildId")?;
    // v3 sends the track as a base64 string under `encodedTrack` (3.4+) or the
    // legacy `track` field.
    let encoded = value
        .get("encodedTrack")
        .or_else(|| value.get("track"))
        .and_then(Value::as_str)
        .map(str::to_owned);

    let event = match kind {
        "TrackStartEvent" => Event::TrackStartEvent {
            guild_id,
            track: EventTrack::from_encoded(require(encoded, "encodedTrack")?),
        },
        "TrackEndEvent" => Event::TrackEndEvent {
            guild_id,
            track: EventTrack::from_encoded(require(encoded, "encodedTrack")?),
            reason: serde_json::from_value::<TrackEndReason>(
                value.get("reason").cloned().unwrap_or(Value::Null),
            )?,
        },
        "TrackExceptionEvent" => Event::TrackExceptionEvent {
            guild_id,
            track: EventTrack::from_encoded(require(encoded, "encodedTrack")?),
            exception: serde_json::from_value(
                value.get("exception").cloned().unwrap_or(Value::Null),
            )?,
        },
        "TrackStuckEvent" => Event::TrackStuckEvent {
            guild_id,
            track: EventTrack::from_encoded(require(encoded, "encodedTrack")?),
            threshold_ms: value
                .get("thresholdMs")
                .and_then(Value::as_u64)
                .unwrap_or_default(),
        },
        "WebSocketClosedEvent" => Event::WebSocketClosedEvent {
            guild_id,
            code: value
                .get("code")
                .and_then(Value::as_i64)
                .unwrap_or_default() as i32,
            reason: value
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            by_remote: value
                .get("byRemote")
                .and_then(Value::as_bool)
                .unwrap_or_default(),
        },
        _ => return Ok(None),
    };
    Ok(Some(event))
}

/// Convert a v3 `/loadtracks` response into the canonical [`LoadResult`].
pub fn normalize_load_result(value: &Value) -> Result<LoadResult> {
    let load_type = str_field(value, "loadType")?;
    let tracks = || -> Result<Vec<Track>> {
        value
            .get("tracks")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().map(v3_track).collect::<Result<Vec<_>>>())
            .unwrap_or_else(|| Ok(Vec::new()))
    };

    match load_type.as_str() {
        "TRACK_LOADED" => {
            let mut ts = tracks()?;
            let first = ts
                .drain(..)
                .next()
                .ok_or_else(|| Error::Other("TRACK_LOADED with no tracks".into()))?;
            Ok(LoadResult::Track(first))
        }
        "SEARCH_RESULT" => Ok(LoadResult::Search(tracks()?)),
        "PLAYLIST_LOADED" => {
            let info = value.get("playlistInfo").cloned().unwrap_or(Value::Null);
            Ok(LoadResult::Playlist(Playlist {
                info: PlaylistInfo {
                    name: info
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned(),
                    selected_track: info
                        .get("selectedTrack")
                        .and_then(Value::as_i64)
                        .unwrap_or(-1) as i32,
                },
                plugin_info: Value::Null,
                tracks: tracks()?,
            }))
        }
        "NO_MATCHES" => Ok(LoadResult::Empty),
        "LOAD_FAILED" => Ok(LoadResult::Error(serde_json::from_value(
            value.get("exception").cloned().unwrap_or(Value::Null),
        )?)),
        other => Err(Error::Other(format!("unknown v3 loadType {other:?}"))),
    }
}

/// Build a canonical [`Track`] from a v3 track object (`{encoded, track, info}`).
fn v3_track(value: &Value) -> Result<Track> {
    let encoded = value
        .get("encoded")
        .or_else(|| value.get("track"))
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Other("v3 track missing encoded/track".into()))?
        .to_owned();
    let info: TrackInfo =
        serde_json::from_value(value.get("info").cloned().unwrap_or(Value::Null))?;
    Ok(Track {
        encoded,
        info,
        plugin_info: Value::Null,
        user_data: Value::Null,
    })
}

fn str_field(value: &Value, key: &str) -> Result<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| Error::Other(format!("missing string field {key:?}")))
}

fn require<T>(opt: Option<T>, key: &str) -> Result<T> {
    opt.ok_or_else(|| Error::Other(format!("missing field {key:?}")))
}
