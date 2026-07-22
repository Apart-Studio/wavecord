# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Queue ordering / loop modes, and event-driven auto-advance."""

import asyncio
import json

from wavecord.dispatcher import EventDispatcher
from wavecord.queue import LoopMode, Queue, bind_autoplay


def t(name):
    return {"encoded": name, "info": {"title": name}}


def test_fifo_order():
    q = Queue()
    q.extend([t("a"), t("b"), t("c")])
    assert q.next()["encoded"] == "a"
    assert q.next()["encoded"] == "b"
    assert q.next()["encoded"] == "c"
    assert q.next() is None
    assert q.current is None


def test_loop_track_repeats_current():
    q = Queue(loop=LoopMode.TRACK)
    q.add(t("a"))
    q.add(t("b"))
    assert q.next()["encoded"] == "a"
    assert q.next()["encoded"] == "a"
    q.loop = LoopMode.OFF
    assert q.next()["encoded"] == "b"


def test_loop_queue_cycles():
    q = Queue(loop=LoopMode.QUEUE)
    q.extend([t("a"), t("b")])
    got = [q.next()["encoded"] for _ in range(5)]
    assert got == ["a", "b", "a", "b", "a"]


def test_len_bool_iter():
    q = Queue()
    assert not q and len(q) == 0
    q.extend([t("a"), t("b")])
    assert q and len(q) == 2
    assert [x["encoded"] for x in q] == ["a", "b"]


class ScriptedNode:
    def __init__(self, events):
        self._events = [json.dumps(e) for e in events]
        self.played: list[tuple[str, str]] = []

    async def next_event(self):
        await asyncio.sleep(0)
        return self._events.pop(0) if self._events else None

    async def play(self, guild_id, encoded, **kw):
        self.played.append((guild_id, encoded))


def _end(guild="1", reason="finished"):
    return {"op": "event", "type": "TrackEndEvent", "guildId": guild,
            "track": {"encoded": "x", "info": None}, "reason": reason}


async def test_autoplay_advances_on_finish_only():
    node = ScriptedNode([
        _end(reason="finished"),
        _end(reason="stopped"),
        _end(reason="finished"),
    ])
    disp = EventDispatcher(node)
    q = Queue()
    q.extend([t("a"), t("b")])
    bind_autoplay(disp, node, "1", q)

    await disp.start()

    assert node.played == [("1", "a"), ("1", "b")]


async def test_autoplay_ignores_other_guilds():
    node = ScriptedNode([_end(guild="999", reason="finished")])
    disp = EventDispatcher(node)
    q = Queue()
    q.add(t("a"))
    bind_autoplay(disp, node, "1", q)

    await disp.start()
    assert node.played == []
