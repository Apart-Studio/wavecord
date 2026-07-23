# Players and playback

A [`Player`](../reference.md) is a per-guild handle that sends playback commands
to a node. You usually get one from a voice client (`vc.player`), but you can
also construct one directly:

```python
player = wavecord.Player(node, guild_id)
```

## Searching and playing

`search` loads a query or URL through Lavalink, and `play` starts an encoded
track:

```python
result = await player.search("ytsearch:daft punk")
track = result["data"][0]          # a search result is a list
await player.play(track["encoded"])
```

The shape of `result` follows Lavalink's `loadType` (`track`, `search`,
`playlist`, `empty`, or `error`). See [Searching and sources](sources.md).

## Controls

```python
await player.pause()             # or pause(False) to resume
await player.resume()
await player.set_volume(50)      # 0 to 1000
await player.seek(30_000)        # milliseconds
await player.stop()
await player.destroy()           # tear the player down on the node
```

`play` accepts the same extra options Lavalink does, as keyword arguments:

```python
await player.play(
    track["encoded"],
    start_ms=15_000,
    end_ms=60_000,
    volume=80,
    paused=False,
    no_replace=False,
)
```

## Filters and plugins

Players also expose the audio filters and the plugin commands:

```python
await player.set_filters(filters)          # see the Filters guide
await player.lyrics()                        # LavaLyrics, see Plugins
await player.set_sponsorblock(["sponsor"])   # SponsorBlock, see Plugins
```

See [Filters and equalizer](filters.md) and [Plugins](plugins.md).
