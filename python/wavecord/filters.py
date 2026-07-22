# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Helpers to build Lavalink filter payloads for ``Node.set_filters``."""

from __future__ import annotations

from collections.abc import Iterable
from typing import Any


def equalizer(bands: dict[int, float] | Iterable[tuple[int, float]]) -> list[dict[str, Any]]:
    """Build an equalizer list from ``{band: gain}`` (band 0-14, gain -0.25..1.0)."""
    items = bands.items() if isinstance(bands, dict) else bands
    return [{"band": int(b), "gain": float(g)} for b, g in items]


def build(
    *,
    volume: float | None = None,
    equalizer: list[dict[str, Any]] | None = None,  # noqa: A002
    karaoke: dict[str, Any] | None = None,
    timescale: dict[str, Any] | None = None,
    tremolo: dict[str, Any] | None = None,
    vibrato: dict[str, Any] | None = None,
    rotation: dict[str, Any] | None = None,
    distortion: dict[str, Any] | None = None,
    channel_mix: dict[str, Any] | None = None,
    low_pass: dict[str, Any] | None = None,
    plugin_filters: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Assemble a filters dict, omitting anything left as ``None``."""
    out: dict[str, Any] = {}
    if volume is not None:
        out["volume"] = volume
    if equalizer is not None:
        out["equalizer"] = equalizer
    if karaoke is not None:
        out["karaoke"] = karaoke
    if timescale is not None:
        out["timescale"] = timescale
    if tremolo is not None:
        out["tremolo"] = tremolo
    if vibrato is not None:
        out["vibrato"] = vibrato
    if rotation is not None:
        out["rotation"] = rotation
    if distortion is not None:
        out["distortion"] = distortion
    if channel_mix is not None:
        out["channelMix"] = channel_mix
    if low_pass is not None:
        out["lowPass"] = low_pass
    if plugin_filters is not None:
        out["pluginFilters"] = plugin_filters
    return out
