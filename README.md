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

Copy the `plugin-template/` crate as a starting point. A plugin is a Rust crate that targets `wasm32-unknown-unknown` and depends on `scylla-plugin-api` and `extism-pdk`:

```toml
# Cargo.toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
scylla-plugin-api = { path = "../scylla-plugin-api" }
extism-pdk = "1"
serde_json = "1"
```

Implement the three plugin exports:

```rust
#[plugin_fn]
pub fn get_config_schema(Json(()): Json<()>) -> FnResult<Json<PluginSchema>> {
    Ok(Json(PluginSchema {
        fields: vec![
            ConfigField {
                key: "key".into(),
                label: "Display label".into(),
                field_type: "string".into(),
                default: "".into(),
            },
        ],
        accepts_cookies: false,
    }))
}

#[plugin_fn]
pub fn scrape_book(Json(input): Json<ScrapeInput>) -> FnResult<Json<ScrapeOutput>> {
    // Use host_curl_fetch to fetch pages, parse HTML, return book metadata + chapters
}

#[plugin_fn]
pub fn scrape_chapter(Json(input): Json<ScrapeInput>) -> FnResult<Json<ChapterOutput>> {
    // Return chapter title + content
}
```

Plugins fetch pages via the host's `curl_fetch` function (with optional cookies) and can call `scylla_fail` to abort with an error message.

## Releasing

Build the plugin, then create a GitHub release with the `.wasm` file(s) attached as assets:

```sh
cargo build --target wasm32-unknown-unknown --release
gh release create v0.1.0 target/wasm32-unknown-unknown/release/plugin-my-plugin.wasm
```

The asset must be named `plugin-<domain>.wasm` — the domain is extracted from the filename (e.g. `plugin-example.com.wasm` registers for `example.com`). GitHub releases are flat: every `.wasm` asset attached to the release is downloaded, regardless of directory structure. A single release can bundle multiple plugins.

Users install by entering the repo URL in Scylla Reader's "Install Plugin from GitHub" (command palette). Scylla fetches the **latest stable release** and installs its `plugin-<domain>.wasm` assets.

> [!WARNING]
> Plugins run arbitrary wasm on your machine. Verify a plugin yourself or only use trusted sources.