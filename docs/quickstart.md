# Quick start

This is a minimal music bot using discord.py. It connects to a Lavalink node,
plays a track, and prints when playback finishes.

```python
import discord
from discord.ext import commands

import wavecord
from wavecord.adapters.discordpy import WaveCordVoiceClient
from wavecord.dispatcher import EventDispatcher

intents = discord.Intents.default()
intents.message_content = True
bot = commands.Bot(command_prefix="!", intents=intents)
node: wavecord.Node


@bot.event
async def on_ready():
    global node
    node = wavecord.Node("127.0.0.1", 2333, "youshallnotpass", str(bot.user.id))
    await node.connect()

    dispatcher = EventDispatcher(node)

    @dispatcher.on("track_end")
    async def on_track_end(event):  # a typed wavecord.events.Event
        print("finished", event.guild_id, event.reason)

    dispatcher.start()


@bot.command()
async def play(ctx, *, query: str):
    vc = await ctx.author.voice.channel.connect(
        cls=WaveCordVoiceClient.with_node(node)
    )
    result = await vc.player.search(query)
    track = result["data"] if result["loadType"] == "track" else result["data"][0]
    await vc.player.play(track["encoded"])
    await ctx.send(f"Playing {track['info']['title']}")


bot.run("YOUR_TOKEN")
```

## What is happening here

1. **The node** is your connection to Lavalink. You create one
   [`wavecord.Node`](reference.md), call `await node.connect()`, and WaveCord
   detects whether the server speaks v3 or v4.
2. **The dispatcher** turns the node's event stream into typed callbacks. You
   register handlers with `@dispatcher.on(name)` and call `dispatcher.start()`
   to begin pumping events in the background. See [Events](guide/events.md).
3. **The voice client** is the adapter for your Discord library. Connecting with
   `cls=WaveCordVoiceClient.with_node(node)` wires the voice gateway to the node
   and gives you a `vc.player`. See [Discord libraries](adapters.md).
4. **The player** issues the actual commands: `search`, `play`, `pause`, and so
   on. See [Players and playback](guide/players.md).

## Next steps

- [Nodes and connecting](guide/nodes.md) covers every constructor option.
- [Queue](guide/queue.md) adds a track queue with auto-advance.
- [Filters and equalizer](guide/filters.md) shapes the audio.
- A fuller example bot lives in
  [examples/music_bot.py](https://github.com/Apart-Studio/wavecord/blob/main/examples/music_bot.py).
