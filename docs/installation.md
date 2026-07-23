# Installation

WaveCord ships prebuilt wheels on PyPI, so there is no Rust toolchain to install.

```bash
pip install wavecord
```

Add the Discord library you use as an extra so it is installed alongside WaveCord:

=== "discord.py"

    ```bash
    pip install "wavecord[discordpy]"
    ```

=== "py-cord"

    ```bash
    pip install "wavecord[pycord]"
    ```

=== "disnake"

    ```bash
    pip install "wavecord[disnake]"
    ```

=== "nextcord"

    ```bash
    pip install "wavecord[nextcord]"
    ```

## Requirements

- Python 3.9 or newer.
- A running Lavalink v3 or v4 node. See the [Lavalink documentation](https://lavalink.dev)
  for how to run one.
- One of the supported Discord libraries, for voice.

## A Lavalink node

WaveCord talks to a Lavalink server; it does not bundle one. The quickest way to
get a node for local development is the official Docker image:

```bash
docker run --rm -p 2333:2333 \
  -e SERVER_PORT=2333 \
  -e LAVALINK_SERVER_PASSWORD=youshallnotpass \
  ghcr.io/lavalink-devs/lavalink:4
```

`youshallnotpass` is Lavalink's documented default password. Change it for
anything beyond local testing.

## From source

If you want to build from source (for development or an unsupported platform),
you need a Rust toolchain and [maturin](https://www.maturin.rs/):

```bash
git clone https://github.com/Apart-Studio/wavecord
cd wavecord
python -m venv .venv && source .venv/bin/activate
pip install ".[dev]"
maturin develop
```
