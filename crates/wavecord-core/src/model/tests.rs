// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! Tests that lock the exact Lavalink JSON wire format.

use super::*;
use serde_json::json;

fn sample_track() -> serde_json::Value {
    json!({
        "encoded": "QAAA...",
        "info": {
            "identifier": "dQw4w9WgXcQ",
            "isSeekable": true,
            "author": "RickAstley",
            "length": 212000,
            "isStream": false,
            "position": 0,
            "title": "Never Gonna Give You Up",
            "uri": "https://youtu.be/dQw4w9WgXcQ",
            "artworkUrl": "https://img/x.jpg",
            "isrc": null,
            "sourceName": "youtube"
        },
        "pluginInfo": {},
        "userData": {}
    })
}

#[test]
fn parse_track() {
    let t: Track = serde_json::from_value(sample_track()).unwrap();
    assert_eq!(t.info.title, "Never Gonna Give You Up");
    assert_eq!(t.info.length, 212000);
    assert!(t.info.is_seekable);
    assert_eq!(t.info.isrc, None);
}

#[test]
fn parse_ready() {
    let msg: ServerMessage =
        serde_json::from_value(json!({"op":"ready","resumed":false,"sessionId":"abc"})).unwrap();
    match msg {
        ServerMessage::Ready(r) => {
            assert!(!r.resumed);
            assert_eq!(r.session_id, "abc");
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn parse_player_update() {
    let msg: ServerMessage = serde_json::from_value(json!({
        "op":"playerUpdate","guildId":"1",
        "state":{"time":1710000000000i64,"position":1500,"connected":true,"ping":42}
    }))
    .unwrap();
    match msg {
        ServerMessage::PlayerUpdate(p) => {
            assert_eq!(p.guild_id, "1");
            assert_eq!(p.state.position, 1500);
            assert!(p.state.connected);
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn parse_stats() {
    let msg: ServerMessage = serde_json::from_value(json!({
        "op":"stats","players":1,"playingPlayers":1,"uptime":10000,
        "memory":{"free":1,"used":2,"allocated":3,"reservable":4},
        "cpu":{"cores":8,"systemLoad":0.5,"lavalinkLoad":0.1},
        "frameStats":{"sent":3000,"nulled":10,"deficit":0}
    }))
    .unwrap();
    match msg {
        ServerMessage::Stats(s) => {
            assert_eq!(s.playing_players, 1);
            assert_eq!(s.cpu.cores, 8);
            assert!(s.frame_stats.is_some());
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn parse_track_start_and_end_events() {
    let start: ServerMessage = serde_json::from_value(json!({
        "op":"event","type":"TrackStartEvent","guildId":"1","track": sample_track()
    }))
    .unwrap();
    assert!(matches!(
        start,
        ServerMessage::Event(Event::TrackStartEvent { .. })
    ));

    let end: ServerMessage = serde_json::from_value(json!({
        "op":"event","type":"TrackEndEvent","guildId":"1","track": sample_track(),"reason":"finished"
    }))
    .unwrap();
    match end {
        ServerMessage::Event(Event::TrackEndEvent { reason, .. }) => {
            assert_eq!(reason, TrackEndReason::Finished);
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn parse_ws_closed_event() {
    let msg: ServerMessage = serde_json::from_value(json!({
        "op":"event","type":"WebSocketClosedEvent","guildId":"1",
        "code":4006,"reason":"","byRemote":true
    }))
    .unwrap();
    match msg {
        ServerMessage::Event(Event::WebSocketClosedEvent {
            code, by_remote, ..
        }) => {
            assert_eq!(code, 4006);
            assert!(by_remote);
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn parse_load_result_variants() {
    let track: LoadResult =
        serde_json::from_value(json!({"loadType":"track","data": sample_track()})).unwrap();
    assert!(matches!(track, LoadResult::Track(_)));

    let search: LoadResult =
        serde_json::from_value(json!({"loadType":"search","data":[sample_track()]})).unwrap();
    match search {
        LoadResult::Search(v) => assert_eq!(v.len(), 1),
        _ => panic!("wrong variant"),
    }

    let playlist: LoadResult = serde_json::from_value(json!({
        "loadType":"playlist",
        "data":{"info":{"name":"Mix","selectedTrack":-1},"pluginInfo":{},"tracks":[sample_track()]}
    }))
    .unwrap();
    match playlist {
        LoadResult::Playlist(p) => {
            assert_eq!(p.info.name, "Mix");
            assert_eq!(p.tracks.len(), 1);
        }
        _ => panic!("wrong variant"),
    }

    let empty: LoadResult =
        serde_json::from_value(json!({"loadType":"empty","data":null})).unwrap();
    assert!(matches!(empty, LoadResult::Empty));

    let error: LoadResult = serde_json::from_value(json!({
        "loadType":"error",
        "data":{"message":"boom","severity":"common","cause":"x","causeStackTrace":"..."}
    }))
    .unwrap();
    match error {
        LoadResult::Error(e) => assert_eq!(e.severity, Severity::Common),
        _ => panic!("wrong variant"),
    }
}

mod v3 {
    use crate::model::{Event, LoadResult, ServerMessage, TrackEndReason};
    use crate::protocol::v3::{normalize_load_result, normalize_message};
    use serde_json::json;

    #[test]
    fn v3_track_end_uppercase_reason_and_string_track() {
        // v3 sends the track as a base64 STRING and SCREAMING_CASE reasons.
        let raw = json!({
            "op": "event", "type": "TrackEndEvent", "guildId": "42",
            "encodedTrack": "ENC", "track": "ENC", "reason": "FINISHED"
        })
        .to_string();
        let msg = normalize_message(&raw).unwrap().unwrap();
        match msg {
            ServerMessage::Event(Event::TrackEndEvent { track, reason, .. }) => {
                assert_eq!(track.encoded, "ENC");
                assert!(track.info.is_none()); // v3 carries no full info
                assert_eq!(reason, TrackEndReason::Finished);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn v3_ready_and_stats_reuse_v4_shape() {
        let ready =
            normalize_message(&json!({"op":"ready","resumed":false,"sessionId":"s1"}).to_string())
                .unwrap()
                .unwrap();
        assert!(matches!(ready, ServerMessage::Ready(_)));

        let stats = normalize_message(
            &json!({"op":"stats","players":0,"playingPlayers":0,"uptime":1,
                    "memory":{"free":1,"used":2,"allocated":3,"reservable":4},
                    "cpu":{"cores":8,"systemLoad":0.0,"lavalinkLoad":0.0}})
            .to_string(),
        )
        .unwrap()
        .unwrap();
        assert!(matches!(stats, ServerMessage::Stats(_)));
    }

    #[test]
    fn v3_player_update_without_ping_defaults() {
        // Older v3 omits `ping`; it must default rather than fail.
        let msg = normalize_message(
            &json!({"op":"playerUpdate","guildId":"7",
                    "state":{"time":1,"position":2,"connected":true}})
            .to_string(),
        )
        .unwrap()
        .unwrap();
        match msg {
            ServerMessage::PlayerUpdate(p) => {
                assert_eq!(p.guild_id, "7");
                assert_eq!(p.state.ping, -1);
            }
            _ => panic!("wrong variant"),
        }
    }

    fn v3_track() -> serde_json::Value {
        json!({
            "encoded": "ENC", "track": "ENC",
            "info": {
                "identifier": "id", "isSeekable": true, "author": "a",
                "length": 1000, "isStream": false, "position": 0,
                "title": "t", "uri": "u", "sourceName": "http"
            }
        })
    }

    #[test]
    fn v3_loadtracks_shapes_normalize_to_canonical() {
        // TRACK_LOADED -> Track
        let track = normalize_load_result(&json!({
            "loadType": "TRACK_LOADED", "tracks": [v3_track()], "playlistInfo": {}
        }))
        .unwrap();
        match track {
            LoadResult::Track(t) => {
                assert_eq!(t.encoded, "ENC");
                assert_eq!(t.info.source_name, "http");
                assert!(t.info.artwork_url.is_none()); // absent on v3
            }
            _ => panic!("wrong variant"),
        }

        // SEARCH_RESULT -> Search
        let search = normalize_load_result(&json!({
            "loadType": "SEARCH_RESULT", "tracks": [v3_track(), v3_track()], "playlistInfo": {}
        }))
        .unwrap();
        assert!(matches!(search, LoadResult::Search(v) if v.len() == 2));

        // PLAYLIST_LOADED -> Playlist
        let playlist = normalize_load_result(&json!({
            "loadType": "PLAYLIST_LOADED",
            "tracks": [v3_track()],
            "playlistInfo": {"name": "Mix", "selectedTrack": 0}
        }))
        .unwrap();
        match playlist {
            LoadResult::Playlist(p) => {
                assert_eq!(p.info.name, "Mix");
                assert_eq!(p.info.selected_track, 0);
            }
            _ => panic!("wrong variant"),
        }

        // NO_MATCHES -> Empty
        assert!(matches!(
            normalize_load_result(&json!({"loadType":"NO_MATCHES","tracks":[],"playlistInfo":{}}))
                .unwrap(),
            LoadResult::Empty
        ));

        // LOAD_FAILED -> Error (with uppercase severity)
        let err = normalize_load_result(&json!({
            "loadType": "LOAD_FAILED", "tracks": [], "playlistInfo": {},
            "exception": {"message": "boom", "severity": "COMMON", "cause": "x"}
        }))
        .unwrap();
        assert!(matches!(err, LoadResult::Error(_)));
    }

    #[test]
    fn v3_play_op_shape() {
        use crate::protocol::v3::play_op;
        let op = play_op("42", "ENC", Some(1000), None, Some(50), None, true);
        assert_eq!(op["op"], "play");
        assert_eq!(op["guildId"], "42");
        assert_eq!(op["track"], "ENC");
        assert_eq!(op["startTime"], 1000);
        assert_eq!(op["volume"], 50);
        assert_eq!(op["noReplace"], true);
        assert!(op.get("endTime").is_none());
        assert!(op.get("pause").is_none());
    }

    #[test]
    fn v3_filters_op_flattens_fields() {
        use crate::protocol::v3::filters_op;
        let filters = json!({
            "volume": 1.5,
            "equalizer": [{"band": 0, "gain": 0.2}]
        });
        let op = filters_op("42", &filters);
        // v3 puts the filter fields at the top level next to op/guildId.
        assert_eq!(op["op"], "filters");
        assert_eq!(op["guildId"], "42");
        assert_eq!(op["volume"], 1.5);
        assert_eq!(op["equalizer"][0]["band"], 0);
    }
}

#[test]
fn serialize_update_player_play_and_stop() {
    let play = serde_json::to_value(UpdatePlayer::play("ENCODED")).unwrap();
    assert_eq!(play, json!({"track":{"encoded":"ENCODED"}}));

    let stop = serde_json::to_value(UpdatePlayer::stop()).unwrap();
    assert_eq!(stop, json!({"track":{"encoded":null}}));

    let vol = serde_json::to_value(UpdatePlayer {
        volume: Some(50),
        paused: Some(true),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(vol, json!({"volume":50,"paused":true}));
}
