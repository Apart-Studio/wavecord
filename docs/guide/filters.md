# Filters and equalizer

Filters shape the audio Lavalink sends. WaveCord exposes them the same way for
v3 and v4; the [`wavecord.filters`](../reference.md) helpers build the filter
dictionary, and `player.set_filters` applies it.

## Building a filter set

```python
from wavecord import filters

payload = filters.build(
    volume=1.0,
    timescale={"speed": 1.0, "pitch": 1.0, "rate": 1.2},
    tremolo={"frequency": 2.0, "depth": 0.5},
)
await player.set_filters(payload)
```

`build` takes any of Lavalink's filters as keyword arguments and omits anything
left as `None`:

`volume`, `equalizer`, `karaoke`, `timescale`, `tremolo`, `vibrato`, `rotation`,
`distortion`, `channel_mix`, `low_pass`, and `plugin_filters`.

## Equalizer

The `equalizer` helper turns band gains into the list Lavalink expects. Pass a
dict of `band: gain` or an iterable of `(band, gain)` pairs:

```python
eq = filters.equalizer({0: 0.25, 1: 0.25, 2: 0.15})
await player.set_filters(filters.build(equalizer=eq))
```

Bands are `0` to `14`; gains range from `-0.25` to `1.0`.

## Clearing filters

Apply an empty filter set to reset everything:

```python
await player.set_filters({})
```
