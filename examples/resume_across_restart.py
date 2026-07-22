# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Playback survives a bot restart.

WaveCord persists the Lavalink session id to a file and reuses it on startup. If
the bot process restarts within the node's resume timeout, Lavalink keeps the
players alive and audio never stops; WaveCord reconnects to the same session (the
``ready`` event reports ``resumed=True``).

Try it: run the bot, !play something, stop the bot with Ctrl-C, then start it
again within the resume timeout. The music keeps playing, and the log prints
"resumed session".
"""

import json
import os
import pathlib

import discord
import wavecord
from discord.ext import commands
from wavecord.adapters.discordpy import WaveCordVoiceClient
from wavecord.dispatcher import EventDispatcher

SESSION_FILE = pathlib.Path("wavecord_session.json")

intents = discord.Intents.default()
intents.message_content = True
bot = commands.Bot(command_prefix="!", intents=intents)

node: wavecord.Node


def saved_session() -> str | None:
    if SESSION_FILE.exists():
        return json.loads(SESSION_FILE.read_text()).get("session_id")
    return None


@bot.event
async def on_ready() -> None:
    global node
    node = wavecord.Node(
        "127.0.0.1", 2333, "youshallnotpass", str(bot.user.id),
        session_id=saved_session(),
        resume=True,
        resume_timeout=120,
    )

    dispatcher = EventDispatcher(node)

    @dispatcher.on("ready")
    async def _on_ready(event) -> None:
        SESSION_FILE.write_text(json.dumps({"session_id": event.session_id}))
        SESSION_FILE.chmod(0o600)  # the session id controls the node's players
        print("resumed session" if event.resumed else "fresh session", event.session_id)

    await node.connect()
    dispatcher.start()
    print(f"{bot.user} ready on Lavalink {await node.version()}")


@bot.command()
async def play(ctx: commands.Context, *, query: str) -> None:
    vc: WaveCordVoiceClient = ctx.voice_client or await ctx.author.voice.channel.connect(
        cls=WaveCordVoiceClient.with_node(node)
    )
    result = await vc.player.search(query)
    if result["loadType"] not in ("track", "search"):
        return await ctx.send("Nothing found.")
    track = result["data"] if result["loadType"] == "track" else result["data"][0]
    await vc.player.play(track["encoded"])
    await ctx.send(f"Playing **{track['info']['title']}**")


if __name__ == "__main__":
    bot.run(os.environ["DISCORD_TOKEN"])
