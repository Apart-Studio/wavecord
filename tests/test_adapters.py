# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""The shared adapter builder wires voice callbacks to the node and joins/leaves
the channel - tested against a fake VoiceProtocol base, so no Discord library is
required to run the suite."""

import pytest

from wavecord.adapters._base import build_voice_client


class FakeVoiceProtocolBase:
    """Stands in for discord.VoiceProtocol / disnake.VoiceProtocol / ..."""

    def __init__(self, client, channel):
        self.client = client
        self.channel = channel

    def cleanup(self):
        self._cleaned = True


class FakeGuild:
    def __init__(self):
        self.id = 4242
        self.voice_states = []

    async def change_voice_state(self, *, channel, self_deaf=False, self_mute=False):
        self.voice_states.append(None if channel is None else "joined")


class FakeChannel:
    def __init__(self, guild):
        self.guild = guild


class FakeNode:
    def __init__(self):
        self.voice_updates = []
        self.destroyed = []

    async def update_voice(self, guild_id, token, endpoint, session_id, channel_id=None):
        self.voice_updates.append((guild_id, token, endpoint, session_id, channel_id))

    async def destroy(self, guild_id):
        self.destroyed.append(guild_id)


def _make():
    node = FakeNode()
    guild = FakeGuild()
    Cls = build_voice_client(FakeVoiceProtocolBase).with_node(node)
    vc = Cls(client=object(), channel=FakeChannel(guild))
    return node, guild, vc


def test_build_is_subclass_and_binds_node():
    node, _guild, vc = _make()
    assert isinstance(vc, FakeVoiceProtocolBase)
    assert vc.node is node
    assert vc.player.guild_id == "4242"


async def test_voice_handshake_forwards_to_node():
    node, _guild, vc = _make()
    await vc.on_voice_server_update({"token": "T", "endpoint": "eu.gg"})
    await vc.on_voice_state_update({"channel_id": "5", "session_id": "S"})
    assert node.voice_updates == [("4242", "T", "eu.gg", "S", "5")]


async def test_connect_and_disconnect_change_voice_state():
    node, guild, vc = _make()
    await vc.connect(timeout=60.0, reconnect=True)
    assert guild.voice_states == ["joined"]
    await vc.disconnect()
    assert guild.voice_states == ["joined", None]
    assert node.destroyed == ["4242"]


@pytest.mark.parametrize(
    "module",
    [
        "wavecord.adapters.discordpy",
        "wavecord.adapters.pycord",
        "wavecord.adapters.disnake",
        "wavecord.adapters.nextcord",
    ],
)
def test_all_adapter_modules_expose_the_class(module):
    mod = pytest.importorskip(module)
    assert hasattr(mod.WaveCordVoiceClient, "with_node")
