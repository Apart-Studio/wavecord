# Searching and sources

Loading tracks goes through Lavalink. WaveCord gives you the raw REST call and a
set of prefix helpers so you do not have to remember each search scheme.

## Loading tracks

```python
result = await node.load_tracks("ytsearch:daft punk around the world")
```

The result follows Lavalink's `loadType`:

| `loadType` | `data` holds |
| --- | --- |
| `track` | a single track object |
| `search` | a list of track objects |
| `playlist` | a playlist with `info` and `tracks` |
| `empty` | nothing was found |
| `error` | an exception describing what went wrong |

```python
if result["loadType"] == "search":
    first = result["data"][0]
    await player.play(first["encoded"])
```

## Source prefixes

The [`wavecord.sources`](../reference.md) helpers build the query string for a
given search source. They require the matching Lavalink source manager or
plugin to be installed on your node.

```python
from wavecord import sources

await node.load_tracks(sources.youtube("daft punk"))
await node.load_tracks(sources.soundcloud("lofi"))
await node.load_tracks(sources.spotify("around the world"))
```

Available helpers: `youtube`, `youtube_music`, `soundcloud`, `spotify`,
`apple_music`, `deezer`, and `yandex_music`. The Spotify, Apple Music, Deezer,
and Yandex helpers rely on the [LavaSrc](https://github.com/topi314/LavaSrc)
plugin.

For the LavaSearch plugin's richer results (tracks, albums, artists, playlists,
and text), use `node.load_search`; see [Plugins](plugins.md).
