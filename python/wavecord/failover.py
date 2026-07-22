# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Move a downed node's guilds to healthy nodes and replay them."""

from __future__ import annotations

import asyncio
import logging
import time
from collections.abc import Awaitable
from typing import Callable

from .pool import NodePool, PooledNode

log = logging.getLogger("wavecord")

ReplayFn = Callable[[str, PooledNode, object, int], Awaitable[None]]


async def failover(pool: NodePool, label: str, replay: ReplayFn) -> list[str]:
    """Reassign every guild on node ``label`` to a healthy node and replay it.

    ``replay`` receives ``(guild_id, old_pooled, new_node, position_ms)`` and is
    where the app re-establishes voice and restarts playback. Returns the guild
    ids that were moved.
    """
    downed = pool.get_pooled(label)
    if downed is not None:
        downed.available = False

    moved: list[str] = []
    for gid in pool.guilds_on(label):
        position = downed.positions.get(gid, 0) if downed else 0
        new_node = pool.assign(gid).node
        try:
            await replay(gid, downed, new_node, position)
            moved.append(gid)
        except Exception:  # noqa: BLE001
            log.exception("failover replay failed for guild %s", gid)
    return moved


class HealthMonitor:
    """Polls node connection state and fails a node over after ``grace`` seconds
    of being disconnected."""

    def __init__(
        self,
        pool: NodePool,
        replay: ReplayFn,
        *,
        grace: float = 10.0,
        interval: float = 1.0,
    ) -> None:
        self.pool = pool
        self.replay = replay
        self.grace = grace
        self.interval = interval
        self._down_since: dict[str, float] = {}
        self._task: asyncio.Task | None = None

    async def _tick(self, now: float) -> list[str]:
        """Run one health pass; return labels that were failed over."""
        failed: list[str] = []
        for pooled in self.pool.nodes:
            if pooled.node.is_connected():
                self._down_since.pop(pooled.label, None)
                if not pooled.available and pooled.stats is not None:
                    pooled.available = True
                continue
            since = self._down_since.setdefault(pooled.label, now)
            if now - since >= self.grace and pooled.available:
                await failover(self.pool, pooled.label, self.replay)
                failed.append(pooled.label)
        return failed

    async def _run(self) -> None:
        while True:
            await self._tick(time.monotonic())
            await asyncio.sleep(self.interval)

    def start(self) -> asyncio.Task:
        """Start the background health monitor (idempotent). Returns the task."""
        if self._task is None or self._task.done():
            self._task = asyncio.ensure_future(self._run())
        return self._task

    async def stop(self) -> None:
        """Stop the background health monitor."""
        if self._task is not None:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass
            self._task = None
