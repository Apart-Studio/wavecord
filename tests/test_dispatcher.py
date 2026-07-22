# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""The dispatcher parses raw node events into typed objects and fans them out to
sync and async handlers, including a wildcard."""

import asyncio
import json

import wavecord
from wavecord.dispatcher import EventDispatcher
from wavecord.events import Event


class ScriptedNode:
    """Fake node yielding a fixed sequence of raw JSON strings (as the real node
    does over next_event), then None."""

    def __init__(self, events):
        self._events = [json.dumps(e) for e in events]

    async def next_event(self):
        await asyncio.sleep(0)
        return self._events.pop(0) if self._events else None


def _track(enc="ENC"):
    return {"encoded": enc, "info": None}


async def test_dispatch_typed_events_to_sync_and_async_handlers():
    node = ScriptedNode([
        {"op": "ready", "resumed": False, "sessionId": "s"},
        {"op": "event", "type": "TrackStartEvent", "guildId": "1", "track": _track()},
        {"op": "event", "type": "TrackEndEvent", "guildId": "1", "track": _track(),
         "reason": "finished"},
    ])
    disp = EventDispatcher(node)

    starts: list[Event] = []
    ends: list[Event] = []
    everything: list = []

    @disp.on("track_start")
    def _on_start(e):
        starts.append(e)

    @disp.on("track_end")
    async def _on_end(e):
        ends.append(e)

    @disp.on("*")
    async def _on_any(e):
        everything.append(e)

    await disp.start()

    assert len(starts) == 1 and starts[0].guild_id == "1"
    assert len(ends) == 1 and ends[0].reason == "finished"
    assert ends[0].track.encoded == "ENC"
    assert len(everything) == 3


async def test_one_failing_handler_does_not_stop_the_pump():
    node = ScriptedNode([
        {"op": "event", "type": "TrackStartEvent", "guildId": "1", "track": _track()},
        {"op": "event", "type": "TrackStartEvent", "guildId": "2", "track": _track()},
    ])
    disp = EventDispatcher(node)
    seen = []

    @disp.on("track_start")
    def _boom(e):
        raise RuntimeError("handler blew up")

    @disp.on("track_start")
    def _ok(e):
        seen.append(e.guild_id)

    await disp.start()
    assert seen == ["1", "2"]


async def test_skips_full_decode_for_unwanted_events(monkeypatch):
    import wavecord.dispatcher as d

    node = ScriptedNode([
        {"op": "playerUpdate", "guildId": "1", "state": {"time": 1, "position": 2}},
        {"op": "event", "type": "TrackEndEvent", "guildId": "1",
         "track": {"encoded": "E"}, "reason": "finished"},
    ])
    disp = EventDispatcher(node)

    fully_decoded = []
    orig = d.decode_event
    monkeypatch.setattr(d, "decode_event", lambda raw: (fully_decoded.append(raw), orig(raw))[1])

    ends = []

    @disp.on("track_end")
    async def _te(e):
        ends.append(e)

    await disp.start()

    assert len(ends) == 1
    assert len(fully_decoded) == 1


async def test_wildcard_still_receives_everything():
    node = ScriptedNode([
        {"op": "playerUpdate", "guildId": "1", "state": {"time": 1, "position": 2}},
        {"op": "stats", "players": 0, "playingPlayers": 0, "uptime": 1},
    ])
    disp = EventDispatcher(node)
    seen = []

    @disp.on("*")
    async def _all(e):
        seen.append(type(e).__name__)

    await disp.start()
    assert seen == ["PlayerUpdate", "Stats"]


def test_dispatcher_exported():
    assert hasattr(wavecord, "EventDispatcher")
