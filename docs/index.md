# WaveCord

**A high-performance [Lavalink](https://lavalink.dev) client for Python, powered by a Rust core.**

WaveCord speaks both **Lavalink v3 and v4** through a single, version-neutral API,
with the version detected automatically on connect. The performance-critical work
(WebSocket, REST, serialization, reconnect, and node management) runs in native
Rust, off the Python GIL, while a thin async layer on top works with every major
Discord library.

<div class="grid cards" markdown>

- __Lavalink v3 and v4__

    One API for both protocols, auto-detected on connect.

- __Off-GIL networking__

    WebSocket, REST, JSON parsing, and v3/v4 normalization run on native tokio
    threads, keeping your event loop responsive under load.

- __Typed events__

    Handlers receive real objects (`event.track.info.title`), decoded straight
    into [msgspec](https://jcristharif.com/msgspec/) structs.

- __Library-agnostic__

    Adapters for discord.py, py-cord, disnake, and nextcord.

- __Batteries included__

    Queue with auto-advance, filters and equalizer, plugins, and a typed event
    dispatcher.

- __Built to scale__

    Multi-node pool with load balancing, reconnect with backoff, session
    resuming, and failover.

</div>

## Where to go next

- New here? Start with [Installation](installation.md) and the [Quick start](quickstart.md).
- Building a bot? Read the [Guide](guide/nodes.md) sections one by one.
- Looking for a specific class or function? See the [API reference](reference.md).

## Requirements

- Python 3.9 or newer
- A running Lavalink v3 or v4 node
- One of the supported Discord libraries (for voice)
