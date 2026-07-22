# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Async event dispatcher over a :class:`wavecord.Node`."""

from __future__ import annotations

import asyncio
import inspect
import logging
from collections import defaultdict
from collections.abc import Awaitable
from typing import Any, Callable, Optional

from . import events
from ._wavecord import Node
from .events import decode as decode_event

log = logging.getLogger("wavecord")

Handler = Callable[[Any], Optional[Awaitable[None]]]


class EventDispatcher:
    """Fans a node's normalized event stream out to registered handlers.

    Register handlers with ``@dispatcher.on("track_end")`` and call
    :meth:`start` to pump events in the background. Handlers may be sync or
    async, and a ``"*"`` handler receives every event.
    """

    def __init__(self, node: Node) -> None:
        self.node = node
        self._listeners: dict[str, list[Handler]] = defaultdict(list)
        self._task: asyncio.Task | None = None

    def on(self, name: str) -> Callable[[Handler], Handler]:
        """Decorator to register a handler for an event name (or ``"*"``)."""

        def decorator(fn: Handler) -> Handler:
            self._listeners[name].append(fn)
            return fn

        return decorator

    def add_listener(self, name: str, fn: Handler) -> None:
        """Register a handler for an event name."""
        self._listeners[name].append(fn)

    def remove_listener(self, name: str, fn: Handler) -> None:
        """Remove a previously registered handler."""
        try:
            self._listeners[name].remove(fn)
        except ValueError:
            pass

    async def dispatch(self, name: str, event: Any) -> None:
        """Call every handler registered for ``name`` (and for ``"*"``)."""
        for fn in (*self._listeners.get(name, ()), *self._listeners.get("*", ())):
            try:
                result = fn(event)
                if inspect.isawaitable(result):
                    await result
            except Exception:  # noqa: BLE001
                log.exception("error in %r handler for %s", fn, name)

    async def _handle(self, raw, wildcard: bool) -> None:
        """Decode one raw message and dispatch it, skipping the full decode for
        events that have no listener (unless a ``"*"`` handler is present)."""
        if not wildcard:
            name = events.event_name(raw)
            if name is None or name not in self._listeners:
                return
        parsed = decode_event(raw)
        if parsed is not None:
            await self.dispatch(*parsed)

    async def _pump(self) -> None:
        node = self.node
        batched = hasattr(node, "next_events")
        while True:
            if batched:
                batch = await node.next_events(64)
                if batch is None:
                    break
            else:
                one = await node.next_event()
                if one is None:
                    break
                batch = [one]
            wildcard = "*" in self._listeners
            for raw in batch:
                await self._handle(raw, wildcard)

    def start(self) -> asyncio.Task:
        """Start the background event pump (idempotent). Returns the task."""
        if self._task is None or self._task.done():
            self._task = asyncio.ensure_future(self._pump())
        return self._task

    async def stop(self) -> None:
        """Stop the background event pump."""
        if self._task is not None:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass
            self._task = None
