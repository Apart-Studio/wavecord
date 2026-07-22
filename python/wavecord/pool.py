# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""A pool of Lavalink nodes with load-based balancing and failover."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from ._wavecord import Node
from .dispatcher import EventDispatcher


def penalty(stats: dict[str, Any] | None) -> float:
    """Lavalink-style load penalty for a node from its latest ``stats`` payload.

    Lower is less loaded. A node with no stats yet is treated as idle (0.0).
    """
    if not stats:
        return 0.0
    players = stats.get("playingPlayers", 0)
    cpu = stats.get("cpu", {}).get("lavalinkLoad", 0.0)
    cpu_penalty = 1.05 ** (100 * cpu) * 10 - 10
    return players + cpu_penalty


@dataclass
class PooledNode:
    """A node in the pool with its latest stats and per-guild positions."""

    label: str
    node: Node
    dispatcher: EventDispatcher | None = None
    stats: dict[str, Any] | None = None
    available: bool = True
    positions: dict[str, int] = field(default_factory=dict)

    @property
    def penalty(self) -> float:
        """This node's current load penalty."""
        return penalty(self.stats)


class NodePool:
    """Manages several nodes and routes guilds to the least-loaded one."""

    def __init__(self) -> None:
        self._nodes: dict[str, PooledNode] = {}
        self._assignments: dict[str, str] = {}

    def __len__(self) -> int:
        return len(self._nodes)

    @property
    def nodes(self) -> list[PooledNode]:
        """All pooled nodes."""
        return list(self._nodes.values())

    def register(
        self, label: str, node: Node, dispatcher: EventDispatcher | None = None
    ) -> PooledNode:
        """Add an already-created node. If a dispatcher is given, the pool tracks
        the node's live stats and player positions through it."""
        pooled = PooledNode(label, node, dispatcher)
        self._nodes[label] = pooled
        if dispatcher is not None:
            dispatcher.add_listener("stats", self._make_stats_listener(pooled))
            dispatcher.add_listener("player_update", self._make_position_listener(pooled))
        return pooled

    async def add_node(self, label: str, host: str, port: int, password: str,
                       user_id: str, **kwargs: Any) -> PooledNode:
        """Create, connect, and register a node with its own event dispatcher."""
        node = Node(host, port, password, user_id, **kwargs)
        await node.connect()
        dispatcher = EventDispatcher(node)
        pooled = self.register(label, node, dispatcher)
        dispatcher.start()
        return pooled

    def _make_stats_listener(self, pooled: PooledNode):
        def _on_stats(event) -> None:
            pooled.stats = {
                "players": event.players,
                "playingPlayers": event.playing_players,
                "uptime": event.uptime,
                "memory": event.memory or {},
                "cpu": event.cpu or {},
            }
            pooled.available = True

        return _on_stats

    def _make_position_listener(self, pooled: PooledNode):
        def _on_update(event) -> None:
            pooled.positions[event.guild_id] = event.state.position

        return _on_update

    def best(self) -> PooledNode | None:
        """The least-loaded available node, or ``None`` if none are available."""
        candidates = [p for p in self._nodes.values() if p.available]
        if not candidates:
            return None
        return min(candidates, key=lambda p: p.penalty)

    def get_node(self, guild_id: int | str) -> Node:
        """Return the node assigned to this guild, assigning the best one if the
        guild is new or its current node has become unavailable."""
        return self.assign(guild_id).node

    def assign(self, guild_id: int | str) -> PooledNode:
        """Return the guild's assigned node, choosing the best available if needed."""
        gid = str(guild_id)
        current = self._nodes.get(self._assignments.get(gid, ""))
        if current is not None and current.available:
            return current
        chosen = self.best()
        if chosen is None:
            raise RuntimeError("no available Lavalink nodes in the pool")
        self._assignments[gid] = chosen.label
        return chosen

    def get_pooled(self, label: str) -> PooledNode | None:
        """Return the pooled node with the given label, if any."""
        return self._nodes.get(label)

    def assigned_label(self, guild_id: int | str) -> str | None:
        """Return the label of the node assigned to a guild, if any."""
        return self._assignments.get(str(guild_id))

    def guilds_on(self, label: str) -> list[str]:
        """Return the guild ids currently assigned to a node."""
        return [g for g, node_label in self._assignments.items() if node_label == label]
