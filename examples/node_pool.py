# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Scaling across multiple Lavalink nodes with load balancing and failover.

The NodePool spreads guilds across nodes by live load, and a HealthMonitor moves
a guild to a healthy node if its node goes down. Set LAVALINK_NODES to a
comma-separated list of host:port:password, for example:

  export LAVALINK_NODES="127.0.0.1:2333:youshallnotpass,127.0.0.1:2334:youshallnotpass"

Run:
  pip install "discord.py>=2.3"
  export DISCORD_TOKEN=your-bot-token
  python examples/node_pool.py
"""

import os

import discord
import wavecord
from discord.ext import commands
from wavecord.adapters.discordpy import WaveCordVoiceClient
from wavecord.failover import HealthMonitor

intents = discord.Intents.default()
intents.message_content = True
bot = commands.Bot(command_prefix="!", intents=intents)

pool = wavecord.NodePool()
current_track: dict[str, str] = {}


@bot.event
async def on_ready() -> None:
    spec = os.environ.get("LAVALINK_NODES", "127.0.0.1:2333:youshallnotpass")
    for i, entry in enumerate(spec.split(",")):
        host, port, password = entry.split(":")
        await pool.add_node(f"node-{i}", host, int(port), password, str(bot.user.id))
    print(f"{bot.user} ready with {len(pool)} node(s)")

    async def replay(guild_id, old, new_node, position_ms) -> None:
        encoded = current_track.get(guild_id)
        if encoded:
            await new_node.play(guild_id, encoded, start_ms=position_ms)

    HealthMonitor(pool, replay, grace=10.0).start()


@bot.command()
async def play(ctx: commands.Context, *, query: str) -> None:
    if ctx.author.voice is None:
        return await ctx.send("Join a voice channel first.")

    node = pool.get_node(ctx.guild.id)
    vc: WaveCordVoiceClient = ctx.voice_client or await ctx.author.voice.channel.connect(
        cls=WaveCordVoiceClient.with_node(node)
    )

    result = await vc.player.search(query)
    if result["loadType"] not in ("track", "search"):
        return await ctx.send("Nothing found.")
    track = result["data"] if result["loadType"] == "track" else result["data"][0]

    current_track[str(ctx.guild.id)] = track["encoded"]
    await vc.player.play(track["encoded"])
    await ctx.send(f"Playing **{track['info']['title']}** on `{pool.assigned_label(ctx.guild.id)}`")


@bot.command()
async def leave(ctx: commands.Context) -> None:
    if ctx.voice_client:
        current_track.pop(str(ctx.guild.id), None)
        await ctx.voice_client.disconnect()


if __name__ == "__main__":
    bot.run(os.environ["DISCORD_TOKEN"])
