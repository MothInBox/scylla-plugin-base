# Scylla Plugin Base

WebAssembly plugins for [Scylla Reader](https://github.com/MothInBox/scylla-reader) — the TUI reader that scrapes web novels via Extism plugins.

## Plugins

| Plugin | Domain | Description |
|--------|--------|-------------|
| `plugin-template` | `template` | Reference implementation — a minimal plugin showing the full API surface |
| `plugin-royalroad` | `royalroad` | Scraper for RoyalRoad |

## Layout

- `scylla-plugin-api/` — the shared plugin API contract (`PluginSchema`, `ScrapeInput`, `ScrapeOutput`, `ChapterOutput`). Plugins depend on this crate.
- `plugin-*/` — individual plugins, each compiled to a `plugin-<domain>.wasm` binary.

## Building

Requires the `wasm32-unknown-unknown` target:

```sh
rustup target add wasm32-unknown-unknown
cargo build --release
```

Each plugin's `.cargo/config.toml` sets the wasm target and the getrandom custom backend. The built binaries land in `target/wasm32-unknown-unknown/release/`.

## Installing

Copy the built `.wasm` files to the Scylla Reader plugins directory:

```sh
cp target/wasm32-unknown-unknown/release/plugin-*.wasm ~/.config/scylla-reader/plugins/
```

Scylla Reader discovers `plugin-<domain>.wasm` files on startup — the domain is extracted from the filename (e.g. `plugin-royalroad.wasm` registers for `royalroad`).

## Writing a plugin

See `plugin-template/` for the reference implementation. A plugin exports three functions:

- `get_config_schema` — the config fields the plugin accepts
- `scrape_book` — given a book URL, return the book metadata + chapter list
- `scrape_chapter` — given a chapter URL, return the chapter title + content

Plugins fetch pages via the host's `curl_fetch` function (with optional cookies) and can call `scylla_fail` to abort with an error message.

> [!WARNING]
> Plugins run arbitrary wasm on your machine. Verify a plugin yourself or only use trusted sources.