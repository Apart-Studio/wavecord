# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Shared voice-client builder for the discord.py-family adapters."""

from __future__ import annotations

from typing import Any

from .._wavecord import Node
from ..player import Player
from .forwarder import VoiceForwarder


def build_voice_client(voice_protocol_base: type) -> type:
    """Return a WaveCord voice-client class subclassing the given ``VoiceProtocol``."""

    class WaveCordVoiceClient(voice_protocol_base):  # type: ignore[valid-type, misc]
        """Voice client that routes the Discord voice handshake to a WaveCord node
        and exposes a :class:`~wavecord.player.Player`."""

        node: Node

        def __init__(self, client: Any, channel: Any) -> None:
            super().__init__(client, channel)
            self.client = client
            self.channel = channel
            guild_id = channel.guild.id
            self._forwarder = VoiceForwarder(self.node, guild_id)
            self.player = Player(self.node, guild_id)

        @classmethod
        def with_node(cls, node: Node) -> type[WaveCordVoiceClient]:
            """Bind a node and return a subclass to pass as ``channel.connect(cls=...)``."""
            return type(cls.__name__, (cls,), {"node": node})

        async def on_voice_state_update(self, data: dict) -> None:
            await self._forwarder.on_voice_state_update(data)

        async def on_voice_server_update(self, data: dict) -> None:
            await self._forwarder.on_voice_server_update(data)

        async def connect(self, *, timeout: float, reconnect: bool, **kwargs: Any) -> None:
            await self.channel.guild.change_voice_state(
                channel=self.channel,
                self_deaf=kwargs.get("self_deaf", False),
                self_mute=kwargs.get("self_mute", False),
            )

        async def disconnect(self, *, force: bool = False) -> None:
            try:
                await self.channel.guild.change_voice_state(channel=None)
                await self.player.destroy()
            finally:
                self.cleanup()

    return WaveCordVoiceClient
