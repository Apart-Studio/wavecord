# Discord libraries

WaveCord is library-agnostic. A thin adapter connects your Discord library's
voice gateway to a node and gives you a `player`. The pattern is the same for
every library: connect the voice channel with the WaveCord voice client bound to
your node.

| Library | Import |
| --- | --- |
| [discord.py](https://github.com/Rapptz/discord.py) | `wavecord.adapters.discordpy` |
| [py-cord](https://github.com/Pycord-Development/pycord) | `wavecord.adapters.pycord` |
| [disnake](https://github.com/DisnakeDev/disnake) | `wavecord.adapters.disnake` |
| [nextcord](https://github.com/nextcord/nextcord) | `wavecord.adapters.nextcord` |

## Connecting

=== "discord.py"

    ```python
    from wavecord.adapters.discordpy import WaveCordVoiceClient

    vc = await channel.connect(cls=WaveCordVoiceClient.with_node(node))
    await vc.player.play(track["encoded"])
    ```

=== "py-cord"

    ```python
    from wavecord.adapters.pycord import WaveCordVoiceClient

    vc = await channel.connect(cls=WaveCordVoiceClient.with_node(node))
    await vc.player.play(track["encoded"])
    ```

=== "disnake"

    ```python
    from wavecord.adapters.disnake import WaveCordVoiceClient

    vc = await channel.connect(cls=WaveCordVoiceClient.with_node(node))
    await vc.player.play(track["encoded"])
    ```

=== "nextcord"

    ```python
    from wavecord.adapters.nextcord import WaveCordVoiceClient

    vc = await channel.connect(cls=WaveCordVoiceClient.with_node(node))
    await vc.player.play(track["encoded"])
    ```

`WaveCordVoiceClient.with_node(node)` binds the voice client to a node. Once
connected, `vc.player` is a [`Player`](guide/players.md) for that guild, and the
adapter forwards Discord's voice updates to the node for you.

## Runnable examples

Each library has a full example bot in the repository:

- [examples/music_bot.py](https://github.com/Apart-Studio/wavecord/blob/main/examples/music_bot.py) (discord.py)
- [examples/disnake_bot.py](https://github.com/Apart-Studio/wavecord/blob/main/examples/disnake_bot.py)
- [examples/nextcord_bot.py](https://github.com/Apart-Studio/wavecord/blob/main/examples/nextcord_bot.py)
- [examples/slash_commands.py](https://github.com/Apart-Studio/wavecord/blob/main/examples/slash_commands.py)
