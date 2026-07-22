# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Audio filters and equalizer.

Play something first (with any of the other example bots or the !play here), then
apply a filter live:

  !bassboost boost the low end
  !nightcore speed and pitch up
  !eightd 8D rotation effect
  !reset clear all filters

Run:
  pip install "discord.py>=2.3"
  export DISCORD_TOKEN=your-bot-token # a Lavalink node must be running
  python examples/filters.py
"""

import os

import discord
import wavecord
from discord.ext import commands
from wavecord import filters
from wavecord.adapters.discordpy import WaveCordVoiceClient

intents = discord.Intents.default()
intents.message_content = True
bot = commands.Bot(command_prefix="!", intents=intents)

node: wavecord.Node


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
    result = await vc.player.search(query)
    if result["loadType"] not in ("track", "search"):
        return await ctx.send("Nothing found.")
    track = result["data"] if result["loadType"] == "track" else result["data"][0]
    await vc.player.play(track["encoded"])
    await ctx.send(f"Playing **{track['info']['title']}**")


@bot.command()
async def bassboost(ctx: commands.Context) -> None:
    await ctx.voice_client.player.set_filters(
        filters.build(equalizer=filters.equalizer({0: 0.25, 1: 0.25, 2: 0.2, 3: 0.1}))
    )
    await ctx.send("Bass boosted.")


@bot.command()
async def nightcore(ctx: commands.Context) -> None:
    await ctx.voice_client.player.set_filters(
        filters.build(timescale={"speed": 1.2, "pitch": 1.2, "rate": 1.0})
    )
    await ctx.send("Nightcore on.")


@bot.command()
async def eightd(ctx: commands.Context) -> None:
    await ctx.voice_client.player.set_filters(filters.build(rotation={"rotationHz": 0.2}))
    await ctx.send("8D on.")


@bot.command()
async def reset(ctx: commands.Context) -> None:
    await ctx.voice_client.player.set_filters({})
    await ctx.send("Filters cleared.")


if __name__ == "__main__":
    bot.run(os.environ["DISCORD_TOKEN"])
