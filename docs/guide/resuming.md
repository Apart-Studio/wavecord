# Surviving restarts

Lavalink can keep players alive for a short window after your bot disconnects. If
your bot restarts within that window and reconnects with the same session id,
Lavalink hands the still-playing players back, so the music never stops.

## How it works

1. On first connect, read the session id and persist it somewhere durable:

    ```python
    node = wavecord.Node("127.0.0.1", 2333, "youshallnotpass", str(bot.user.id))
    await node.connect()
    session_id = await node.session_id()
    save_session_id(session_id)   # write it to a file or database
    ```

2. On the next start, pass that id back into the constructor:

    ```python
    node = wavecord.Node(
        "127.0.0.1", 2333, "youshallnotpass", str(bot.user.id),
        session_id=load_session_id(),
    )
    await node.connect()
    ```

Lavalink resumes the session and the players continue.

## Resume window

The `resume` and `resume_timeout` constructor options control how long Lavalink
holds the session after a drop. `resume` is on by default; raise
`resume_timeout` if your restarts take longer than the default window.

## Protect the session id

A session id lets anything reconnect to your players. Store it with restrictive
permissions. The bundled example writes it with mode `0600`:

```python
import os

with open("wavecord_session.json", "w") as f:
    f.write(session_id)
os.chmod("wavecord_session.json", 0o600)
```

A complete example is in
[examples/resume_across_restart.py](https://github.com/Apart-Studio/wavecord/blob/main/examples/resume_across_restart.py).
