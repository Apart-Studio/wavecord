# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""The library-agnostic VoiceForwarder buffers gateway payloads and only calls
the node once session_id + token + endpoint are all present - regardless of the
order the two gateway events arrive in."""

import wavecord
from wavecord.adapters import VoiceForwarder


class FakeNode:
    def __init__(self) -> None:
        self.calls: list[tuple] = []

    async def update_voice(self, guild_id, token, endpoint, session_id, channel_id=None):
        self.calls.append((guild_id, token, endpoint, session_id, channel_id))


async def test_forwards_only_when_complete_server_first():
    node = FakeNode()
    fwd = VoiceForwarder(node, 42)

    await fwd.on_voice_server_update({"token": "T", "endpoint": "eu.discord.gg"})
    assert node.calls == []

    await fwd.on_voice_state_update({"channel_id": "5", "session_id": "S"})
    assert node.calls == [("42", "T", "eu.discord.gg", "S", "5")]


async def test_forwards_only_when_complete_state_first():
    node = FakeNode()
    fwd = VoiceForwarder(node, 42)

    await fwd.on_voice_state_update({"channel_id": "5", "session_id": "S"})
    assert node.calls == []

    await fwd.on_voice_server_update({"token": "T", "endpoint": "eu.discord.gg"})
    assert node.calls == [("42", "T", "eu.discord.gg", "S", "5")]


async def test_disconnect_resets_buffer():
    node = FakeNode()
    fwd = VoiceForwarder(node, 42)

    await fwd.on_voice_state_update({"channel_id": None, "session_id": "S"})
    await fwd.on_voice_server_update({"token": "T", "endpoint": "eu.discord.gg"})
    assert node.calls == []


def test_player_is_exported():
    assert hasattr(wavecord, "Player")
