# Queue

WaveCord ships a small, Python-side [`Queue`](../reference.md). Queue operations
are not a hot path, so they live in Python for flexibility; the performance work
stays in Rust.

## Basics

```python
from wavecord.queue import Queue, LoopMode

queue = Queue()
queue.add(track)                 # a track dict from search
queue.extend([track1, track2])
queue.shuffle()
queue.clear()

len(queue)                       # tracks waiting
bool(queue)                      # True if any are waiting
queue.current                    # the track playing now, or None
```

`next()` advances the queue and returns the next track, honoring the loop mode:

```python
track = queue.next()
if track is not None:
    await player.play(track["encoded"])
```

## Loop modes

```python
queue = Queue(loop=LoopMode.QUEUE)
```

| Mode | Behavior |
| --- | --- |
| `LoopMode.OFF` | Play each track once, then stop. |
| `LoopMode.TRACK` | Repeat the current track. |
| `LoopMode.QUEUE` | Send finished tracks to the back of the queue. |

## Auto-advance

`bind_autoplay` wires a queue to a dispatcher so the next track plays
automatically when the current one ends naturally. Manual stops (stopped,
replaced, cleanup) do not advance.

```python
from wavecord.queue import bind_autoplay

bind_autoplay(dispatcher, node, guild_id, queue)
```

It returns the registered handler, so you can remove it later with
`dispatcher.remove_listener("track_end", handler)`.
