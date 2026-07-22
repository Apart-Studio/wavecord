# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Search-source prefix helpers for ``Node.load_tracks`` / ``Player.search``.

Each returns the query with the right Lavalink source prefix. The plugin-backed
sources (Spotify, Apple Music, Deezer, Yandex) require the LavaSrc plugin on your
Lavalink node.
"""

from __future__ import annotations


def youtube(query: str) -> str:
    """``ytsearch:`` (built-in YouTube source or the YouTube plugin)."""
    return f"ytsearch:{query}"


def youtube_music(query: str) -> str:
    """``ytmsearch:`` (YouTube Music)."""
    return f"ytmsearch:{query}"


def soundcloud(query: str) -> str:
    """``scsearch:`` (SoundCloud)."""
    return f"scsearch:{query}"


def spotify(query: str) -> str:
    """``spsearch:`` (Spotify, via LavaSrc)."""
    return f"spsearch:{query}"


def apple_music(query: str) -> str:
    """``amsearch:`` (Apple Music, via LavaSrc)."""
    return f"amsearch:{query}"


def deezer(query: str) -> str:
    """``dzsearch:`` (Deezer, via LavaSrc)."""
    return f"dzsearch:{query}"


def yandex_music(query: str) -> str:
    """``ymsearch:`` (Yandex Music, via LavaSrc)."""
    return f"ymsearch:{query}"
