# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Slash commands (discord.py app commands) instead of prefix commands.

Provides /play and /leave. No Message Content Intent needed for slash commands.

Run:
  pip install "discord.py>=2.3"
  export DISCORD_TOKEN=your-bot-token # a Lavalink node must be running
  python examples/slash_commands.py
"""

import os

import discord
import wavecord
from discord import app_commands
from wavecord.adapters.discordpy import WaveCordVoiceClient


class MusicBot(discord.Client):
    def __init__(self) -> None:
        super().__init__(intents=discord.Intents.default())
        self.tree = app_commands.CommandTree(self)
        self.node: wavecord.Node | None = None

    async def setup_hook(self) -> None:
        await self.tree.sync()

    async def on_ready(self) -> None:
        self.node = wavecord.Node("127.0.0.1", 2333, "youshallnotpass", str(self.user.id))
        await self.node.connect()
        print(f"{self.user} ready on Lavalink {await self.node.version()}")


bot = MusicBot()


@bot.tree.command(description="Play a track by search or URL")
async def play(interaction: discord.Interaction, query: str) -> None:
    if not interaction.user.voice:
        return await interaction.response.send_message("Join a voice channel first.", ephemeral=True)

    vc: WaveCordVoiceClient = interaction.guild.voice_client or await interaction.user.voice.channel.connect(
        cls=WaveCordVoiceClient.with_node(bot.node)
    )
    result = await vc.player.search(query)
    if result["loadType"] not in ("track", "search"):
        return await interaction.response.send_message("Nothing found.", ephemeral=True)

    track = result["data"] if result["loadType"] == "track" else result["data"][0]
    await vc.player.play(track["encoded"])
    await interaction.response.send_message(f"Playing **{track['info']['title']}**")


@bot.tree.command(description="Disconnect the bot")
async def leave(interaction: discord.Interaction) -> None:
    if interaction.guild.voice_client:
        await interaction.guild.voice_client.disconnect()
    await interaction.response.send_message("Bye!")


if __name__ == "__main__":
    bot.run(os.environ["DISCORD_TOKEN"])
