# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Failover moves a downed node's guilds to a healthy node and replays each at
its last-known position; the health monitor triggers it after a grace period."""

from wavecord.failover import HealthMonitor, failover
from wavecord.pool import NodePool


class FakeNode:
    def __init__(self, connected=True):
        self._connected = connected

    def is_connected(self):
        return self._connected


async def test_failover_moves_guilds_and_replays_with_position():
    pool = NodePool()
    a = pool.register("a", FakeNode())
    b = pool.register("b", FakeNode())
    a.stats = {"playingPlayers": 1, "cpu": {"lavalinkLoad": 0.0}}
    b.stats = {"playingPlayers": 1, "cpu": {"lavalinkLoad": 0.0}}

    pool.assign("g1")
    pool.assign("g2")
    assert pool.assigned_label("g1") == "a"
    a.positions = {"g1": 42000, "g2": 5000}

    replayed = []

    async def replay(gid, old, new_node, position_ms):
        replayed.append((gid, new_node, position_ms))

    moved = await failover(pool, "a", replay)

    assert sorted(moved) == ["g1", "g2"]
    assert pool.assigned_label("g1") == "b"
    assert pool.assigned_label("g2") == "b"
    positions = {gid: pos for gid, _n, pos in replayed}
    assert positions == {"g1": 42000, "g2": 5000}
    assert all(n is b.node for _g, n, _p in replayed)


async def test_health_monitor_triggers_after_grace():
    pool = NodePool()
    a = pool.register("a", FakeNode(connected=False))
    b = pool.register("b", FakeNode())
    a.stats = {"playingPlayers": 0, "cpu": {"lavalinkLoad": 0.0}}
    b.stats = {"playingPlayers": 0, "cpu": {"lavalinkLoad": 0.0}}
    pool.assign("g")
    assert pool.assigned_label("g") == "a"

    calls = []

    async def replay(gid, old, new_node, position_ms):
        calls.append(gid)

    mon = HealthMonitor(pool, replay, grace=10.0)

    assert await mon._tick(0.0) == []
    assert calls == []
    assert await mon._tick(11.0) == ["a"]
    assert calls == ["g"]
    assert pool.assigned_label("g") == "b"


def test_failover_exports():
    import wavecord
    assert hasattr(wavecord, "HealthMonitor") and hasattr(wavecord, "failover")
