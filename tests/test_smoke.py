# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors
"""Smoke tests: the extension imports and the async bridge works."""

import wavecord


def test_version_is_exposed():
    assert isinstance(wavecord.__version__, str)
    assert wavecord.__version__


async def test_ping_bridges_tokio_to_asyncio():
    assert await wavecord.ping() == "pong"
