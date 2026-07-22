# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Search-source prefix helpers."""

from wavecord import sources


def test_prefixes():
    assert sources.youtube("a") == "ytsearch:a"
    assert sources.youtube_music("a") == "ytmsearch:a"
    assert sources.soundcloud("a") == "scsearch:a"
    assert sources.spotify("a") == "spsearch:a"
    assert sources.apple_music("a") == "amsearch:a"
    assert sources.deezer("a") == "dzsearch:a"
    assert sources.yandex_music("a") == "ymsearch:a"
