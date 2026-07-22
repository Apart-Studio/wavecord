# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). The public Python API
(`Node`, `Player`, `EventDispatcher`, `NodePool`, `events`, and the adapters) is
stable.

## [1.1.0]

First public release. Core behavior is verified end to end against live Lavalink
3.7.13 and 4.2.2 nodes, including real audio playback.

### Core
- Lavalink **v3 and v4** behind one version-neutral API, auto-detected on connect
  (v4 commands over REST, v3 over WebSocket ops; v3 events and loadtracks are
  normalized to the v4 shape).
- A pure-Rust core (`wavecord-core`) with PyO3 bindings and an async bridge
  between tokio and asyncio. WebSocket, REST, parsing, and reconnect run off the
  GIL.
- Typed events decoded straight into msgspec structs, a per-event name peek that
  skips decoding events with no listener, and batched delivery via
  `Node.next_events`.
- Player controls, a queue with loop modes and auto-advance, and version-neutral
  filters and equalizer.

### Discord libraries
- Adapters for discord.py, py-cord, disnake, and nextcord.

### Resilience and scaling
- Automatic reconnect with exponential backoff, session resuming, a multi-node
  pool with load balancing, and failover.
- Playback survives a bot restart: persist and reuse the session id so Lavalink
  resumes the still-alive players.

### Plugins and endpoints
- Plugin support: track `pluginInfo` and `userData` on typed events; dispatched
  SponsorBlock and LavaLyrics events; `load_search` (LavaSearch), `lyrics` and
  `current_lyrics` (LavaLyrics), and `set_sponsorblock_categories`.
- REST endpoints: `info`, `decode_track`, and route planner
  (`routeplanner_status` / `routeplanner_free` / `routeplanner_free_all`).
- `wavecord.sources` search-source prefixes (YouTube, YouTube Music, SoundCloud,
  Spotify, Apple Music, Deezer, Yandex Music) and `wavecord.metrics` for a pool
  snapshot and Prometheus exposition text.

### Errors and security
- `WaveCordError`, a single exception type for connection, REST, and protocol
  failures.
- REST URL path segments are percent-encoded and Prometheus label values are
  escaped (defense in depth).

[1.1.0]: https://github.com/Apart-Studio/wavecord/releases/tag/1.1.0
