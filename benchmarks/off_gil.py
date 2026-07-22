# SPDX-License-Identifier: MIT
# Copyright (c) 2026 WaveCord contributors

from __future__ import annotations

import asyncio
import gc
import json
import multiprocessing as mp
import time

N_EVENTS = 500_000
HEARTBEAT_INTERVAL = 0.005
BATCH = 256
WARMUP = 20_000
TRIALS = 3
PORT = 8791


def _messages(n):
    ready = json.dumps({"op": "ready", "resumed": False, "sessionId": "bench"})
    out = [ready]
    for i in range(n):
        out.append(
            json.dumps(
                {
                    "op": "playerUpdate",
                    "guildId": "1234567890",
                    "state": {
                        "time": 1710000000000 + i,
                        "position": i * 20,
                        "connected": True,
                        "ping": 42,
                    },
                }
            )
        )
    return out


def _server_process(port, n):
    import websockets

    payloads = _messages(n)

    async def handler(ws):
        for msg in payloads:
            await ws.send(msg)
        try:
            await ws.wait_closed()
        except Exception:
            pass

    async def main():
        async with websockets.serve(handler, "127.0.0.1", port, max_queue=None):
            await asyncio.Future()

    asyncio.run(main())


class Heartbeat:
    def __init__(self, interval):
        self.interval = interval
        self.samples = []
        self._stop = False

    async def run(self):
        while not self._stop:
            t0 = time.perf_counter()
            await asyncio.sleep(self.interval)
            late_ms = (time.perf_counter() - t0 - self.interval) * 1000.0
            self.samples.append(max(0.0, late_ms))

    def stop(self):
        self._stop = True

    def stats(self):
        s = sorted(self.samples)
        if not s:
            return {"p50": 0.0, "p99": 0.0, "max": 0.0}
        def pct(q):
            return s[min(len(s) - 1, int(q * len(s)))]

        return {"p50": pct(0.50), "p99": pct(0.99), "max": s[-1]}


async def _measured(consume_one, n):
    got = 0
    while got < WARMUP:
        got += await consume_one()

    gc.collect()
    gc.disable()
    try:
        hb = Heartbeat(HEARTBEAT_INTERVAL)
        hb_task = asyncio.create_task(hb.run())
        measured = 0
        target = n - WARMUP
        start = time.perf_counter()
        while measured < target:
            k = await consume_one()
            if k == 0:
                break
            measured += k
        elapsed = time.perf_counter() - start
        hb.stop()
        await hb_task
    finally:
        gc.enable()
    return elapsed, measured, hb.stats()


async def _bench_wavecord(port, n):
    import wavecord
    from wavecord.events import decode

    node = wavecord.Node(
        "127.0.0.1", port, "youshallnotpass", "1", version="4", reconnect=False
    )
    await node.connect()

    async def consume_one():
        batch = await node.next_events(BATCH)
        if batch is None:
            return 0
        for raw in batch:
            decode(raw)
        return len(batch)

    return await _measured(consume_one, n)


async def _bench_pure_python(port, n):
    import websockets

    url = f"ws://127.0.0.1:{port}/v4/websocket"
    async with websockets.connect(url, max_queue=None) as ws:
        await ws.recv()  # the ready message

        async def consume_one():
            raw = await ws.recv()
            obj = json.loads(raw)
            state = obj.get("state") or {}
            _ = (obj.get("op"), obj.get("guildId"), state.get("position"))
            return 1

        return await _measured(consume_one, n)


def _median(values):
    s = sorted(values)
    return s[len(s) // 2]


async def _amain():
    srv = mp.Process(target=_server_process, args=(PORT, N_EVENTS), daemon=True)
    srv.start()
    await asyncio.sleep(1.0)  # let the server bind

    agg = {"pure_python": [], "wavecord": []}
    try:
        for t in range(TRIALS):
            e, c, j = await _bench_pure_python(PORT, N_EVENTS)
            r = c / e if e else 0.0
            agg["pure_python"].append((r, j))
            print(
                f"[trial {t + 1}] pure-Python {r:>10,.0f} ev/s | "
                f"jitter p50 {j['p50']:.2f} p99 {j['p99']:.2f} max {j['max']:.2f} ms"
            )
            await asyncio.sleep(0.3)
            e, c, j = await _bench_wavecord(PORT, N_EVENTS)
            r = c / e if e else 0.0
            agg["wavecord"].append((r, j))
            print(
                f"[trial {t + 1}] WaveCord    {r:>10,.0f} ev/s | "
                f"jitter p50 {j['p50']:.2f} p99 {j['p99']:.2f} max {j['max']:.2f} ms"
            )
            await asyncio.sleep(0.3)
    finally:
        srv.terminate()
        srv.join()

    summary = {}
    for name, rows in agg.items():
        summary[name] = {
            "rate": _median([r for r, _ in rows]),
            "p50": _median([j["p50"] for _, j in rows]),
            "p99": _median([j["p99"] for _, j in rows]),
        }

    pp, wc = summary["pure_python"], summary["wavecord"]
    print("\nmedian over", TRIALS, "trials")
    print(f"pure-Python: {pp['rate']:>10,.0f} ev/s | loop p50 {pp['p50']:.2f}ms p99 {pp['p99']:.2f}ms")
    print(f"WaveCord: {wc['rate']:>10,.0f} ev/s | loop p50 {wc['p50']:.2f}ms p99 {wc['p99']:.2f}ms")
    print(f"\nthroughput: WaveCord {wc['rate'] / pp['rate']:.2f}x")
    print(f"loop p50 jitter: WaveCord {pp['p50'] / wc['p50']:.2f}x smoother")
    print(f"loop p99 jitter: WaveCord {pp['p99'] / wc['p99']:.2f}x smoother")
    return summary


if __name__ == "__main__":
    mp.set_start_method("spawn", force=True)
    asyncio.run(_amain())
