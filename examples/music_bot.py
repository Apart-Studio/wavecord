# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""A full-featured music bot: queue, playback controls, and auto-advance.

Commands:
  !play <query> add a track (or playlist) and start playing
  !skip skip to the next track
  !pause / !resume
  !stop stop and clear the queue
  !queue show the upcoming tracks
  !nowplaying show the current track
  !loop <off|track|queue>
  !leave

Run:
  pip install "discord.py>=2.3"
  export DISCORD_TOKEN=your-bot-token # a Lavalink node must be running
  python examples/music_bot.py
"""

import os

import discord
import wavecord
from discord.ext import commands
from wavecord.adapters.discordpy import WaveCordVoiceClient
from wavecord.dispatcher import EventDispatcher
from wavecord.queue import LoopMode, Queue, bind_autoplay

intents = discord.Intents.default()
intents.message_content = True
bot = commands.Bot(command_prefix="!", intents=intents)

node: wavecord.Node
dispatcher: EventDispatcher
queues: dict[str, Queue] = {}


@bot.event
async def on_ready() -> None:
    global node, dispatcher
    node = wavecord.Node("127.0.0.1", 2333, "youshallnotpass", str(bot.user.id))
    await node.connect()

    dispatcher = EventDispatcher(node)

    @dispatcher.on("track_start")
    async def _log(event) -> None:
        title = event.track.info.title if event.track and event.track.info else "a track"
        print(f"[{event.guild_id}] now playing: {title}")

    dispatcher.start()
    print(f"{bot.user} ready on Lavalink {await node.version()}")


def queue_for(guild_id: str) -> Queue:
    if guild_id not in queues:
        queue = Queue()
        queues[guild_id] = queue
        bind_autoplay(dispatcher, node, guild_id, queue)
    return queues[guild_id]


async def ensure_voice(ctx: commands.Context) -> WaveCordVoiceClient | None:
    if ctx.voice_client:
        return ctx.voice_client
    if ctx.author.voice is None:
        await ctx.send("Join a voice channel first.")
        return None
    return await ctx.author.voice.channel.connect(cls=WaveCordVoiceClient.with_node(node))


@bot.command()
async def play(ctx: commands.Context, *, query: str) -> None:
    vc = await ensure_voice(ctx)
    if vc is None:
        return

    result = await vc.player.search(query)
    load_type = result["loadType"]
    if load_type in ("empty", "error"):
        return await ctx.send("Nothing found.")

    queue = queue_for(str(ctx.guild.id))
    if load_type == "playlist":
        tracks = result["data"]["tracks"]
        queue.extend(tracks)
        await ctx.send(f"Queued **{len(tracks)}** tracks.")
    else:
        track = result["data"] if load_type == "track" else result["data"][0]
        queue.add(track)
        await ctx.send(f"Queued **{track['info']['title']}**.")

    if queue.current is None:
        nxt = queue.next()
        await vc.player.play(nxt["encoded"])


@bot.command()
async def skip(ctx: commands.Context) -> None:
    vc: WaveCordVoiceClient = ctx.voice_client
    if not vc:
        return
    nxt = queue_for(str(ctx.guild.id)).next()
    if nxt is None:
        await vc.player.stop()
        return await ctx.send("Queue finished.")
    await vc.player.play(nxt["encoded"])
    await ctx.send(f"Skipped to **{nxt['info']['title']}**.")


@bot.command()
async def pause(ctx: commands.Context) -> None:
    if ctx.voice_client:
        await ctx.voice_client.player.pause()


@bot.command()
async def resume(ctx: commands.Context) -> None:
    if ctx.voice_client:
        await ctx.voice_client.player.resume()


@bot.command()
async def stop(ctx: commands.Context) -> None:
    if ctx.voice_client:
        queue_for(str(ctx.guild.id)).clear()
        await ctx.voice_client.player.stop()
        await ctx.send("Stopped.")


@bot.command(name="queue")
async def show_queue(ctx: commands.Context) -> None:
    queue = queue_for(str(ctx.guild.id))
    if not queue:
        return await ctx.send("The queue is empty.")
    lines = [f"{i}. {t['info']['title']}" for i, t in enumerate(queue, 1)]
    await ctx.send("**Up next:**\n" + "\n".join(lines[:10]))


@bot.command()
async def nowplaying(ctx: commands.Context) -> None:
    current = queue_for(str(ctx.guild.id)).current
    await ctx.send(f"Now playing **{current['info']['title']}**." if current else "Nothing is playing.")


@bot.command()
async def loop(ctx: commands.Context, mode: str = "off") -> None:
    try:
        queue_for(str(ctx.guild.id)).loop = LoopMode(mode.lower())
    except ValueError:
        return await ctx.send("Use: !loop off | track | queue")
    await ctx.send(f"Loop set to **{mode.lower()}**.")


@bot.command()
async def leave(ctx: commands.Context) -> None:
    if ctx.voice_client:
        queues.pop(str(ctx.guild.id), None)
        await ctx.voice_client.disconnect()
        await ctx.send("Left the channel.")


if __name__ == "__main__":
    bot.run(os.environ["DISCORD_TOKEN"])
