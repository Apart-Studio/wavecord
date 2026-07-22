# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""disnake voice adapter."""

from __future__ import annotations

import disnake

from ._base import build_voice_client

WaveCordVoiceClient = build_voice_client(disnake.VoiceProtocol)
