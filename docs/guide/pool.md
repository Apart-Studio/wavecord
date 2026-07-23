# Node pool and scaling

For larger bots you run several Lavalink nodes and spread guilds across them. The
[`NodePool`](../reference.md) connects nodes, balances load using Lavalink's own
penalty scoring, and keeps guild assignments sticky.

## Building a pool

```python
from wavecord.pool import NodePool

pool = NodePool()
await pool.add_node("eu-1", "10.0.0.1", 2333, "youshallnotpass", str(bot.user.id))
await pool.add_node("eu-2", "10.0.0.2", 2333, "youshallnotpass", str(bot.user.id))
```

`add_node` creates the node, connects it, starts an event dispatcher, and
registers it. Extra keyword arguments are forwarded to `Node`.

## Picking a node

```python
pool.best()                      # the least loaded available node
node = pool.get_node(guild_id)   # the node assigned to this guild (sticky)
pool.assign(guild_id)            # assign a guild and return its node
```

`get_node` always returns the same node for a given guild until you reassign it,
so a guild's player stays on one node.

## Inspecting the pool

```python
pool.nodes()                     # every PooledNode
pool.get_pooled("eu-1")          # a node by label
pool.assigned_label(guild_id)    # which label a guild is on
pool.guilds_on("eu-1")           # guild ids currently on a node
```

## Failover

If a node goes down, [`failover`](../reference.md) moves its guilds to a healthy
node and replays each player at its last position through a callback you supply.
[`HealthMonitor`](../reference.md) can run this automatically.

```python
from wavecord.failover import failover, HealthMonitor

async def replay(guild_id, node, position_ms):
    # re-issue play for guild_id on the new node, seeking to position_ms
    ...

moved = await failover(pool, "eu-1", replay)

monitor = HealthMonitor(pool, replay)
```

See the [API reference](../reference.md) for the exact signatures, and
`wavecord.metrics` in [Metrics](metrics.md) to observe the pool.
