# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""discord.py voice adapter."""

from __future__ import annotations

import discord

from ._base import build_voice_client

WaveCordVoiceClient = build_voice_client(discord.VoiceProtocol)
