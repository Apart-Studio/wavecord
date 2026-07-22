# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Plugin features and observability.

Shows Spotify search (LavaSrc), SponsorBlock, lyrics (LavaLyrics), and a metrics
snapshot. The plugin commands need the matching Lavalink plugins installed.

Commands: !spotify <query> !sponsorblock !lyrics !metrics
"""

import os

import discord
import wavecord
from discord.ext import commands
from wavecord import metrics, sources
from wavecord.adapters.discordpy import WaveCordVoiceClient

intents = discord.Intents.default()
intents.message_content = True
bot = commands.Bot(command_prefix="!", intents=intents)

pool = wavecord.NodePool()


@bot.event
async def on_ready() -> None:
    await pool.add_node("main", "127.0.0.1", 2333, "youshallnotpass", str(bot.user.id))
    print(f"{bot.user} ready")


@bot.command()
async def spotify(ctx: commands.Context, *, query: str) -> None:
    node = pool.get_node(ctx.guild.id)
    vc: WaveCordVoiceClient = ctx.voice_client or await ctx.author.voice.channel.connect(
        cls=WaveCordVoiceClient.with_node(node)
    )
    result = await vc.player.search(sources.spotify(query))
    if result["loadType"] not in ("track", "search"):
        return await ctx.send("Nothing found (is LavaSrc installed?).")
    track = result["data"] if result["loadType"] == "track" else result["data"][0]
    await vc.player.play(track["encoded"])
    await ctx.send(f"Playing **{track['info']['title']}**")


@bot.command()
async def sponsorblock(ctx: commands.Context) -> None:
    if ctx.voice_client:
        await ctx.voice_client.player.set_sponsorblock(["sponsor", "selfpromo", "interaction"])
        await ctx.send("SponsorBlock segments will be skipped.")


@bot.command()
async def lyrics(ctx: commands.Context) -> None:
    if ctx.voice_client:
        result = await ctx.voice_client.player.lyrics()
        await ctx.send("No lyrics." if not result else str(result.get("text", result))[:1900])


@bot.command(name="metrics")
async def show_metrics(ctx: commands.Context) -> None:
    await ctx.send(f"```\n{metrics.prometheus(pool)}\n```")


if __name__ == "__main__":
    bot.run(os.environ["DISCORD_TOKEN"])
