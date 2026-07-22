# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""The smallest possible WaveCord bot: join, play, leave (discord.py).

Run:
  pip install "discord.py>=2.3"
  export DISCORD_TOKEN=your-bot-token # a Lavalink node must be running
  python examples/basic.py

Then, in a server: join a voice channel and type !play <search or url>
Enable the Message Content Intent for your bot in the Discord Developer Portal.
"""

import os

import discord
import wavecord
from discord.ext import commands
from wavecord.adapters.discordpy import WaveCordVoiceClient

intents = discord.Intents.default()
intents.message_content = True
bot = commands.Bot(command_prefix="!", intents=intents)

node: wavecord.Node


def first_track(result: dict) -> dict | None:
    """Pull a single playable track out of a loadtracks result."""
    load_type = result["loadType"]
    if load_type == "track":
        return result["data"]
    if load_type == "search":
        return result["data"][0] if result["data"] else None
    if load_type == "playlist":
        return result["data"]["tracks"][0]
    return None


@bot.event
async def on_ready() -> None:
    global node
    node = wavecord.Node("127.0.0.1", 2333, "youshallnotpass", str(bot.user.id))
    await node.connect()
    print(f"{bot.user} ready on Lavalink {await node.version()}")


@bot.command()
async def play(ctx: commands.Context, *, query: str) -> None:
    if ctx.author.voice is None:
        return await ctx.send("Join a voice channel first.")

    vc: WaveCordVoiceClient = ctx.voice_client or await ctx.author.voice.channel.connect(
        cls=WaveCordVoiceClient.with_node(node)
    )

    track = first_track(await vc.player.search(query))
    if track is None:
        return await ctx.send("Nothing found.")

    await vc.player.play(track["encoded"])
    await ctx.send(f"Playing **{track['info']['title']}**")


@bot.command()
async def leave(ctx: commands.Context) -> None:
    if ctx.voice_client:
        await ctx.voice_client.disconnect()
        await ctx.send("Bye!")


if __name__ == "__main__":
    bot.run(os.environ["DISCORD_TOKEN"])
