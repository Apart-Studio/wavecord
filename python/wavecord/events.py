# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Typed event objects decoded from the node's normalized JSON with msgspec."""

from __future__ import annotations

from typing import Optional, Union

import msgspec


class TrackInfo(msgspec.Struct, rename="camel"):
    """Metadata for a track."""

    identifier: str = ""
    title: str = ""
    author: str = ""
    length: int = 0
    is_seekable: bool = False
    is_stream: bool = False
    position: int = 0
    uri: Optional[str] = None
    artwork_url: Optional[str] = None
    isrc: Optional[str] = None
    source_name: str = ""


class Track(msgspec.Struct, rename="camel"):
    """A track: its encoded string, metadata, and plugin/user data."""

    encoded: str
    info: Optional[TrackInfo] = None
    plugin_info: Optional[dict] = None
    user_data: Optional[dict] = None


class Exception_(msgspec.Struct, rename="camel"):
    """A Lavalink exception attached to a track event."""

    message: Optional[str] = None
    severity: str = ""
    cause: Optional[str] = None


class PlayerState(msgspec.Struct, rename="camel"):
    """Periodic player state: position, connection, and ping."""

    time: int = 0
    position: int = 0
    connected: bool = False
    ping: int = -1


class Ready(msgspec.Struct, tag_field="op", tag="ready", rename="camel"):
    """The ``ready`` message carrying the session id."""

    session_id: str
    resumed: bool = False


class PlayerUpdate(msgspec.Struct, tag_field="op", tag="playerUpdate", rename="camel"):
    """A ``playerUpdate`` message with the current player state."""

    guild_id: str
    state: PlayerState


class Stats(msgspec.Struct, tag_field="op", tag="stats", rename="camel"):
    """A ``stats`` message with node health metrics."""

    players: int = 0
    playing_players: int = 0
    uptime: int = 0
    memory: Optional[dict] = None
    cpu: Optional[dict] = None
    frame_stats: Optional[dict] = None


class Event(msgspec.Struct, tag_field="op", tag="event", rename="camel"):
    """A player lifecycle or plugin event. ``type`` names the specific event;
    only the fields relevant to that type are populated."""

    type: str
    guild_id: str
    track: Optional[Track] = None
    reason: Optional[str] = None
    exception: Optional[Exception_] = None
    threshold_ms: Optional[int] = None
    code: Optional[int] = None
    by_remote: Optional[bool] = None
    segments: Optional[list] = None
    segment: Optional[dict] = None
    chapters: Optional[list] = None
    chapter: Optional[dict] = None
    lyrics: Optional[dict] = None
    line_index: Optional[int] = None
    line: Optional[dict] = None


Message = Union[Ready, PlayerUpdate, Stats, Event]

_decoder = msgspec.json.Decoder(Message)


class _Peek(msgspec.Struct):
    """Minimal struct to name a message without a full decode."""

    op: str
    type: Optional[str] = None


_peek_decoder = msgspec.json.Decoder(_Peek)

_EVENT_NAMES = {
    "TrackStartEvent": "track_start",
    "TrackEndEvent": "track_end",
    "TrackExceptionEvent": "track_exception",
    "TrackStuckEvent": "track_stuck",
    "WebSocketClosedEvent": "websocket_closed",
    # SponsorBlock plugin
    "SegmentsLoaded": "segments_loaded",
    "SegmentSkipped": "segment_skipped",
    "ChaptersLoaded": "chapters_loaded",
    "ChapterStarted": "chapter_started",
    # LavaLyrics plugin
    "LyricsFoundEvent": "lyrics_found",
    "LyricsNotFoundEvent": "lyrics_not_found",
    "LyricsLineEvent": "lyrics_line",
}

# Decoded message class -> event name (used by decode).
_OP_NAMES = {Ready: "ready", PlayerUpdate: "player_update", Stats: "stats"}
# Wire op string -> event name (used by the cheap name peek).
_OP_STR_NAMES = {"ready": "ready", "playerUpdate": "player_update", "stats": "stats"}


def decode(raw: str | bytes) -> tuple[str, object] | None:
    """Decode a raw ``next_event`` string into ``(name, event)``, or ``None`` to skip."""
    msg = _decoder.decode(raw)
    if type(msg) is Event:
        name = _EVENT_NAMES.get(msg.type)
        return (name, msg) if name is not None else None
    return _OP_NAMES[type(msg)], msg


def event_name(raw: str | bytes) -> str | None:
    """Return the name of a raw message without a full decode, or ``None``."""
    p = _peek_decoder.decode(raw)
    if p.op == "event":
        return _EVENT_NAMES.get(p.type)
    return _OP_STR_NAMES.get(p.op)


__all__ = [
    "TrackInfo", "Track", "Exception_", "PlayerState",
    "Ready", "PlayerUpdate", "Stats", "Event", "Message", "decode", "event_name",
]
