# Metrics

The [`wavecord.metrics`](../reference.md) module turns a node pool into
observability data: a plain snapshot for your own dashboards, or Prometheus
exposition text for a `/metrics` endpoint.

## Snapshot

```python
from wavecord import metrics

data = metrics.snapshot(pool)
```

`snapshot` returns a list of dictionaries, one per node, with its label, whether
it is available, its player counts, and its penalty score.

## Prometheus

```python
text = metrics.prometheus(pool)
```

`prometheus` returns exposition text you can serve directly. A minimal endpoint
with aiohttp:

```python
from aiohttp import web
from wavecord import metrics


async def handle_metrics(request):
    return web.Response(text=metrics.prometheus(request.app["pool"]))


app = web.Application()
app["pool"] = pool
app.router.add_get("/metrics", handle_metrics)
```

Label values are escaped, so node labels are safe to expose.
