# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""The filters builders assemble the expected wire dict, and Node.set_filters
accepts it (dict -> Rust serde_json::Value round-trip)."""


import wavecord
from wavecord import filters


def test_equalizer_from_dict_and_pairs():
    assert filters.equalizer({0: 0.2, 3: -0.1}) == [
        {"band": 0, "gain": 0.2},
        {"band": 3, "gain": -0.1},
    ]
    assert filters.equalizer([(1, 0.5)]) == [{"band": 1, "gain": 0.5}]


def test_build_omits_none_and_maps_names():
    f = filters.build(
        volume=1.5, equalizer=filters.equalizer({0: 0.25}), low_pass={"smoothing": 20}
    )
    assert f == {
        "volume": 1.5,
        "equalizer": [{"band": 0, "gain": 0.25}],
        "lowPass": {"smoothing": 20},
    }
    assert "karaoke" not in f


async def test_set_filters_accepts_dict_but_requires_connection():
    node = wavecord.Node("127.0.0.1", 2333, "pw", "1")
    try:
        await node.set_filters("42", filters.build(volume=2.0))
    except wavecord.WaveCordError as e:
        assert "not connected" in str(e).lower()
    else:
        raise AssertionError("expected a not-connected error")
