# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Metrics snapshot and Prometheus rendering."""

from wavecord import metrics
from wavecord.pool import NodePool


class FakeNode:
    def is_connected(self):
        return True


def _pool():
    pool = NodePool()
    pooled = pool.register("a", FakeNode())
    pooled.stats = {
        "players": 5,
        "playingPlayers": 3,
        "uptime": 1000,
        "cpu": {"lavalinkLoad": 0.4, "systemLoad": 0.2},
        "memory": {},
    }
    return pool


def test_snapshot_reports_load():
    row = metrics.snapshot(_pool())[0]
    assert row["node"] == "a"
    assert row["players"] == 5
    assert row["playing_players"] == 3
    assert row["connected"] is True
    assert row["cpu_lavalink_load"] == 0.4
    assert row["penalty"] > 0


def test_prometheus_text():
    text = metrics.prometheus(_pool())
    assert '# TYPE wavecord_node_penalty gauge' in text
    assert 'wavecord_node_playing_players{node="a"} 3' in text
    assert 'wavecord_node_up{node="a"} 1' in text
