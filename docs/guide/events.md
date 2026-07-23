# Events

Lavalink streams events over the WebSocket. WaveCord parses and normalizes them
in Rust, then the [`EventDispatcher`](../reference.md) decodes each one into a
typed [msgspec](https://jcristharif.com/msgspec/) struct and calls your handlers.

## Registering handlers

```python
from wavecord.dispatcher import EventDispatcher

dispatcher = EventDispatcher(node)


@dispatcher.on("track_end")
async def on_track_end(event):
    print(event.guild_id, event.reason)


dispatcher.start()   # begins pumping events in the background
```

`start()` returns the background `asyncio.Task`. Call `await dispatcher.stop()`
to stop pumping during shutdown.

You can also register and remove handlers imperatively:

```python
async def handler(event): ...

dispatcher.add_listener("track_start", handler)
dispatcher.remove_listener("track_start", handler)
```

## Typed event objects

Handlers receive real objects, not raw dictionaries. Attributes are decoded once
and typed:

```python
@dispatcher.on("track_start")
async def on_track_start(event):
    print(event.track.info.title, "by", event.track.info.author)
```

## Event names

| Name | Fired when |
| --- | --- |
| `ready` | The node is ready and has sent its session id. |
| `player_update` | Periodic player state (position, connection, ping). |
| `stats` | Node health metrics. |
| `track_start` | A track starts playing. |
| `track_end` | A track ends. `event.reason` says why. |
| `track_exception` | A track raised an exception. |
| `track_stuck` | A track got stuck. |
| `websocket_closed` | The Discord voice WebSocket closed. |

Plugin events (SponsorBlock and LavaLyrics) are documented in
[Plugins](plugins.md).

!!! note "Only decode what you listen to"
    The dispatcher peeks at each message's name first and skips a full decode
    when no handler is registered for it. Register a `"*"` handler to receive
    every event.
