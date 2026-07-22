# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Library-agnostic voice-state collector shared by all Discord adapters."""

from __future__ import annotations

from .._wavecord import Node


class VoiceForwarder:
    """Buffers the Discord voice values and forwards them to the node once the
    session id, token and endpoint are all known."""

    def __init__(self, node: Node, guild_id: int | str) -> None:
        self.node = node
        self.guild_id = str(guild_id)
        self._session_id: str | None = None
        self._token: str | None = None
        self._endpoint: str | None = None
        self._channel_id: str | None = None

    async def on_voice_state_update(self, data: dict) -> None:
        """Feed a raw ``VOICE_STATE_UPDATE`` payload."""
        if data.get("channel_id") is None:
            self._session_id = self._token = self._endpoint = self._channel_id = None
            return
        self._session_id = data["session_id"]
        self._channel_id = str(data["channel_id"])
        await self._flush()

    async def on_voice_server_update(self, data: dict) -> None:
        """Feed a raw ``VOICE_SERVER_UPDATE`` payload."""
        self._token = data["token"]
        self._endpoint = data.get("endpoint")
        await self._flush()

    async def _flush(self) -> None:
        if self._session_id and self._token and self._endpoint and self._channel_id:
            await self.node.update_voice(
                self.guild_id,
                self._token,
                self._endpoint,
                self._session_id,
                channel_id=self._channel_id,
            )
