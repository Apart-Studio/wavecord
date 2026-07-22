# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Same idea as basic.py, but on nextcord.

WaveCord is library-agnostic; only the import of the adapter changes.

Run:
  pip install "nextcord>=2.6"
  export DISCORD_TOKEN=your-bot-token # a Lavalink node must be running
  python examples/nextcord_bot.py
"""

import os

import nextcord
import wavecord
from nextcord.ext import commands
from wavecord.adapters.nextcord import WaveCordVoiceClient

intents = nextcord.Intents.default()
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
async def leave(ctx: commands.Context) -> None:
    if ctx.voice_client:
        await ctx.voice_client.disconnect()


if __name__ == "__main__":
    bot.run(os.environ["DISCORD_TOKEN"])
