# Nodes and connecting

A [`Node`](../reference.md) is a single connection to a Lavalink server. It owns
the WebSocket, the REST client, and the background reconnect supervisor, all
running in Rust off the GIL.

## Creating a node

```python
import wavecord

node = wavecord.Node(
    "127.0.0.1",        # host
    2333,               # port
    "youshallnotpass",  # password
    str(bot.user.id),   # your bot's user id, as a string
)
await node.connect()
```

`connect()` resolves once the node has sent its first `ready` message. If the
initial connection fails, it raises `wavecord.WaveCordError`.

## Constructor options

| Parameter | Default | Description |
| --- | --- | --- |
| `host` | required | Lavalink host. |
| `port` | required | Lavalink port. |
| `password` | required | Lavalink server password. |
| `user_id` | required | Your bot's user id, as a string. |
| `secure` | `False` | Use `wss`/`https` instead of `ws`/`http`. |
| `client_name` | set | The `Client-Name` header sent to Lavalink. |
| `session_id` | `None` | Resume an existing session id (see [Surviving restarts](resuming.md)). |
| `version` | `None` | Force `"3"` or `"4"` instead of auto-detecting. |
| `reconnect` | `True` | Reconnect automatically with backoff. |
| `resume` | `True` | Ask Lavalink to hold the session across reconnects. |
| `resume_timeout` | set | Seconds Lavalink keeps the session after a drop. |

!!! tip "Version detection"
    Leave `version` as `None` and WaveCord calls `GET /version` on connect to
    pick v3 or v4. Set it explicitly only if your node does not expose that
    endpoint, or to skip the extra request.

## Connection state

```python
node.is_connected()          # True while the WebSocket is up
await node.version()          # "3" or "4"
await node.session_id()       # the current session id, or None
```

`is_connected()` flips to `False` while the reconnect supervisor is between
attempts and back to `True` once it recovers, without you doing anything.

## REST helpers

A connected node exposes Lavalink's REST endpoints directly:

```python
await node.load_tracks("ytsearch:never gonna give you up")
await node.info()             # node version, source managers, plugins
await node.decode_track(encoded)
```

See [Searching and sources](sources.md) for `load_tracks` and `load_search`, and
[Plugins](plugins.md) for the plugin endpoints.
