# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""nextcord voice adapter."""

from __future__ import annotations

import nextcord

from ._base import build_voice_client

WaveCordVoiceClient = build_voice_client(nextcord.VoiceProtocol)
