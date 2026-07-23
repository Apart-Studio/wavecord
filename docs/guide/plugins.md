# Plugins

WaveCord supports popular Lavalink plugins: LavaSrc, LavaSearch, LavaLyrics, and
SponsorBlock. Each requires the plugin to be installed on your Lavalink node.

## LavaSrc

LavaSrc adds Spotify, Apple Music, Deezer, and Yandex Music as search sources.
Use the [source prefixes](sources.md):

```python
from wavecord import sources

await node.load_tracks(sources.spotify("around the world"))
```

Tracks loaded through plugins carry extra data. On typed events, that is
available as `event.track.plugin_info` and `event.track.user_data`.

## LavaSearch

LavaSearch returns tracks, albums, artists, playlists, and text in one call:

```python
result = await node.load_search(
    "daft punk",
    types="track,album,artist,playlist",
)
```

## LavaLyrics

Fetch lyrics for an encoded track, or for whatever is playing in a guild:

```python
await node.lyrics(encoded)                  # for a specific track
await node.current_lyrics(guild_id)         # for the current track
await player.lyrics()                        # same, via the player
```

LavaLyrics also dispatches events: `lyrics_found`, `lyrics_not_found`, and
`lyrics_line`.

## SponsorBlock

Set the SponsorBlock categories to skip for a guild:

```python
await node.set_sponsorblock_categories(guild_id, ["sponsor", "selfpromo"])
await player.set_sponsorblock(["sponsor"])   # same, via the player
```

SponsorBlock dispatches `segments_loaded`, `segment_skipped`, `chapters_loaded`,
and `chapter_started`.

## Route planner

If your node uses a route planner, WaveCord exposes its endpoints:

```python
await node.routeplanner_status()
await node.routeplanner_free("1.2.3.4")
await node.routeplanner_free_all()
```
