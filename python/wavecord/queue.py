# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""A per-guild track queue with loop modes and event-driven auto-advance."""

from __future__ import annotations

import enum
import random
from collections import deque
from collections.abc import Iterable, Iterator
from typing import Any

from ._wavecord import Node
from .dispatcher import EventDispatcher

Track = dict[str, Any]

_ADVANCE_REASONS = frozenset({"finished", "loadFailed"})


class LoopMode(enum.Enum):
    """How the queue repeats after a track ends."""

    OFF = "off"
    TRACK = "track"
    QUEUE = "queue"


class Queue:
    """An ordered collection of tracks with a current item and loop mode."""

    def __init__(self, loop: LoopMode = LoopMode.OFF) -> None:
        self._items: deque[Track] = deque()
        self.loop = loop
        self.current: Track | None = None

    def __len__(self) -> int:
        return len(self._items)

    def __iter__(self) -> Iterator[Track]:
        return iter(self._items)

    def __bool__(self) -> bool:
        return bool(self._items)

    def add(self, track: Track) -> None:
        """Append a track to the end of the queue."""
        self._items.append(track)

    def extend(self, tracks: Iterable[Track]) -> None:
        """Append several tracks to the end of the queue."""
        self._items.extend(tracks)

    def clear(self) -> None:
        """Remove all tracks and the current item."""
        self._items.clear()
        self.current = None

    def shuffle(self) -> None:
        """Shuffle the queued tracks in place."""
        random.shuffle(self._items)

    def next(self) -> Track | None:
        """Advance to and return the next track per the loop mode, or ``None``
        when the queue is exhausted."""
        if self.loop is LoopMode.TRACK and self.current is not None:
            return self.current
        if not self._items:
            self.current = None
            return None
        self.current = self._items.popleft()
        if self.loop is LoopMode.QUEUE:
            self._items.append(self.current)
        return self.current


def bind_autoplay(
    dispatcher: EventDispatcher,
    node: Node,
    guild_id: int | str,
    queue: Queue,
):
    """Play the next queued track when the current one ends naturally.

    Returns the registered handler so it can be removed later. Manual stops
    (``stopped``/``replaced``/``cleanup``) do not advance the queue.
    """
    gid = str(guild_id)

    async def _advance(event) -> None:
        if event.guild_id != gid or event.reason not in _ADVANCE_REASONS:
            return
        track = queue.next()
        if track is not None:
            await node.play(gid, track["encoded"])

    dispatcher.add_listener("track_end", _advance)
    return _advance
