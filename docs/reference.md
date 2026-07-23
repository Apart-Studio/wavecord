# API reference

This page lists the public API. For task-oriented explanations, see the
[Guide](guide/nodes.md).

Everything below is importable from the top-level `wavecord` package.

## Node

`wavecord.Node(host, port, password, user_id, *, secure=False, client_name=..., session_id=None, version=None, reconnect=True, resume=True, resume_timeout=...)`

Connection and lifecycle:

- `await connect()` : open the connection, resolve on the first `ready`.
- `is_connected() -> bool`
- `await version() -> str | None`
- `await session_id() -> str | None`
- `await next_event() -> str | None` : one normalized event as JSON.
- `await next_events(max_n=...) -> list[str] | None` : a batch of events.

Playback (guild-scoped):

- `await play(guild_id, encoded, *, no_replace=False, start_ms=None, end_ms=None, volume=None, paused=None)`
- `await stop(guild_id)`
- `await set_pause(guild_id, paused)`
- `await set_volume(guild_id, volume)`
- `await seek(guild_id, position_ms)`
- `await set_filters(guild_id, filters)`
- `await update_voice(guild_id, token, endpoint, session_id, channel_id=None)`
- `await destroy(guild_id)`

REST and plugins:

- `await load_tracks(identifier)`
- `await info()`
- `await decode_track(encoded)`
- `await load_search(query, types=...)`
- `await lyrics(encoded, skip_track_source=False)`
- `await current_lyrics(guild_id, skip_track_source=False)`
- `await set_sponsorblock_categories(guild_id, categories)`
- `await routeplanner_status()` / `routeplanner_free(address)` / `routeplanner_free_all()`

## Player

`wavecord.Player(node, guild_id)`

- `await play(encoded, **kwargs)`
- `await pause(paused=True)` / `await resume()`
- `await stop()`
- `await seek(position_ms)`
- `await set_volume(volume)`
- `await set_filters(filters)`
- `await lyrics(skip_track_source=False)`
- `await set_sponsorblock(categories)`
- `await search(query)`
- `await destroy()`

## EventDispatcher

`wavecord.EventDispatcher(node)`

- `on(name)` : decorator to register a handler.
- `add_listener(name, fn)` / `remove_listener(name, fn)`
- `start() -> asyncio.Task`
- `await stop()`

See [Events](guide/events.md) for event names and the typed event objects in
`wavecord.events`.

## Queue

`wavecord.Queue(loop=LoopMode.OFF)`

- `add(track)` / `extend(tracks)` / `clear()` / `shuffle()`
- `next() -> Track | None`
- `current` : the track playing now, or `None`.
- `len(queue)`, `iter(queue)`, `bool(queue)`

`wavecord.LoopMode` : `OFF`, `TRACK`, `QUEUE`.

`wavecord.queue.bind_autoplay(dispatcher, node, guild_id, queue)` : play the next
track when the current one ends naturally. Returns the handler.

## NodePool

`wavecord.NodePool()`

- `await add_node(label, host, port, password, user_id, **kwargs) -> PooledNode`
- `register(label, node, dispatcher) -> PooledNode`
- `nodes() -> list[PooledNode]`
- `best() -> PooledNode | None`
- `get_node(guild_id) -> Node`
- `assign(guild_id) -> PooledNode`
- `get_pooled(label) -> PooledNode | None`
- `assigned_label(guild_id) -> str | None`
- `guilds_on(label) -> list[str]`

`wavecord.PooledNode` : a node plus its stats and penalty score.

`wavecord.failover(pool, label, replay)` and `wavecord.HealthMonitor` handle node
loss. See [Node pool and scaling](guide/pool.md).

## filters

`wavecord.filters.build(*, volume=None, equalizer=None, karaoke=None, timescale=None, tremolo=None, vibrato=None, rotation=None, distortion=None, channel_mix=None, low_pass=None, plugin_filters=None) -> dict`

`wavecord.filters.equalizer(bands) -> list[dict]` : `bands` is a `{band: gain}`
dict or an iterable of `(band, gain)` pairs.

## sources

`wavecord.sources.youtube(query)`, `youtube_music`, `soundcloud`, `spotify`,
`apple_music`, `deezer`, `yandex_music`. Each returns a query string for
`node.load_tracks`.

## metrics

`wavecord.metrics.snapshot(pool) -> list[dict]`

`wavecord.metrics.prometheus(pool) -> str`

## Errors

`wavecord.WaveCordError` : the single exception type for connection, REST, and
protocol failures.
