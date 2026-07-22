# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""events.decode: the typed-Struct decode used by the dispatcher, plus the
WaveCordError exception type."""

import json

import wavecord
from wavecord import events


def _decode(obj):
    return events.decode(json.dumps(obj))


def test_ready_stats_player_update():
    name, ev = _decode({"op": "ready", "resumed": False, "sessionId": "abc"})
    assert name == "ready" and ev.session_id == "abc" and ev.resumed is False

    name, ev = _decode({"op": "stats", "players": 3, "playingPlayers": 2, "uptime": 10})
    assert name == "stats" and ev.players == 3 and ev.playing_players == 2

    name, ev = _decode({"op": "playerUpdate", "guildId": "7",
                        "state": {"time": 1, "position": 2, "connected": True, "ping": 5}})
    assert name == "player_update" and ev.guild_id == "7" and ev.state.position == 2


def test_events_typed_attributes():
    name, ev = _decode({
        "op": "event", "type": "TrackEndEvent", "guildId": "1",
        "track": {"encoded": "ENC", "info": {"title": "Song", "author": "X", "length": 100}},
        "reason": "finished",
    })
    assert name == "track_end"
    assert ev.guild_id == "1" and ev.reason == "finished"
    assert ev.track.encoded == "ENC" and ev.track.info.title == "Song"


def test_websocket_closed_and_stuck_fields():
    _, ev = _decode({"op": "event", "type": "WebSocketClosedEvent", "guildId": "1",
                     "code": 4006, "reason": "bye", "byRemote": True})
    assert ev.code == 4006 and ev.by_remote is True

    _, ev = _decode({"op": "event", "type": "TrackStuckEvent", "guildId": "1",
                     "track": {"encoded": "E"}, "thresholdMs": 5000})
    assert ev.threshold_ms == 5000


def test_unknown_event_type_is_skipped():
    assert events.decode(json.dumps(
        {"op": "event", "type": "SomeFuturePluginEvent", "guildId": "1"}
    )) is None


def test_track_exposes_plugin_and_user_data():
    _, ev = _decode({
        "op": "event", "type": "TrackStartEvent", "guildId": "1",
        "track": {"encoded": "E", "info": {"title": "T"},
                  "pluginInfo": {"albumName": "Album"}, "userData": {"requester": "42"}},
    })
    assert ev.track.plugin_info == {"albumName": "Album"}
    assert ev.track.user_data == {"requester": "42"}


def test_plugin_events_are_named_and_typed():
    name, ev = _decode({"op": "event", "type": "SegmentsLoaded", "guildId": "1",
                        "segments": [{"category": "sponsor"}]})
    assert name == "segments_loaded" and ev.segments[0]["category"] == "sponsor"

    name, ev = _decode({"op": "event", "type": "LyricsLineEvent", "guildId": "1",
                        "lineIndex": 2, "line": {"line": "hello"}})
    assert name == "lyrics_line" and ev.line_index == 2 and ev.line["line"] == "hello"


def test_event_name_peek_covers_every_op():
    def name(obj):
        return events.event_name(json.dumps(obj))

    assert name({"op": "ready", "resumed": False, "sessionId": "s"}) == "ready"
    assert name({"op": "stats", "players": 0}) == "stats"
    assert name({"op": "playerUpdate", "guildId": "1", "state": {}}) == "player_update"
    assert name({"op": "event", "type": "TrackEndEvent", "guildId": "1"}) == "track_end"
    assert name({"op": "event", "type": "UnknownFuture"}) is None


def test_wavecord_error_is_exception():
    assert issubclass(wavecord.WaveCordError, Exception)
