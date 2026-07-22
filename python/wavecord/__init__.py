# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""WaveCord: a high-performance Lavalink v3/v4 client with a Rust core."""

from . import events, filters, metrics, sources
from ._wavecord import Node, WaveCordError, __version__, decode_message, ping
from .dispatcher import EventDispatcher
from .failover import HealthMonitor, failover
from .player import Player
from .pool import NodePool, PooledNode
from .queue import LoopMode, Queue

__all__ = [
    "Node",
    "WaveCordError",
    "Player",
    "EventDispatcher",
    "NodePool",
    "PooledNode",
    "HealthMonitor",
    "failover",
    "Queue",
    "LoopMode",
    "events",
    "filters",
    "sources",
    "metrics",
    "decode_message",
    "__version__",
    "ping",
]
