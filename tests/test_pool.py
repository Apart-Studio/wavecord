# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Pool load-balancing: least-loaded node wins, assignments are sticky, and an
unavailable node's guilds get reassigned."""

import asyncio
import json

from wavecord.dispatcher import EventDispatcher
from wavecord.pool import NodePool, penalty


class FakeNode:
    def __init__(self, name):
        self.name = name


def _stats(playing, load=0.0):
    return {"playingPlayers": playing, "cpu": {"lavalinkLoad": load}}


def test_penalty_orders_by_players_then_cpu():
    assert penalty(None) == 0.0
    assert penalty(_stats(0)) < penalty(_stats(5))
    assert penalty(_stats(5, 0.9)) > penalty(_stats(5, 0.1))


def test_best_picks_least_loaded():
    pool = NodePool()
    a = pool.register("a", FakeNode("a"))
    b = pool.register("b", FakeNode("b"))
    a.stats = _stats(10)
    b.stats = _stats(2)
    assert pool.best().label == "b"


def test_assignment_is_sticky():
    pool = NodePool()
    a = pool.register("a", FakeNode("a"))
    b = pool.register("b", FakeNode("b"))
    a.stats = _stats(1)
    b.stats = _stats(9)

    first = pool.assign("guild1").label
    assert first == "a"
    a.stats = _stats(100)
    assert pool.assign("guild1").label == "a"


def test_unavailable_node_triggers_reassignment():
    pool = NodePool()
    a = pool.register("a", FakeNode("a"))
    b = pool.register("b", FakeNode("b"))
    a.stats = _stats(1)
    b.stats = _stats(2)

    assert pool.assign("g").label == "a"
    a.available = False
    assert pool.assign("g").label == "b"
    assert pool.guilds_on("b") == ["g"]


def test_no_available_nodes_raises():
    pool = NodePool()
    p = pool.register("a", FakeNode("a"))
    p.available = False
    try:
        pool.assign("g")
    except RuntimeError as e:
        assert "no available" in str(e)
    else:
        raise AssertionError("expected RuntimeError")


class ScriptedNode:
    def __init__(self, events):
        self._events = [json.dumps(e) for e in events]

    async def next_event(self):
        await asyncio.sleep(0)
        return self._events.pop(0) if self._events else None


async def test_stats_and_positions_tracked_through_dispatcher():
    node = ScriptedNode([
        {"op": "stats", "players": 2, "playingPlayers": 3, "uptime": 1,
         "cpu": {"cores": 8, "systemLoad": 0.1, "lavalinkLoad": 0.5}},
        {"op": "playerUpdate", "guildId": "77",
         "state": {"time": 1, "position": 4200, "connected": True, "ping": 5}},
    ])
    pool = NodePool()
    dispatcher = EventDispatcher(node)
    pooled = pool.register("a", node, dispatcher)

    await dispatcher.start()

    assert pooled.stats["playingPlayers"] == 3
    assert pooled.stats["players"] == 2
    assert pooled.stats["cpu"]["lavalinkLoad"] == 0.5
    assert pooled.penalty > 3
    assert pooled.positions["77"] == 4200
