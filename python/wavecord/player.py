# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Per-guild convenience wrapper around a node."""

from __future__ import annotations

from typing import Any

from ._wavecord import Node


class Player:
    """A per-guild player that delegates every call to the Rust node."""

    def __init__(self, node: Node, guild_id: int | str) -> None:
        self.node = node
        self.guild_id = str(guild_id)

    async def play(self, encoded: str, **kwargs: Any) -> dict:
        """Play an encoded track string (from :meth:`search`)."""
        return await self.node.play(self.guild_id, encoded, **kwargs)

    async def pause(self, paused: bool = True) -> dict:
        """Pause playback."""
        return await self.node.set_pause(self.guild_id, paused)

    async def resume(self) -> dict:
        """Resume playback."""
        return await self.node.set_pause(self.guild_id, False)

    async def stop(self) -> dict:
        """Stop playback, keeping the voice connection."""
        return await self.node.stop(self.guild_id)

    async def seek(self, position_ms: int) -> dict:
        """Seek to a position in milliseconds."""
        return await self.node.seek(self.guild_id, position_ms)

    async def set_volume(self, volume: int) -> dict:
        """Set the volume (0-1000)."""
        return await self.node.set_volume(self.guild_id, volume)

    async def set_filters(self, filters: dict[str, Any]) -> dict | None:
        """Apply an audio filter set (build one with :mod:`wavecord.filters`)."""
        return await self.node.set_filters(self.guild_id, filters)

    async def lyrics(self, skip_track_source: bool = False) -> dict | None:
        """Lyrics for the current track (needs the LavaLyrics plugin)."""
        return await self.node.current_lyrics(self.guild_id, skip_track_source)

    async def set_sponsorblock(self, categories: list[str]) -> None:
        """Set SponsorBlock categories to skip (needs the SponsorBlock plugin)."""
        await self.node.set_sponsorblock_categories(self.guild_id, categories)

    async def search(self, query: str) -> dict:
        """Resolve a URL or a search query (e.g. ``ytsearch:...``)."""
        return await self.node.load_tracks(query)

    async def destroy(self) -> None:
        """Destroy the player and its voice connection on the node."""
        await self.node.destroy(self.guild_id)
