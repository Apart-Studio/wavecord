# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Observability helpers: turn a :class:`~wavecord.NodePool` into a metrics
snapshot or Prometheus exposition text."""

from __future__ import annotations

from typing import Any

from .pool import NodePool


def _label(value: str) -> str:
    """Escape a Prometheus label value (backslash, quote, newline)."""
    return value.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")


def snapshot(pool: NodePool) -> list[dict[str, Any]]:
    """One dict per node with its live load metrics."""
    rows = []
    for pooled in pool.nodes:
        stats = pooled.stats or {}
        cpu = stats.get("cpu") or {}
        rows.append({
            "node": pooled.label,
            "available": pooled.available,
            "connected": bool(pooled.node.is_connected()),
            "penalty": round(pooled.penalty, 3),
            "players": stats.get("players", 0),
            "playing_players": stats.get("playingPlayers", 0),
            "uptime_ms": stats.get("uptime", 0),
            "cpu_lavalink_load": cpu.get("lavalinkLoad", 0.0),
            "cpu_system_load": cpu.get("systemLoad", 0.0),
        })
    return rows


def prometheus(pool: NodePool) -> str:
    """Render the pool's metrics as Prometheus exposition text.

    Serve the returned string from an HTTP endpoint with content type
    ``text/plain; version=0.0.4``.
    """
    metrics = {
        "wavecord_node_up": ("gauge", "1 if the node is connected and available"),
        "wavecord_node_penalty": ("gauge", "Load-balancing penalty (lower is less loaded)"),
        "wavecord_node_players": ("gauge", "Total players on the node"),
        "wavecord_node_playing_players": ("gauge", "Actively playing players on the node"),
        "wavecord_node_cpu_lavalink_load": ("gauge", "Lavalink CPU load fraction"),
    }
    lines: list[str] = []
    for name, (kind, help_text) in metrics.items():
        lines.append(f"# HELP {name} {help_text}")
        lines.append(f"# TYPE {name} {kind}")
        for row in snapshot(pool):
            label = f'{{node="{_label(row["node"])}"}}'
            value = {
                "wavecord_node_up": int(row["available"] and row["connected"]),
                "wavecord_node_penalty": row["penalty"],
                "wavecord_node_players": row["players"],
                "wavecord_node_playing_players": row["playing_players"],
                "wavecord_node_cpu_lavalink_load": row["cpu_lavalink_load"],
            }[name]
            lines.append(f"{name}{label} {value}")
    return "\n".join(lines) + "\n"
